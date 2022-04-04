use async_trait::async_trait;
use futures::task::Poll;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::node::Node;
use crate::request::{Receiver as myReceiver, Request, Response, Sender as mySender};
use crate::util::{poll_future_notify, RequestError, Result};

pub struct ChannelSender<Req: Request, Res: Response> {
    req_tx: Sender<(u64, Req)>,
    serial: AtomicU64,
    res_map: Arc<Mutex<HashMap<u64, Result<Res>>>>,
    terminate: Arc<AtomicBool>,
    pool: Runtime,
}

// TODO: consider Arc<Self> as method receiver.
pub struct ChannelReceiver<Req: Request, Res: Response, N: Node> {
    req_rx: Receiver<(u64, Req)>,
    res_tx: Sender<(u64, Result<Res>)>,
    pool: Runtime,
    node: Arc<N>,
    phantom: PhantomData<N>,
}

pub fn new_channel_connect<'a, Req: Request, Res: Response, N: Node<Req = Req, Res = Res>>(
    node: Arc<N>,
) -> ChannelSender<Req, Res>
where
    Req: Request + 'static,
    Res: Response + 'static,
    N: Node + Sync + Send + 'static,
{
    let (req_tx, req_rx) = channel(1024);
    let (res_tx, res_rx) = channel(1024);
    let pool = Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("request-handle-pool")
        .build()
        .unwrap();
    let tx = ChannelSender::new(req_tx, pool);
    let rx: ChannelReceiver<Req, Res, N> =
        ChannelReceiver::new(req_rx, res_tx, tx.terminate.clone(), node);
    let terminate = tx.terminate.clone();
    let res_map = tx.res_map.clone();
    tx.pool.spawn(polling_resp(res_rx, res_map, terminate));
    tx.pool.spawn(myReceiver::collect_req(rx));
    tx
}

async fn polling_resp<Res>(
    mut res_rx: Receiver<(u64, Result<Res>)>,
    res_map: Arc<Mutex<HashMap<u64, Result<Res>>>>,
    terminate: Arc<AtomicBool>,
) where
    Res: Response,
{
    loop {
        if terminate.load(Ordering::Acquire) {
            return;
        }
        match res_rx.recv().await {
            Some(res) => {
                let mut m = res_map.lock().unwrap();
                m.insert(res.0, res.1);
                drop(m);
            }
            None => return,
        }
    }
}

impl<Req: Request, Res: Response> ChannelSender<Req, Res> {
    fn new(req_tx: Sender<(u64, Req)>, pool: Runtime) -> Self {
        Self {
            req_tx,
            serial: AtomicU64::new(0),
            res_map: Arc::new(Mutex::new(HashMap::new())),
            terminate: Arc::new(AtomicBool::new(false)),
            pool,
        }
    }

    fn new_serial_id(&self) -> u64 {
        self.serial.fetch_add(1, Ordering::Relaxed)
    }
}

async fn wait_resp<Res>(id: u64, res_map: Arc<Mutex<HashMap<u64, Result<Res>>>>) -> Result<Res>
where
    Res: Response + 'static,
{
    let f = futures::future::poll_fn(move |cx| {
        cx.waker().wake_by_ref();
        let mut m = res_map.lock().unwrap();
        let poll_result = match m.remove(&id) {
            Some(res) => Poll::Ready(res),
            None => Poll::Pending,
        };
        drop(m);
        poll_result
    });
    // TODO: handle the error
    poll_future_notify(f)
}

#[async_trait]
impl<Req, Res> mySender for ChannelSender<Req, Res>
where
    Req: Request,
    Res: Response + 'static,
{
    type Req = Req;
    type Res = Res;

    async fn send(&self, req: Self::Req) -> Result<Res> {
        let serial_id = self.new_serial_id();
        self.req_tx
            .send((serial_id, req))
            .await
            .map_err(|e| RequestError::SendError(e.to_string()))?;
        wait_resp(serial_id, self.res_map.clone()).await
    }

    fn close(&mut self) {
        self.terminate.store(true, Ordering::Release);
    }
}

impl<Req, Res, N> ChannelReceiver<Req, Res, N>
where
    Req: Request,
    Res: Response,
    N: Node,
{
    fn new(
        req_rx: Receiver<(u64, Req)>,
        res_tx: Sender<(u64, Result<Res>)>,
        terminate: Arc<AtomicBool>,
        node: Arc<N>,
    ) -> Self {
        let pool = Builder::new_multi_thread()
            .worker_threads(1)
            .thread_name("request-handle-pool")
            .build()
            .unwrap();
        Self {
            req_rx,
            res_tx,
            pool,
            node,
            phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<Req, Res, N: Node<Req = Req, Res = Res>> myReceiver for ChannelReceiver<Req, Res, N>
where
    Req: Request + 'static,
    Res: Response + 'static,
    N: Node + Sync + Send + 'static,
{
    type Req = Req;
    type Res = Res;
    type N = N;

    async fn collect_req(mut self) {
        loop {
            match self.req_rx.recv().await {
                Some((serial_id, req)) => {
                    let node = self.node.clone();
                    let res_tx = self.res_tx.clone();
                    self.pool.spawn(async move {
                        let res = node.process(req).await;
                        if let Err(_) = res_tx.send((serial_id, res)).await {
                            return;
                        }
                    });
                }
                None => return,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::test::run_in_tokio;

    struct Plus1Node;

    #[async_trait]
    impl Node for Plus1Node {
        type Req = i32;
        type Res = i32;
        async fn process(&self, req: Self::Req) -> Result<Self::Res> {
            Ok(req + 1)
        }
    }
    struct Multi1Node;

    #[async_trait]
    impl Node for Multi1Node {
        type Req = i32;
        type Res = i32;
        async fn process(&self, req: Self::Req) -> Result<Self::Res> {
            Ok(2 * req)
        }
    }

    #[test]
    fn test_send_recv() {
        run_in_tokio(async move {
            let mut tx1 = new_channel_connect(Arc::new(Plus1Node {}));
            for i in 0..1000 {
                let r = tx1.send(i).await.unwrap();
                assert_eq!(i + 1, r);
            }
            let mut tx2 = new_channel_connect(Arc::new(Multi1Node {}));
            for i in 0..1000 {
                let r = tx2.send(i).await.unwrap();
                assert_eq!(2 * i, r);
            }
            // hack the test
            std::mem::forget(tx1);
            // hack the test
            std::mem::forget(tx2);
        });
    }
}
