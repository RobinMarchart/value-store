use std::{collections::HashMap, fmt::Debug, sync::Arc};

use serde::{
    de::{self, Visitor},
    ser, Deserialize, Serialize,
};

use crate::{apply::ApplyChange, error::ValueStoreError};

use super::PathElement;

pub struct Blob {
    pub mime: String,
    pub data: Vec<u8>,
}

#[derive(Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(Arc<String>),
    Blob(Arc<Blob>),
    Array(Arc<Vec<Value>>),
    Map(Arc<HashMap<String, Value>>),
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(v) => Debug::fmt(v, f),
            Value::Float(v) => Debug::fmt(v, f),
            Value::Bool(v) => Debug::fmt(v, f),
            Value::String(v) => Debug::fmt(v, f),
            Value::Array(v) => Debug::fmt(v.as_slice(), f),
            Value::Map(v) => Debug::fmt(v, f),
            Value::Blob(blob) => write!(f, "Blob of type {}", blob.mime),
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Integer(v) => serializer.serialize_i64(*v),
            Value::Float(v) => serializer.serialize_f64(*v),
            Value::Bool(v) => serializer.serialize_bool(*v),
            Value::String(v) => serializer.serialize_str(v),
            Value::Array(v) => Serialize::serialize(v, serializer),
            Value::Map(v) => Serialize::serialize(v, serializer),
            Value::Blob(blob) => {
                if blob.mime.len() > u8::MAX as usize {
                    Err(<S::Error as ser::Error>::custom(
                        "mime type should have a max len of 255",
                    ))
                } else {
                    let mut buf = Vec::with_capacity(1 + blob.mime.len() + blob.data.len());
                    buf.push(blob.mime.len() as u8);
                    buf.extend_from_slice(blob.mime.as_bytes());
                    buf.extend_from_slice(&blob.data);
                    serializer.serialize_bytes(&buf)
                }
            }
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
        Ok(Value::String(v.to_string().into()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::String(v.into()))
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
        Ok(Value::Array(res.into()))
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
        Ok(Value::Map(res.into()))
    }
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let str_len = *v
            .first()
            .ok_or_else(|| <E as de::Error>::invalid_length(0, &"at least 1"))?
            as usize;
        let mime = std::str::from_utf8(v.get(1..str_len + 1).ok_or_else(|| {
            <E as de::Error>::invalid_length(
                v.len(),
                &"the mime type with the length indicated in the first byte",
            )
        })?)
        .map_err(|_| {
            <E as de::Error>::invalid_value(
                de::Unexpected::Other("non utf-8 value"),
                &"mime type in utf-8 encoding",
            )
        })?
        .to_string();
        Ok(Value::Blob(
            Blob {
                mime,
                data: v[str_len + 1..].to_vec(),
            }
            .into(),
        ))
    }
    fn visit_byte_buf<E>(self, mut v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let str_len = *v
            .first()
            .ok_or_else(|| <E as de::Error>::invalid_length(0, &"at least 1"))?
            as usize;
        let mime = std::str::from_utf8(v.get(1..str_len + 1).ok_or_else(|| {
            <E as de::Error>::invalid_length(
                v.len(),
                &"the mime type with the length indicated in the first byte",
            )
        })?)
        .map_err(|_| {
            <E as de::Error>::invalid_value(
                de::Unexpected::Other("non utf-8 value"),
                &"mime type in utf-8 encoding",
            )
        })?
        .to_string();
        v.copy_within(str_len + 1.., 0);
        v.truncate(v.len() - str_len - 1);
        Ok(Value::Blob(Blob { mime, data: v }.into()))
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
                    if let Some(entry) = Arc::make_mut(map).get_mut(name) {
                        entry.get_mut(next)
                    } else {
                        None
                    }
                }
                (PathElement::Index(index), Value::Array(arr)) => {
                    if let Some(entry) = Arc::make_mut(arr).get_mut(*index as usize) {
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
    pub fn apply_iter<'l, I: IntoIterator<Item = &'l C>, C: ApplyChange + 'l>(
        &'l mut self,
        i: I,
    ) -> Result<(), ValueStoreError> {
        for change in i {
            self.apply(change)?;
        }
        Ok(())
    }

    pub fn apply<C: ApplyChange>(&mut self, change: &C) -> Result<(), ValueStoreError> {
        change.apply(self)
    }
}
impl Default for Value {
    fn default() -> Self {
        Value::Map(HashMap::new().into())
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
            (Value::Array(v1), Value::Array(v2)) => v1 == v2,
            (Value::Map(v1), Value::Map(v2)) => v1 == v2,
            (Value::Blob(v1), Value::Blob(v2)) => v1.mime == v2.mime && v1.data == v2.data,
            _ => false,
        }
    }
}

impl Eq for Value {}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use ciborium::{from_reader, into_writer};
    use serde_test::{assert_de_tokens, assert_tokens, Token};

    use super::{Blob, Value};

    #[test]
    fn value_eq_float() {
        assert_eq!(Value::Float(0.0), Value::Float(-0.0));
        assert_eq!(Value::Float(f64::NAN), Value::Float(f64::NAN));
        let n1 = f64::from_bits(u64::MAX);
        assert!(n1.is_nan());
        let n2 = f64::from_bits(u64::MAX - 1);
        assert!(n2.is_nan());
        assert_eq!(Value::Float(n1), Value::Float(n2))
    }

    #[test]
    fn value_int_ser_de() {
        assert_tokens(&Value::Integer(4), &[Token::I64(4)]);
        assert_de_tokens(&Value::Integer(4), &[Token::U64(4)]);
    }

    #[test]
    fn value_float_ser_de() {
        assert_tokens(&Value::Float(3.14), &[Token::F64(3.14)]);
    }

    #[test]
    fn value_bool_ser_de() {
        assert_tokens(&Value::Bool(false), &[Token::Bool(false)]);
    }

    #[test]
    fn value_str_ser_de() {
        assert_tokens(
            &Value::String("test".to_string().into()),
            &[Token::Str("test")],
        );
        assert_de_tokens(
            &Value::String("t".to_string().into()),
            &[Token::String("t")],
        );
        assert_de_tokens(
            &Value::String("t".to_string().into()),
            &[Token::BorrowedStr("t")],
        );
    }

    #[test]
    fn value_arr_ser_de() {
        assert_tokens(
            &Value::Array(vec![].into()),
            &[Token::Seq { len: Some(0) }, Token::SeqEnd],
        );
        assert_de_tokens(
            &Value::Array(vec![].into()),
            &[Token::Seq { len: None }, Token::SeqEnd],
        );
        assert_tokens(
            &Value::Array(vec![Value::Bool(true)].into()),
            &[
                Token::Seq { len: Some(1) },
                Token::Bool(true),
                Token::SeqEnd,
            ],
        );
        assert_de_tokens(
            &Value::Array(vec![Value::Bool(true)].into()),
            &[Token::Seq { len: None }, Token::Bool(true), Token::SeqEnd],
        );
    }

    #[test]
    fn value_map_ser_de() {
        assert_tokens(
            &Value::Map(HashMap::from_iter([]).into()),
            &[Token::Map { len: Some(0) }, Token::MapEnd],
        );
        assert_de_tokens(
            &Value::Map(HashMap::from_iter([]).into()),
            &[Token::Map { len: None }, Token::MapEnd],
        );
        assert_tokens(
            &Value::Map(HashMap::from_iter([("test".to_string(), Value::Integer(5))]).into()),
            &[
                Token::Map { len: Some(1) },
                Token::Str("test"),
                Token::I64(5),
                Token::MapEnd,
            ],
        );
        assert_de_tokens(
            &Value::Map(HashMap::from_iter([("test".to_string(), Value::Integer(5))]).into()),
            &[
                Token::Map { len: None },
                Token::Str("test"),
                Token::I64(5),
                Token::MapEnd,
            ],
        );
    }

    #[test]
    fn value_blob_ser_de() {
        assert_tokens(
            &Value::Blob(
                Blob {
                    mime: "abcd".to_string(),
                    data: b"efgh".to_vec(),
                }
                .into(),
            ),
            &[Token::Bytes(b"\x04abcdefgh")],
        );
        assert_de_tokens(
            &Value::Blob(
                Blob {
                    mime: "abcd".to_string(),
                    data: b"efgh".to_vec(),
                }
                .into(),
            ),
            &[Token::ByteBuf(b"\x04abcdefgh")],
        );
        assert_de_tokens(
            &Value::Blob(
                Blob {
                    mime: "abcd".to_string(),
                    data: b"efgh".to_vec(),
                }
                .into(),
            ),
            &[Token::BorrowedBytes(b"\x04abcdefgh")],
        );
    }

    #[test]
    fn value_cbor_ser_de() {
        let val = Value::Map(
            HashMap::from_iter([
                ("int".to_string(), Value::Integer(4)),
                ("float".to_string(), Value::Float(0.125)),
                ("bool".to_string(), Value::Bool(false)),
                (
                    "str".to_string(),
                    Value::String("string".to_string().into()),
                ),
                (
                    "arr".to_string(),
                    Value::Array(vec![Value::Integer(1)].into()),
                ),
                (
                    "blob".to_string(),
                    Value::Blob(
                        Blob {
                            mime: "test".to_string(),
                            data: b"hello there".to_vec(),
                        }
                        .into(),
                    ),
                ),
            ])
            .into(),
        );
        let mut serialized = Vec::new();
        into_writer(&val, &mut serialized).expect("serializing failed");
        let res: Value = from_reader(serialized.as_slice()).expect("de-serializing failed");
        assert_eq!(
            val,
            res,
            "value differs after round trip"
        )
    }
}
