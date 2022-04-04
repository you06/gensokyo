use crate::codec::{Key, Value};
use crate::util::Result;

mod in_mem;
pub use in_mem::InMemEngine;

pub trait Engine: Sync + Send {
    type K: Key;
    type V: Value + ToOwned<Owned = Self::V>;

    // The borrow check requires a high-level lock, to avoid it, we just use the immutable borrow.
    // Use unsafe when it's required.
    fn put(&self, k: Self::K, v: Self::V) -> Result<()>;
    fn del(&self, k: &Self::K) -> Result<()>;
    // you should return value with owner ship, because it'll be sent to other nodes(other machines in real world)
    fn get(&self, k: &Self::K) -> Result<Option<Self::V>>;
    /// scan get the values between [lower, upper).
    fn scan(&self, lower: &Self::K, upper: &Self::K) -> Result<Vec<Self::V>>;
}

pub trait SnapshotEngine {}
