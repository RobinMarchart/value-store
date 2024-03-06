use std::sync::Arc;

use crate::{
    error::ValueStoreError,
    types::{change::ChangeContent, PathElement, Value},
};

pub fn apply_delete(
    this: &mut Value,
    path: &[PathElement],
    old: &Value,
    full_path: &[PathElement],
) -> Result<(), ValueStoreError> {
    if path.is_empty() {
        Err(ValueStoreError::InvalidChange {
            change: ChangeContent::Delete {
                path: full_path.to_vec(),
                old:old.clone(),
            },
        })
    } else if let Some(parent) = this.get_mut(&path[..path.len() - 1]) {
        match (parent, &path[path.len() - 1]) {
            (Value::Map(map), PathElement::Field(name)) => match Arc::make_mut(map).entry(name.clone()) {
                std::collections::hash_map::Entry::Occupied(e) => {
                    if PartialEq::eq(old, e.get()) {
                        e.remove();
                        Ok(())
                    } else {
                        Err(ValueStoreError::InvalidChange {
                            change: ChangeContent::Delete {
                                path: full_path.to_vec(),
                                old:old.clone(),
                            },
                        })
                    }
                }
                std::collections::hash_map::Entry::Vacant(_) => {
                    Err(ValueStoreError::InvalidChange {
                        change: ChangeContent::Delete {
                            path: full_path.to_vec(),
                            old:old.clone(),
                        },
                    })
                }
            },
            (Value::Array(vec), PathElement::Index(index)) => {
                if *index as usize >= vec.len() {
                    Err(ValueStoreError::InvalidChange {
                        change: ChangeContent::Delete {
                            path: full_path.to_vec(),
                            old:old.clone(),
                        },
                    })
                } else if PartialEq::eq(&vec[*index as usize], old) {
                    Arc::make_mut(vec).remove(*index as usize);
                    Ok(())
                } else {
                    Err(ValueStoreError::InvalidChange {
                        change: ChangeContent::Delete {
                            path: full_path.to_vec(),
                            old:old.clone(),
                        },
                    })
                }
            }
            _ => Err(ValueStoreError::InvalidChange {
                change: ChangeContent::Delete {
                    path: full_path.to_vec(),
                    old:old.clone(),
                },
            }),
        }
    } else {
        Err(ValueStoreError::InvalidChange {
            change: ChangeContent::Delete {
                path: full_path.to_vec(),
                old:old.clone(),
            },
        })
    }
}

pub fn apply_replace(
    this: &mut Value,
    path: &[PathElement],
    old: &Value,
    new: Value,
    full_path: &[PathElement],
) -> Result<(), ValueStoreError> {
    if let Some(val) = this.get_mut(path) {
        if PartialEq::eq(old, val) {
            *val = new;
            Ok(())
        } else {
            Err(ValueStoreError::InvalidChange {
                change: ChangeContent::Replace {
                    path: full_path.to_vec(),
                    old:old.clone(),
                    new,
                },
            })
        }
    } else {
        Err(ValueStoreError::InvalidChange {
            change: ChangeContent::Replace {
                path: full_path.to_vec(),
                old:old.clone(),
                new,
            },
        })
    }
}

pub fn apply_insert(
    this: &mut Value,
    path: &[PathElement],
    value: Value,
    full_path: &[PathElement],
) -> Result<(), ValueStoreError> {
    if path.is_empty() {
        Err(ValueStoreError::InvalidChange {
            change: ChangeContent::Insert {
                path: full_path.to_vec(),
                value,
            },
        })
    } else if let Some(parent) = this.get_mut(&path[..path.len() - 1]) {
        match (parent, &path[path.len() - 1]) {
            (Value::Map(map), PathElement::Field(name)) => match Arc::make_mut(map).entry(name.clone()) {
                std::collections::hash_map::Entry::Occupied(_) => {
                    Err(ValueStoreError::InvalidChange {
                        change: ChangeContent::Insert {
                            path: full_path.to_vec(),
                            value,
                        },
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
                        change: ChangeContent::Insert {
                            path: full_path.to_vec(),
                            value,
                        },
                    })
                } else {
                    Arc::make_mut(vec).insert(*index as usize, value);
                    Ok(())
                }
            }
            _ => Err(ValueStoreError::InvalidChange {
                change: ChangeContent::Insert {
                    path: full_path.to_vec(),
                    value,
                },
            }),
        }
    } else {
        Err(ValueStoreError::InvalidChange {
            change: ChangeContent::Insert {
                path: full_path.to_vec(),
                value,
            },
        })
    }
}
pub fn apply(this: &mut Value, change: &ChangeContent) -> Result<(), ValueStoreError> {
    match change {
        ChangeContent::Insert { path, value } => apply_insert(this, path, value.clone(), path)?,
        ChangeContent::Replace { path, old, new } => apply_replace(this, path, old, new.clone(), path)?,
        ChangeContent::Delete { path, old } => apply_delete(this, path, old, path)?,
    }
    Ok(())
}
