use std::{collections::HashMap, fmt::Debug};

use serde::{de::Visitor, Deserialize, Serialize};

use crate::error::ValueStoreError;

use super::{change::ChangeContent, PathElement};

#[derive(Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Blob(Vec<u8>),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(v) => Debug::fmt(v, f),
            Value::Float(v) => Debug::fmt(v, f),
            Value::Bool(v) => Debug::fmt(v, f),
            Value::String(v) => Debug::fmt(v, f),
            Value::Array(v) => Debug::fmt(v, f),
            Value::Map(v) => Debug::fmt(v, f),
            Value::Blob(v) => Debug::fmt(v.as_slice(), f),
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Integer(v) => Serialize::serialize(v, serializer),
            Value::Float(v) => Serialize::serialize(v, serializer),
            Value::Bool(v) => Serialize::serialize(v, serializer),
            Value::String(v) => Serialize::serialize(v, serializer),
            Value::Array(v) => Serialize::serialize(v, serializer),
            Value::Map(v) => Serialize::serialize(v, serializer),
            Value::Blob(v) => serializer.serialize_bytes(v),
        }
    }
}

struct ValueVisitor {}

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("either u64, f64, a bool, string, array or map")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Integer(i64::try_from(v).map_err(|_| {
            serde::de::Error::invalid_type(serde::de::Unexpected::Unsigned(v), &self)
        })?))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Float(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::String(v.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::String(v))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut res = if let Some(len) = seq.size_hint() {
            Vec::with_capacity(len)
        } else {
            Vec::new()
        };
        while let Some(v) = seq.next_element()? {
            res.push(v)
        }
        Ok(Value::Array(res))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut res = if let Some(len) = map.size_hint() {
            HashMap::with_capacity(len)
        } else {
            HashMap::new()
        };
        while let Some((key, value)) = map.next_entry()? {
            res.insert(key, value);
        }
        Ok(Value::Map(res))
    }
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Blob(v.to_vec()))
    }
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Blob(v))
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor {})
    }
}

impl Value {
    pub fn get(&self, path: &[PathElement]) -> Option<&Value> {
        if let Some((this, next)) = path.split_first() {
            match (this, self) {
                (PathElement::Field(name), Value::Map(map)) => {
                    if let Some(entry) = map.get(name) {
                        entry.get(next)
                    } else {
                        None
                    }
                }
                (PathElement::Index(index), Value::Array(arr)) => {
                    if let Some(entry) = arr.get(*index as usize) {
                        entry.get(next)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            Some(self)
        }
    }
    pub fn get_mut(&mut self, path: &[PathElement]) -> Option<&mut Value> {
        if let Some((this, next)) = path.split_first() {
            match (this, self) {
                (PathElement::Field(name), Value::Map(map)) => {
                    if let Some(entry) = map.get_mut(name) {
                        entry.get_mut(next)
                    } else {
                        None
                    }
                }
                (PathElement::Index(index), Value::Array(arr)) => {
                    if let Some(entry) = arr.get_mut(*index as usize) {
                        entry.get_mut(next)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            Some(self)
        }
    }
    pub fn apply_iter<I: IntoIterator<Item = ChangeContent>>(
        &mut self,
        i: I,
    ) -> Result<(), ValueStoreError> {
        for change in i {
            match change {
                ChangeContent::Insert { path, value } => self.apply_insert(&path, value,&path)?,
                ChangeContent::Replace { path, old, new } => self.apply_replace(&path, old, new,&path)?,
                ChangeContent::Delete { path, old } => self.apply_delete(&path, old,&path)?,
            }
        }
        Ok(())
    }

    pub fn apply_delete(&mut self, path: &[PathElement], old: Value,full_path:&[PathElement]) -> Result<(), ValueStoreError> {
        if path.is_empty() {
            *self = Default::default();
            Ok(())
        } else if let Some(parent) = self.get_mut(&path[..path.len() - 1]) {
            match (parent, &path[path.len() - 1]) {
                (Value::Map(map), PathElement::Field(name)) => match map.entry(name.clone()) {
                    std::collections::hash_map::Entry::Occupied(e) => {
                        if PartialEq::eq(&old, e.get()) {
                            e.remove();
                            Ok(())
                        } else {
                            Err(ValueStoreError::InvalidChange {
                                change: ChangeContent::Delete { path:full_path.to_vec(), old },
                            })
                        }
                    }
                    std::collections::hash_map::Entry::Vacant(_) => {
                        Err(ValueStoreError::InvalidChange {
                            change: ChangeContent::Delete { path:full_path.to_vec(), old },
                        })
                    }
                },
                (Value::Array(vec), PathElement::Index(index)) => {
                    if *index as usize >= vec.len() {
                        Err(ValueStoreError::InvalidChange {
                            change: ChangeContent::Delete { path:full_path.to_vec(), old },
                        })
                    } else if PartialEq::eq(&vec[*index as usize], &old) {
                        vec.remove(*index as usize);
                        Ok(())
                    } else {
                        Err(ValueStoreError::InvalidChange {
                            change: ChangeContent::Delete { path:full_path.to_vec(), old },
                        })
                    }
                }
                _ => Err(ValueStoreError::InvalidChange {
                    change: ChangeContent::Delete { path:full_path.to_vec(), old },
                }),
            }
        } else {
            Err(ValueStoreError::InvalidChange {
                change: ChangeContent::Delete { path:full_path.to_vec(), old },
            })
        }
    }

    pub fn apply_replace(
        &mut self,
        path: &[PathElement],
        old: Value,
        new: Value,
        full_path:&[PathElement]
    ) -> Result<(), ValueStoreError> {
        if let Some(val) = self.get_mut(&path) {
            if PartialEq::eq(&old, val) {
                *val = new;
                Ok(())
            } else {
                Err(ValueStoreError::InvalidChange {
                    change: ChangeContent::Replace { path:full_path.to_vec(), old, new },
                })
            }
        } else {
            Err(ValueStoreError::InvalidChange {
                change: ChangeContent::Replace { path:full_path.to_vec(), old, new },
            })
        }
    }

    pub fn apply_insert(
        &mut self,
        path: &[PathElement],
        value: Value,
        full_path:&[PathElement]
    ) -> Result<(), ValueStoreError> {
        if path.is_empty() {
            *self = value;
            Ok(())
        } else if let Some(parent) = self.get_mut(&path[..path.len() - 1]) {
            match (parent, &path[path.len() - 1]) {
                (Value::Map(map), PathElement::Field(name)) => match map.entry(name.clone()) {
                    std::collections::hash_map::Entry::Occupied(_) => {
                        Err(ValueStoreError::InvalidChange {
                            change: ChangeContent::Insert { path:full_path.to_vec(), value },
                        })
                    }
                    std::collections::hash_map::Entry::Vacant(e) => {
                        e.insert(value);
                        Ok(())
                    }
                },
                (Value::Array(vec), PathElement::Index(index)) => {
                    if *index as usize > vec.len() {
                        Err(ValueStoreError::InvalidChange {
                            change: ChangeContent::Insert { path:full_path.to_vec(), value },
                        })
                    } else {
                        vec.insert(*index as usize, value);
                        Ok(())
                    }
                }
                _ => Err(ValueStoreError::InvalidChange {
                    change: ChangeContent::Insert { path:full_path.to_vec(), value },
                }),
            }
        } else {
            Err(ValueStoreError::InvalidChange {
                change: ChangeContent::Insert { path:full_path.to_vec(), value },
            })
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Map(HashMap::new())
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(v1), Value::Integer(v2)) => v1 == v2,
            (Value::Float(v1), Value::Float(v2)) => {
                if v1.is_nan() && v2.is_nan() {
                    true
                } else {
                    v1 == v2
                }
            }
            (Value::Bool(v1), Value::Bool(v2)) => v1 == v2,
            (Value::String(v1), Value::String(v2)) => v1 == v2,
            (Value::Blob(v1), Value::Blob(v2)) => v1 == v2,
            (Value::Array(v1), Value::Array(v2)) => v1 == v2,
            (Value::Map(v1), Value::Map(v2)) => v1 == v2,
            _ => false,
        }
    }
}

impl Eq for Value {}
