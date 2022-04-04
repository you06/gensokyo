use async_trait::async_trait;

use gensokyo::codec::byte::{ByteKey as Key, ByteValue as Value};
use gensokyo::storage::InMemEngine;
use gensokyo::util::Result;
use gensokyo::Cluster;
use gensokyo::{codec, node, request, storage, txn, shard};
use std::sync::Arc;

fn main() {
    // let client = ();
    // let cluster = Cluster::new();
}

enum Request {
    Put(Key, Value),
    Del(Key),
    Get(Key),
    Scan(Key, Key),
}

enum Response {
    Put,
    Del,
    Get(Option<Value>),
    Scan(Vec<Value>),
}

struct StorageNode<E>
where
    E: storage::Engine<K = Key, V = Value>,
{
    engine: Arc<E>,
}

#[async_trait]
impl<E> node::Node for StorageNode<E>
where
    E: storage::Engine<K = Key, V = Value>,
{
    type Req = Request;
    type Res = Response;

    async fn process(&self, req: Self::Req) -> Result<Self::Res> {
        let engine = self.engine.clone();
        Ok(match req {
            Request::Put(k, v) => {
                engine.put(k, v)?;
                Response::Put
            }
            Request::Del(k) => {
                engine.del(&k)?;
                Response::Del
            }
            Request::Get(k) => Response::Get(engine.get(&k)?),
            Request::Scan(lower, upper) => Response::Scan(engine.scan(&lower, &upper)?),
        })
    }
}

type Shard = shard::KeySpaceSpilt<Key, request::channel::ChannelSender<Request, Response>>;

// in disaggregate structure, compute node is stateless.
struct ServerNode {
    shard: Arc<Shard>,
}

impl ServerNode {}

#[async_trait]
impl node::Node for ServerNode {
    type Req = Request;
    type Res = Response;

    async fn process(&self, req: Self::Req) -> Result<Self::Res> {
        unimplemented!()
    }
}

#[async_trait]
impl node::Server for ServerNode {
    type S = Shard;

    fn register_shard(&mut self, s: Arc<Self::S>) {
        self.shard = s;
    }

    async fn execute(&self, t: txn::Txn) -> Result<()> {
        Ok(())
    }
}
