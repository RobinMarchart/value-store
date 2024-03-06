use std::collections::{BTreeMap, HashMap};

use super::{Value, change::ChangeContent};

#[derive(Debug)]
pub enum ChangeTree {
    Replace {
        old: Value,
        new: Value,
        changes: Vec<ChangeContent>,
    },
    Remove {
        old: Value,
        changes: Vec<ChangeContent>,
    },
    Add {
        new: Value,
        changes: Vec<ChangeContent>,
    },
    Array{data:BTreeMap<u32, ChangeTree>,offsets:BTreeMap<u32,u32>},
    Map(HashMap<String, ChangeTree>),
}
