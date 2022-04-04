use crate::codec::{Key, Value};
use crate::util::Result;
use crate::node::Server;
use crate::txn::Txn;
use async_trait::async_trait;

pub enum Op<K: Key, V: Value> {
    Put(K, V),
    Get(K),
    Del(K),
    Scan(K, K),
    Commit,
    Rollback,
}

pub enum KVOps<K: Key, V: Value> {
    Ops(Ops<K, V>),
    InteractiveTxn,
}

struct Ops<K: Key, V: Value> {
    ops: Vec<Op<K, V>>,
}

#[async_trait]
impl<K: Key, V: Value, S: Server> Txn for KVOps<K, V> {
    type Server = S;

    /// The execution order should be:
    /// 1. execute,
    /// 2. commit or rollback.
    async fn execute(&mut self, server: &Self::Server) -> Result<()> {

    }

    async fn commit(&mut self, server: &Self::Server) -> Result<()> {

    }

    async fn rollback(&mut self, server: &Self::Server) -> Result<()> {

    }
}
