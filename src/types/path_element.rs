use std::fmt::Debug;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Clone)]
pub enum PathElement {
    Field(String),
    Index(u32),
}

pub enum PathElementRef<'s>{
    Field(&'s str),
    Index(u32)
}

impl PathElement{
    pub fn as_ref(&self)->PathElementRef<'_>{
        match self{
            PathElement::Field(name) => PathElementRef::Field(name),
            PathElement::Index(index) => PathElementRef::Index(*index),
        }
    }
}

impl<'a> PathElementRef<'a> {
    pub fn to_owned(&self)->PathElement{
        match self{
            PathElementRef::Field(name) => PathElement::Field((*name).to_owned()),
            PathElementRef::Index(index) => PathElement::Index(*index),
        }
    }
}

struct PathElementVisitor {}

impl serde::de::Visitor<'_> for PathElementVisitor {
    type Value = PathElement;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("either a string or u32")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PathElement::Index(u32::try_from(v).map_err(|_| {
            serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
        })?))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PathElement::Index(v))
    }
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PathElement::Index(u32::try_from(v).map_err(|_| {
            serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
        })?))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PathElement::Field(v.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PathElement::Field(v))
    }
}

impl<'de> Deserialize<'de> for PathElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(PathElementVisitor {})
    }
}
impl Serialize for PathElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            PathElement::Field(name) => Serialize::serialize(name, serializer),
            PathElement::Index(index) => Serialize::serialize(index, serializer),
        }
    }
}

impl Debug for PathElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathElement::Field(name) => Debug::fmt(name, f),
            PathElement::Index(index) => Debug::fmt(index, f),
        }
    }
}

#[cfg(test)]
mod test {
    use serde_test::{assert_tokens, Token, assert_de_tokens};
    use ciborium::{into_writer,from_reader};

    use super::PathElement;

    #[test]
    fn index_ser_de() {
        assert_tokens(&PathElement::Index(1337), &[Token::U32(1337)]);
        assert_tokens(&PathElement::Index(0), &[Token::U32(0)]);
        assert_de_tokens(&PathElement::Index(1337), &[Token::U64(1337)]);
        assert_de_tokens(&PathElement::Index(1337), &[Token::U16(1337)]);
        assert_de_tokens(&PathElement::Index(13), &[Token::U8(13)]);
        assert_de_tokens(&PathElement::Index(3), &[Token::I64(3)]);
        assert_de_tokens(&PathElement::Index(3), &[Token::I32(3)]);
        assert_de_tokens(&PathElement::Index(3), &[Token::I16(3)]);
        assert_de_tokens(&PathElement::Index(3), &[Token::I8(3)]);
    }

    #[test]
    fn field_ser_de() {
        assert_tokens(
            &PathElement::Field("name".to_string()),
            &[Token::Str("name")],
        );
        assert_de_tokens(&PathElement::Field("name".to_string()), &[Token::String("name")]);
        assert_de_tokens(&PathElement::Field("name".to_string()), &[Token::BorrowedStr("name")]);
    }

    #[test]
    fn cbor_round_trip(){
        let val = [PathElement::Field("test".to_string()),PathElement::Index(1337),PathElement::Field("value".to_string()),PathElement::Index(0)];
        let mut serialized=Vec::new();
        into_writer(&val, &mut serialized).expect("serializing failed");
        let res:Vec<PathElement>=from_reader(serialized.as_slice()).expect("de-serializing failed");
        assert_eq!(val.as_slice(),res.as_slice(),"value differs after round trip")
    }

}
