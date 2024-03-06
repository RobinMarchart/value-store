/**
     *  applies change tree to value
     *  path is the path of the current change tree
     *  */
    fn apply_tree_inner(
        this:&mut Value,
        changes: ChangeTree,
        path: StackList<'_, PathElementRef<'_>>,
    ) -> Result<(), ValueStoreError> {
        match changes {
            ChangeTree::Replace { old, new, changes } => {
                if PartialEq::eq(&old, this) {
                    *self = new;
                    Ok(())
                } else {
                    Err(ValueStoreError::InvalidTreeChange {
                        change: ChangeTree::Replace { old, new, changes },
                        path: path.to_vec_mapped(PathElementRef::to_owned),
                    })
                }
            }
            ChangeTree::Remove { old, changes } => {
                if path.is_empty() {
                    Err(ValueStoreError::InvalidTreeChange {
                        change: ChangeTree::Remove { old, changes },
                        path: Vec::with_capacity(0),
                    })
                } else {
                    unreachable!()
                }
            }
            ChangeTree::Add { new, changes } => {
                if path.is_empty() {
                    Err(ValueStoreError::InvalidTreeChange {
                        change: ChangeTree::Add { new, changes },
                        path: Vec::with_capacity(0),
                    })
                } else {
                    unreachable!()
                }
            }
            ChangeTree::Array(map) => {
                if let Value::Array(array) = self {
                    for (index, child) in map.iter() {
                        if let Some(val) = array.get_mut(*index as usize) {}
                    }
                    Ok(())
                } else {
                    Err(ValueStoreError::InvalidTreeChange {
                        change: ChangeTree::Array(map),
                        path: path.to_vec_mapped(PathElementRef::to_owned),
                    })
                }
            }
            ChangeTree::Map(_) => todo!(),
        }
    }
