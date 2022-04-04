use crate::request::{Request, Response};
use crate::txn::Txn;
use crate::util::Result;
use crate::shard::Shard;
use std::sync::Arc;
use async_trait::async_trait;

#[async_trait]
pub trait Node {
    type Req: Request;
    type Res: Response;

    async fn process(&self, req: Self::Req) -> Result<Self::Res>;
}

#[async_trait]
pub trait Server: Node + Sync {
    type S: Shard;

    fn register_shard(&mut self, s: Arc<Self::S>);
    async fn execute(&self, t: Txn) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::test::run_in_tokio;
    use tokio::sync::mpsc::{channel, Receiver, Sender};

    struct TestNode<T> {
        tx: Sender<T>,
        rx: Receiver<T>,
    }

    impl<T> TestNode<T> {
        fn new() -> Self {
            let (tx, rx) = channel(1024);
            Self { tx, rx }
        }
    }

    #[test]
    fn test_node_wrapper() {}
}
