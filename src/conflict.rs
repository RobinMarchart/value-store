use std::collections::{BTreeMap, HashMap};

use crate::{
    error::ValueStoreError,
    types::{change::ChangeContent, PathElement, Value},
};
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
    pub fn construct<I: IntoIterator<Item = ChangeContent>>(
        iter: I,
    ) -> Result<Option<ChangeTree>, ValueStoreError> {
        let mut res = None;
        for change in iter {
            Self::add_change(&mut res, change)?
        }
        Ok(res)
    }

    fn add_change_insert(
        &mut self,
        path: Vec<PathElement>,
        value: Value,
        index: usize,
    ) -> Result<(), ValueStoreError> {
        if let Some(elem) = path.get(index) {
            match self {
                ChangeTree::Replace { old, new, changes } => {
                    new.apply_insert(&path[index + 1..], value.clone(), &path)?;
                    changes.push(ChangeContent::Insert { path, value });
                    Ok(())
                }
                ChangeTree::Remove { old, changes } => Err(ValueStoreError::InvalidChange {
                    change: ChangeContent::Insert { path, value },
                }),
                ChangeTree::Add { new, changes } => {
                    new.apply_insert(&path[index + 1..], value.clone(), &path)?;
                    changes.push(ChangeContent::Insert { path, value });
                    Ok(())
                }
                ChangeTree::Array(map) => {
                    if let PathElement::Index(i) = elem {
                        if let Some(new) = map.get_mut(i) {
                            new.add_change_insert(path, value, index + 1)
                        } else {
                            let after = map.split_off(i);
                            map.insert(*i, Self::from_insert(path, value, index + 1));
                            for (key, value) in after.into_iter() {
                                map.insert(key + 1, value);
                            }
                            Ok(())
                        }
                    } else {
                        Err(ValueStoreError::InvalidChange {
                            change: ChangeContent::Insert { path, value },
                        })
                    }
                }
                ChangeTree::Map(map) => {
                    if let PathElement::Field(name) = elem {
                        if let Some(new) = map.get_mut(name) {
                            new.add_change_insert(path, value, index + 1)
                        } else {
                            map.insert(name.clone(), Self::from_insert(path, value, index + 1));
                            Ok(())
                        }
                    } else {
                        Err(ValueStoreError::InvalidChange {
                            change: ChangeContent::Insert { path, value },
                        })
                    }
                }
            }
        } else {
            match self {
                ChangeTree::Remove { old, changes } => todo!(),
                _ => Err(ValueStoreError::InvalidChange {
                    change: ChangeContent::Insert { path, value },
                }),
            }
        }
    }

    fn from_insert(path: Vec<PathElement>, value: Value, index: usize) -> Self {
        match path.get(index) {
            Some(PathElement::Field(name)) => {
                let mut new = HashMap::new();
                new.insert(name.clone(), Self::from_insert(path, value, index + 1));
                Self::Map(new)
            }
            Some(PathElement::Index(i)) => {
                let mut new = BTreeMap::new();
                new.insert(*i, Self::from_insert(path, value, index + 1));
                Self::Array(new)
            }
            None => Self::Add {
                new: value.clone(),
                changes: vec![ChangeContent::Insert { path, value }],
            },
        }
    }

    fn from_replace(path: Vec<PathElement>, old: Value, new: Value, index: usize) -> Self {
        match path.get(index) {
            Some(PathElement::Field(name)) => {
                let mut m = HashMap::new();
                m.insert(name.clone(), Self::from_replace(path, old, new, index + 1));
                Self::Map(m)
            }
            Some(PathElement::Index(i)) => {
                let mut m = BTreeMap::new();
                m.insert(*i, Self::from_replace(path, old, new, index + 1));
                Self::Array(m)
            }
            None => Self::Replace {
                old: old.clone(),
                new: new.clone(),
                changes: vec![ChangeContent::Replace { path, old, new }],
            },
        }
    }

    fn add_change(
        this: &mut Option<ChangeTree>,
        change: ChangeContent,
    ) -> Result<(), ValueStoreError> {
        if let Some(this) = this.as_mut() {
            match change {
                ChangeContent::Insert { path, value } => this.add_change_insert(path, value, 0),
                ChangeContent::Replace { path, old, new } => todo!(),
                ChangeContent::Delete { path, old } => todo!(),
            }
        } else {
            *this = Some(match change {
                ChangeContent::Insert { path, value } => Self::from_insert(path, value, 0),
                ChangeContent::Replace { path, old, new } => Self::from_replace(path, old, new, 0),
                ChangeContent::Delete { path, old } => todo!(),
            });
            Ok(())
        }
    }
}
