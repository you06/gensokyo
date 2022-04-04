use crate::node::Node;
use crate::util::Result;
use async_trait::async_trait;

pub trait Request: Send {}
pub trait Response: Send {}

impl<T: Send> Request for T {}
impl<T: Send> Response for T {}

#[async_trait]
pub trait Sender: Send {
    type Req: Request;
    type Res: Response;

    async fn send(&self, req: Self::Req) -> Result<Self::Res>;
    fn close(&mut self);
}

#[async_trait]
pub trait Receiver {
    type Req: Request;
    type Res: Response;
    type N: Node;

    // polling requests and send the response when it's ready.
    async fn collect_req(mut self);
}

pub mod channel;
