use crate::codec::{Key, Value};
use crate::storage::Engine;
use crate::util::Result;
use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Included};
use std::sync::RwLock;

pub struct InMemEngine<K, V>
where
    K: Key,
    V: Value,
{
    inner: RwLock<BTreeMap<K, V>>,
}

unsafe impl<K, V> Send for InMemEngine<K, V>
where
    K: Key,
    V: Value,
{
}
unsafe impl<K, V> Sync for InMemEngine<K, V>
where
    K: Key,
    V: Value,
{
}

impl<K, V> InMemEngine<K, V>
where
    K: Key,
    V: Value,
{
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(BTreeMap::new()),
        }
    }
}

impl<K, V> Engine for InMemEngine<K, V>
where
    K: Key,
    V: Value,
{
    type K = K;
    type V = V;

    fn put(&self, k: K, v: V) -> Result<()> {
        let mut inner = self.inner.write().unwrap();
        if let Some(prev_v) = inner.get_mut(&k) {
            *prev_v = v;
        } else {
            inner.insert(k, v);
        }
        Ok(())
    }

    fn del(&self, k: &K) -> Result<()> {
        let mut inner = self.inner.write().unwrap();
        inner.remove(&k);
        Ok(())
    }

    fn get(&self, k: &K) -> Result<Option<Self::V>> {
        let inner = self.inner.read().unwrap();
        let v = inner.get(k).map(|v| v.to_owned());
        Ok(v)
    }

    fn scan(&self, lower: &K, upper: &K) -> Result<Vec<Self::V>> {
        let inner = self.inner.read().unwrap();
        let (lower, upper) = (Included(lower), Excluded(upper));
        let mut res = vec![];
        for (_, value) in inner.range((lower, upper)) {
            res.push(value.to_owned());
        }
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Key for i32 {}
    impl Value for i32 {}

    #[test]
    fn test_in_mem_engine() {
        let engine = InMemEngine::new();
        for i in 0..1000 {
            let v = engine.get(&i).unwrap();
            assert_eq!(v, None);
            engine.put(i, i).unwrap();
            let v = engine.get(&i).unwrap();
            assert_eq!(v, Some(i));
        }
        for i in 0..1000 {
            let v;
            let expected = if i % 2 == 0 {
                v = 2 * i;
                Some(v)
            } else {
                None
            };
            match expected {
                Some(v) => engine.put(i, v).unwrap(),
                None => engine.del(&i).unwrap(),
            }
            let v = engine.get(&i).unwrap();
            assert_eq!(v, expected);
        }
        let vs = engine.scan(&995, &1002).unwrap();
        assert_eq!(vs, vec![1992, 1996]);
    }
}
