use crate::cluster::Cluster;
use crate::txn::Txn;

pub trait Client<C: Cluster> {
    type Cluster = C;

    pub fn begin() -> Txn;
}

pub struct InteractiveTxnClient<C: Cluster> {
    cluster: C,
}

impl<C> Client<C> for InteractiveTxnClient<C> where C: Cluster {
    type Cluster = C;
}
