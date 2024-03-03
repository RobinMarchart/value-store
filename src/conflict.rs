use std::collections::{BTreeMap, HashMap};

use crate::types::{change::ChangeContent, PathElement, Value};
pub struct ActiveConflict {
    pub common_value: Value,
    pub conflicts: [ChangeTree; 2],
    pub common_changes: [Vec<ChangeContent>; 2],
}
pub struct ResolvedConflict {
    pub value: Value,
    pub changes: [Vec<ChangeContent>; 2],
}

pub enum Conflict {
    Active(ActiveConflict),
    Resolved(ResolvedConflict),
}

pub fn check_conflicts_common_ancestor<
    I1: IntoIterator<Item = ChangeContent>,
    I2: IntoIterator<Item = ChangeContent>,
>(
    ancestor: Value,
    change1: I1,
    change2: I2,
) -> Option<Conflict> {
    None
}

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
    Array(BTreeMap<u32, ChangeTree>),
    Map(HashMap<String, ChangeTree>),
}

impl ChangeTree {
    pub fn construct<I: IntoIterator<Item = ChangeContent>>(iter: I) -> Option<ChangeTree> {
        None
    }
    pub fn add_change(this: &mut Option<ChangeTree>, change: ChangeContent) {
        if let Some(this) = this {
            match change {
                ChangeContent::Insert { path, value } => {}
                ChangeContent::Replace { path, old, new } => todo!(),
                ChangeContent::Delete { path, old } => todo!(),
            }
        } else {
            let (mut val, path) = match change {
                ChangeContent::Insert { path, value } => {
                    let change = ChangeContent::Insert {
                        path: path.clone(),
                        value: value.clone(),
                    };
                    (
                        ChangeTree::Add {
                            new: value,
                            changes: vec![change],
                        },
                        path,
                    )
                }
                ChangeContent::Replace { path, old, new } => {
                    let change = ChangeContent::Replace {
                        path: path.clone(),
                        old: old.clone(),
                        new: new.clone(),
                    };
                    (
                        ChangeTree::Replace {
                            old,
                            new,
                            changes: vec![change],
                        },
                        path,
                    )
                }
                ChangeContent::Delete { path, old } => {
                    let change = ChangeContent::Delete {
                        path: path.clone(),
                        old: old.clone(),
                    };
                    (
                        ChangeTree::Remove {
                            old,
                            changes: vec![change],
                        },
                        path,
                    )
                }
            };
            for path in path.into_iter().rev() {
                match path {
                    PathElement::Field(name) => {
                        let mut new = HashMap::new();
                        new.insert(name, val);
                        val = ChangeTree::Map(new);
                    }
                    PathElement::Index(index) => {
                        let mut new = BTreeMap::new();
                        new.insert(index, val);
                        val = ChangeTree::Array(new);
                    }
                }
            }
        }
    }
}
