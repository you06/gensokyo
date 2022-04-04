use crate::codec::{Key, Value};
use std::borrow::ToOwned;
use std::cmp::Ordering;

#[derive(Eq, PartialEq)]
pub struct ByteKey {
    inner: Vec<u8>,
}

pub struct ByteValue {
    inner: Vec<u8>,
}

impl Key for ByteKey {}

impl PartialOrd for ByteKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl Ord for ByteKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl ToOwned for ByteKey {
    type Owned = Self;

    fn to_owned(&self) -> Self::Owned {
        ByteKey {
            inner: self.inner.to_owned(),
        }
    }
}

impl<const N: usize> From<&[u8; N]> for ByteKey {
    fn from(t: &[u8; N]) -> ByteKey {
        ByteKey {
            inner: t.to_vec(),
        }
    }
}

impl ToString for ByteKey {
    fn to_string(&self) -> String {
        match std::str::from_utf8(&self.inner) {
            Ok(v) => v.to_owned(),
            Err(e) => format!("Invalid UTF-8 sequence: {}", e),
        }
    }
}

impl ByteKey {
    pub fn new(bytes: &[u8]) -> ByteKey {
        ByteKey { inner: bytes.to_owned() }
    }
}

impl Value for ByteValue {}

impl ToOwned for ByteValue {
    type Owned = Self;

    fn to_owned(&self) -> Self::Owned {
        ByteValue {
            inner: self.inner.to_owned(),
        }
    }
}

impl ToString for ByteValue {
    fn to_string(&self) -> String {
        match std::str::from_utf8(&self.inner) {
            Ok(v) => v.to_owned(),
            Err(e) => format!("Invalid UTF-8 sequence: {}", e),
        }
    }
}
