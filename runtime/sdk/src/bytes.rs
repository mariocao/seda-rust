use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bytes(Vec<u8>);

impl Bytes {
    pub fn eject(self) -> Vec<u8> {
        self.0
    }
}

/// We implement Deref over the Bytes type allowing us to avoid a clone for
/// `FromBytes`.
impl Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

pub trait ToBytes {
    fn to_bytes(self) -> Bytes;
}

impl ToBytes for Bytes {
    fn to_bytes(self) -> Bytes {
        self
    }
}

impl ToBytes for Vec<u8> {
    fn to_bytes(self) -> Bytes {
        Bytes(self)
    }
}

impl ToBytes for String {
    fn to_bytes(self) -> Bytes {
        Bytes(self.as_bytes().to_vec())
    }
}

pub trait FromBytes
where
    Self: Sized,
{
    fn from_bytes(bytes: &[u8]) -> Result<Self>;

    fn from_bytes_vec(bytes: Vec<u8>) -> Result<Self>;
}

impl FromBytes for Vec<u8> {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(bytes.to_vec())
    }

    fn from_bytes_vec(bytes: Vec<u8>) -> Result<Self> {
        Ok(bytes)
    }
}

impl FromBytes for String {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(std::str::from_utf8(bytes)?.into())
    }

    fn from_bytes_vec(bytes: Vec<u8>) -> Result<Self> {
        Self::from_bytes(bytes.as_slice())
    }
}

macro_rules! bytes_impls_le_bytes {
    ($type_:ty, $num_bytes:expr) => {
        impl ToBytes for $type_ {
            fn to_bytes(self) -> Bytes {
                Bytes(self.to_le_bytes().to_vec())
            }
        }

        impl FromBytes for $type_ {
            fn from_bytes(bytes: &[u8]) -> Result<Self> {
                let bytes: [u8; $num_bytes] = bytes.try_into()?;
                Ok(<$type_>::from_le_bytes(bytes))
            }

            fn from_bytes_vec(bytes: Vec<u8>) -> Result<Self> {
                Self::from_bytes(bytes.as_slice())
            }
        }
    };
}

bytes_impls_le_bytes!(u8, 1);
bytes_impls_le_bytes!(u32, 4);
bytes_impls_le_bytes!(u64, 8);
bytes_impls_le_bytes!(u128, 16);
bytes_impls_le_bytes!(i8, 1);
bytes_impls_le_bytes!(i32, 4);
bytes_impls_le_bytes!(i64, 8);
bytes_impls_le_bytes!(i128, 16);
bytes_impls_le_bytes!(f32, 4);
bytes_impls_le_bytes!(f64, 8);