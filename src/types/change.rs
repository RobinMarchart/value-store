use std::{
    cmp::Ordering::{Equal, Greater, Less},
    fmt::{self, LowerHex, UpperHex},
};

use serde::{
    de::{self, Visitor},
    ser::SerializeSeq,
    Deserialize, Serialize,
};

use crate::error::ValueStoreError;

use super::{PathElement, Value};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum ChangeContent {
    Insert {
        path: Vec<PathElement>,
        value: Value,
    },
    Replace {
        path: Vec<PathElement>,
        old: Value,
        new: Value,
    },
    Delete {
        path: Vec<PathElement>,
        old: Value,
    },
}

pub type Hash = [u8; 32];

pub fn format_hash_lower(hash: &Hash, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("0x")?;
    for v in hash {
        LowerHex::fmt(v, f)?
    }
    Ok(())
}
pub fn format_hash_upper(hash: &Hash, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("0x")?;
    for v in hash {
        UpperHex::fmt(v, f)?
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub enum Parents {
    One(Hash),
    Two(Hash, Hash),
}

impl Parents {
    pub fn one(p: Hash) -> Result<Self, ValueStoreError> {
        Ok(Self::One(p))
    }
    pub fn two(p1: Hash, p2: Hash) -> Result<Self, ValueStoreError> {
        match p1.cmp(&p2) {
            Less => Ok(Self::Two(p1, p2)),
            Equal => Err(ValueStoreError::ParentHashSame),
            Greater => Ok(Self::Two(p2, p1)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Change {
    pub hash: Hash,
    pub parents: Parents,
    pub content: Vec<ChangeContent>,
}

impl Serialize for Parents {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Parents::One(p) => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                seq.serialize_element(p)?;
                seq.end()
            }
            Parents::Two(p1, p2) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element(p1)?;
                seq.serialize_element(p2)?;
                seq.end()
            }
        }
    }
}

struct ParentsVisitor {}

impl<'de> Visitor<'de> for ParentsVisitor {
    type Value = Parents;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A sequence of one or two hashes")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        if let Some(p1) = seq.next_element()? {
            if let Some(p2) = seq.next_element()? {
                if seq.next_element::<Hash>()?.is_none() {
                    if p1 < p2 {
                        Ok(Parents::Two(p1, p2))
                    } else {
                        Err(<A::Error as de::Error>::custom("parents not ordered"))
                    }
                } else {
                    Err(<A::Error as de::Error>::invalid_length(
                        seq.size_hint().unwrap_or(3),
                        &self,
                    ))
                }
            } else {
                Ok(Parents::One(p1))
            }
        } else {
            Err(<A::Error as de::Error>::invalid_length(0, &self))
        }
    }
}

impl<'de> Deserialize<'de> for Parents {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ParentsVisitor {})
    }
}
