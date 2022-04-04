use std::collections::BTreeMap;
use crate::codec::Key;
use crate::request::Sender;
use crate::shard::Shard;
use crate::util::{ShardError, Either, Result};
use std::ops::Bound::{Included, Excluded, Unbounded};

pub struct KeySpaceSpilt<K: Key, S: Sender> {
    inner: BTreeMap<K, S>,
    begin: Option<S>,
}

impl<K: Key, S: Sender> KeySpaceSpilt<K, S> {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
            begin: None,
        }
    }

    fn left_key(&self, key: &K) -> Option<K> {
        let (lower, upper) = (Unbounded, Excluded(key));
        self.inner.range((lower, upper)).next_back().map(|(k, _)| (k.to_owned()))
    }

    fn left_value_include(&self, key: &K) -> Option<&S> {
        let (lower, upper) = (Unbounded, Included(key));
        self.inner.range((lower, upper)).next_back().map(|(_, sender)| sender)
    }
}

impl<K: Key, S: Sender> Shard for KeySpaceSpilt<K, S> {
    type K = K;
    type S = S;

    fn split(&mut self, key: Self::K, sender: Either<Self::S>) -> Result<()> {
        if self.inner.get(&key).is_some() {
            return Err(ShardError::SplitError(key.to_string()).into());
        }
        let mut current= None;
        match sender {
            Either::Left(s) => {
                match self.left_key(&key) {
                    Some(k) => {
                        let before = self.inner.insert(k, s);
                        current = self.inner.insert(key, before.unwrap());
                    },
                    None => self.begin = Some(s),
                }
            },
            Either::Right(s) => {
                current = self.inner.insert(key, s);
            },
        };
        assert!(current.is_none());
        Ok(())
    }

    fn key2node(&self, key: &Self::K) -> &Self::S {
        match self.left_value_include(&key) {
            Some(sender) => sender,
            None => self.begin.as_ref().unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::byte::ByteKey;
    use std::sync::mpsc::{Sender as stdSender, Receiver as stdReceiver, channel};
    use async_trait::async_trait;
    use crate::util::Error;
    use crate::util::test::run_in_tokio;
    use std::sync::{Mutex};
    use std::fmt::Debug;

    struct MockSender<T: Copy + Send + Debug + PartialEq>(Mutex<stdSender<T>>, i32);

    impl<T: Copy + Send + Debug + PartialEq> MockSender<T> {
        fn new(i: i32) -> (Self, stdReceiver<T>) {
            let (tx, rx) = channel();
            (MockSender(Mutex::new(tx), i), rx)
        }
    }

    #[async_trait]
    impl<T: Copy + Send + Debug + PartialEq> Sender for MockSender<T> {
        type Req = T;
        type Res = T;

        async fn send(&self, req: Self::Req) -> Result<Self::Res> {
            let sender = self.0.lock().unwrap();
            sender.send(req).unwrap();
            Err(Error::Unknown)
        }

        fn close(&mut self) {
            unreachable!()
        }
    }

    async fn send_and_check<T: Copy + Send + Debug + PartialEq> (sender: &MockSender<T>, req: T, rx: &mut stdReceiver<T>) {
        let send_res = sender.send(req).await;
        assert_eq!(send_res.unwrap_err(), Error::Unknown);
        let res = rx.recv().unwrap();
        assert_eq!(res, req);
    }

    #[test]
    fn test_split() {
        let mut key_space_spilt = KeySpaceSpilt::new();
        let k1 = ByteKey::new(b"b");
        let k2 = ByteKey::new(b"d");
        let k3 = ByteKey::new(b"f");
        let (sender1, mut rx1) = MockSender::new(1);
        let (sender2, mut rx2) = MockSender::new(2);
        let (sender3, mut rx3) = MockSender::new(3);

        // [k1...k2...k3...]
        key_space_spilt.split(k1, Either::Left(sender1)).unwrap();
        key_space_spilt.split(k2, Either::Right(sender2)).unwrap();
        key_space_spilt.split(k3, Either::Right(sender3)).unwrap();

        run_in_tokio(async move {
            let sender = key_space_spilt.key2node(&b"a".into());
            assert_eq!(sender.1, 1);
            send_and_check(sender, 1, &mut rx1).await;

            let sender = key_space_spilt.key2node(&b"c".into());
            assert_eq!(sender.1, 1);
            send_and_check(sender, 2, &mut rx1).await;

            let sender = key_space_spilt.key2node(&b"d".into());
            assert_eq!(sender.1, 2);
            send_and_check(sender, 3, &mut rx2).await;

            let sender = key_space_spilt.key2node(&b"e".into());
            assert_eq!(sender.1, 2);
            send_and_check(sender, 4, &mut rx2).await;

            let sender = key_space_spilt.key2node(&b"f".into());
            assert_eq!(sender.1, 3);
            send_and_check(sender, 5, &mut rx3).await;
        });
    }
}
