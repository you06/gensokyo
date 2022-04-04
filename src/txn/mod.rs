use crate::codec::{Key, Value};
use crate::util::Result;
use crate::node::Server;
use async_trait::async_trait;

#[async_trait]
pub trait Txn {
    type Server: Server;

    /// Txn executes in 2 phases:
    /// 1. execute,
    /// 2. commit or rollback.
    async fn execute(&mut self, server: &Self::Server) -> Result<()>;
    async fn commit(&mut self, server: &Self::Server) -> Result<()>;
    async fn rollback(&mut self, server: &Self::Server) -> Result<()>;
}

pub mod kv_ops;
