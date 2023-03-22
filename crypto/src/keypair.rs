use std::fmt;

use bn254::{PrivateKey as Bn254PrivateKey, PublicKey as Bn254PublicKey};
use concat_kdf::derive_key;
use ed25519_dalek::{PublicKey as Ed25519PublicKey, SecretKey as Ed25519PrivateKey, SECRET_KEY_LENGTH};
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize,
    Deserializer,
    Serialize,
};

use super::Result;
use crate::MasterKey;

#[derive(PartialEq, Eq)]
pub enum KeyType {
    Bn254,
    Ed25519,
}

/// KeyPair type has a two generics, one to represent the private and public key
/// types. It defaults to a Bn254 KeyPair.
#[derive(Debug, Clone)]
pub struct KeyPair<Private, Public> {
    pub private_key: Private,
    pub public_key:  Public,
}

// TODO: Remove serialize and deserialize impls
pub type Ed25519KeyPair = KeyPair<Ed25519PrivateKey, Ed25519PublicKey>;
pub type Bn254KeyPair = KeyPair<Bn254PrivateKey, Bn254PublicKey>;

impl Serialize for Ed25519KeyPair {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Ed25519KeyPair", 2)?;
        state.serialize_field("private_key", &self.private_key.as_bytes())?;
        state.serialize_field("public_key", &self.public_key.as_bytes())?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for Ed25519KeyPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            PrivateKey,
            PublicKey,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`secs` or `nanos`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "private_key" => Ok(Field::PrivateKey),
                            "public_key" => Ok(Field::PublicKey),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct Ed25519KeyPairVisitor;

        impl<'de> Visitor<'de> for Ed25519KeyPairVisitor {
            type Value = Ed25519KeyPair;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Duration")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Ed25519KeyPair, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let private_key = Ed25519PrivateKey::from_bytes(
                    seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?,
                )
                .unwrap();
                let public_key = Ed25519PublicKey::from_bytes(
                    seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?,
                )
                .unwrap();

                Ok(Ed25519KeyPair {
                    private_key,
                    public_key,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Ed25519KeyPair, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut private_key = None;
                let mut public_key = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::PrivateKey => {
                            if private_key.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            private_key = Some(Ed25519PrivateKey::from_bytes(map.next_value()?).unwrap());
                        }
                        Field::PublicKey => {
                            if public_key.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            public_key = Some(Ed25519PublicKey::from_bytes(map.next_value()?).unwrap());
                        }
                    }
                }
                let private_key = private_key.ok_or_else(|| de::Error::missing_field("private_key"))?;
                let public_key = public_key.ok_or_else(|| de::Error::missing_field("public_key"))?;
                Ok(Ed25519KeyPair {
                    private_key,
                    public_key,
                })
            }
        }

        const FIELDS: &[&str] = &["private_key", "public_key"];
        deserializer.deserialize_struct("Ed25519KeyPair", FIELDS, Ed25519KeyPairVisitor)
    }
}

impl Serialize for Bn254KeyPair {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Bn254KeyPair", 2)?;
        state.serialize_field("private_key", &self.private_key.to_bytes().unwrap())?;
        state.serialize_field("public_key", &self.public_key.to_compressed().unwrap())?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for Bn254KeyPair {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            PrivateKey,
            PublicKey,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`secs` or `nanos`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "private_key" => Ok(Field::PrivateKey),
                            "public_key" => Ok(Field::PublicKey),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct Bn254KeyPairVisitor;

        impl<'de> Visitor<'de> for Bn254KeyPairVisitor {
            type Value = Bn254KeyPair;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Duration")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Bn254KeyPair, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let private_key_bytes: Vec<u8> = seq.next_element().unwrap().unwrap();
                let private_key = Bn254PrivateKey::try_from(private_key_bytes.as_slice()).unwrap();

                let public_key_bytes: Vec<u8> = seq.next_element().unwrap().unwrap();
                let public_key = Bn254PublicKey::from_compressed(public_key_bytes.as_slice()).unwrap();

                Ok(Bn254KeyPair {
                    private_key,
                    public_key,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Bn254KeyPair, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut private_key = None;
                let mut public_key = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::PrivateKey => {
                            if private_key.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            let private_key_bytes: Vec<u8> = map.next_value()?;
                            private_key = Some(Bn254PrivateKey::try_from(private_key_bytes.as_slice()).unwrap());
                        }
                        Field::PublicKey => {
                            if public_key.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            let public_key_bytes: Vec<u8> = map.next_value()?;
                            public_key = Some(Bn254PublicKey::from_compressed(public_key_bytes).unwrap());
                        }
                    }
                }
                let private_key = private_key.ok_or_else(|| de::Error::missing_field("private_key"))?;
                let public_key = public_key.ok_or_else(|| de::Error::missing_field("public_key"))?;
                Ok(Bn254KeyPair {
                    private_key,
                    public_key,
                })
            }
        }

        const FIELDS: &[&str] = &["private_key", "public_key"];
        deserializer.deserialize_struct("Bn254KeyPair", FIELDS, Bn254KeyPairVisitor)
    }
}

impl MasterKey {
    pub fn derive_bn254(&self, index: usize) -> Result<KeyPair<Bn254PrivateKey, Bn254PublicKey>> {
        let master_sk = derive_key::<sha2::Sha256>(&self.seed, b"bn254", SECRET_KEY_LENGTH)?;
        let sk = derive_key::<sha2::Sha256>(master_sk.as_slice(), &index.to_ne_bytes(), SECRET_KEY_LENGTH)?;
        let private_key = Bn254PrivateKey::try_from(sk.as_slice()).unwrap();
        let public_key = Bn254PublicKey::from_private_key(&private_key);

        Ok(KeyPair {
            public_key,
            private_key,
        })
    }

    pub fn derive_ed25519(&self, index: usize) -> Result<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>> {
        let master_sk = derive_key::<sha2::Sha256>(&self.seed, b"ed25519", SECRET_KEY_LENGTH)?;
        let sk = derive_key::<sha2::Sha256>(master_sk.as_slice(), &index.to_ne_bytes(), SECRET_KEY_LENGTH)?;
        let private_key = Ed25519PrivateKey::from_bytes(sk.as_slice()).unwrap();
        let public_key: Ed25519PublicKey = (&private_key).into();

        Ok(KeyPair {
            public_key,
            private_key,
        })
    }
}
