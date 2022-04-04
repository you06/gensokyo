use crate::node::{Node, Server};
use crate::request::{Request, Response, Sender};
use crate::storage::Engine;
use crate::txn::Txn;
use crate::util::{RequestError, Result};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Cluster<'a, N, S, Req, Res, SE>
where
    N: Node,
    S: Server,
    Req: Request,
    Res: Response,
    SE: Sender,
{
    phantom: PhantomData<(Req, Res)>,
    nodes: Vec<&'a N>,
    servers: Vec<&'a S>,
    map: BTreeMap<u64, SE>,
    id: AtomicU64,
}

impl<'a, N, S, Req, Res, SE> Cluster<'a, N, S, Req, Res, SE>
where
    N: Node,
    S: Server,
    Req: Request,
    Res: Response,
    SE: Sender<Res = Res, Req = Req>,
{
    pub fn new() -> Result<Self> {
        Ok(Cluster {
            nodes: vec![],
            servers: vec![],
            phantom: PhantomData,
            id: AtomicU64::new(0),
            map: BTreeMap::new(),
        })
    }

    fn get_node_id(&mut self) -> u64 {
        self.id.fetch_add(1, Ordering::Relaxed)
    }

    fn join_node(&mut self, sender: SE, n: &'a mut N) {
        self.nodes.push(n);
        let id = self.get_node_id();
        self.map.insert(id, sender);
    }

    fn join_server(&mut self, sender: SE, s: &'a mut S) {
        self.servers.push(s);
        let id = self.get_node_id();
        self.map.insert(id, sender);
    }

    pub async fn send(&mut self, id: u64, req: Req) -> Result<Res> {
        match self.map.get_mut(&id) {
            Some(n) => n
                .send(req)
                .await
                .map_err(|e| RequestError::SendError(e.to_string()).into()),
            None => panic!("node not found"),
        }
    }
}
