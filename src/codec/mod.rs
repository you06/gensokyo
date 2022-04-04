pub trait Key: ToOwned<Owned = Self> + ToString + Ord {}
pub trait Value: ToOwned<Owned = Self> + ToString {}

pub mod byte;
