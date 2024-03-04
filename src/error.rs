use std::fmt::Display;

use crate::types::change::{format_hash_lower, Hash, ChangeContent};

#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "db_sqlx")]
    Sqlx(sqlx::Error),
    Migrate(sqlx::migrate::MigrateError),
    CborDe(ciborium::de::Error<std::io::Error>),
    CborSer(ciborium::ser::Error<std::io::Error>),
    ValueStore(ValueStoreError),
    NoOP,
}

#[derive(Debug)]
pub enum ValueStoreError {
    HeadParentMismatch { parent: Hash },
    ParentHashSame,
    InvalidChange{change: ChangeContent},
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "db_sqlx")]
            Error::Sqlx(e) => Display::fmt(e, f),
            #[cfg(feature = "db_sqlx")]
            Error::Migrate(e) => Display::fmt(e, f),
            Error::NoOP => panic!("no op error actually constructed"),
            Error::CborDe(e) => Display::fmt(e, f),
            Error::CborSer(e) => Display::fmt(e, f),
            Error::ValueStore(e) => Display::fmt(e, f),
        }
    }
}

impl std::error::Error for Error {}

impl Display for ValueStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueStoreError::HeadParentMismatch { parent } => {
                f.write_str("Head not one of the parents of the change. Head hash: ")?;
                format_hash_lower(parent, f)
            }
            ValueStoreError::ParentHashSame => {
                f.write_str("Tried to construct Parents with two times the same parent")
            }
            ValueStoreError::InvalidChange { change } => {
                write!(f,"invalid change: {change:x?}")
            }
        }
    }
}

impl std::error::Error for ValueStoreError {}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "db_sqlx")]
impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(value)
    }
}
#[cfg(feature = "db_sqlx")]
impl From<sqlx::migrate::MigrateError> for Error {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        Self::Migrate(value)
    }
}
impl From<ciborium::de::Error<std::io::Error>> for Error {
    fn from(value: ciborium::de::Error<std::io::Error>) -> Self {
        Self::CborDe(value)
    }
}
impl From<ciborium::ser::Error<std::io::Error>> for Error {
    fn from(value: ciborium::ser::Error<std::io::Error>) -> Self {
        Self::CborSer(value)
    }
}

impl From<ValueStoreError> for Error {
    fn from(value: ValueStoreError) -> Self {
        Self::ValueStore(value)
    }
}
