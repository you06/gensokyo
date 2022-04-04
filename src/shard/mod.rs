use crate::codec::Key;
use crate::request::Sender;
use crate::util::{Either, Result};

pub trait Shard {
    type K: Key;
    type S: Sender;

    /// the key space should be split into left right,
    /// in which { left_key < right_key | left_key ∈ left, right_key ∈ right }
    /// Either::Left will put the sender to the left region,
    /// Either::Right will put the sender to the right region.
    fn split(&mut self, key: Self::K, sender: Either<Self::S>) -> Result<()>;
    fn key2node(&self, key: &Self::K) -> &Self::S;
}

mod key_space_split;
pub use key_space_split::KeySpaceSpilt;
