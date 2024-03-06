use crate::{types::{Value, change::ChangeContent}, error::ValueStoreError};

pub mod simple;

pub trait ApplyChange {
    fn apply(&self,value:&mut Value)->Result<(),ValueStoreError>;
}

impl ApplyChange for ChangeContent {
    fn apply(&self,value:&mut Value)->Result<(),ValueStoreError> {
        simple::apply(value, self)
    }
}
