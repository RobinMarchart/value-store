use std::future::Future;

use crate::{types::change::Hash, Result};

pub trait Storage {
    type ChangeId;
    type BranchId;
    type RepoId;
    fn add_change(
        &self,
        hash: &Hash,
        content: &[u8],
        parents: &[Hash],
    ) -> impl Future<Output = Result<Self::ChangeId>> + Send;
    fn get_change_id(
        &self,
        hash: Hash,
    ) -> impl Future<Output = Result<Option<Self::ChangeId>>> + Send;
    fn get_change_rels(
        &self,
        id: Self::ChangeId,
    ) -> impl Future<Output = Result<Vec<Self::ChangeId>>> + Send;
    fn get_change_content(
        &self,
        id: Self::ChangeId,
    ) -> impl Future<Output = Result<Vec<u8>>> + Send;
}

#[cfg(feature = "db_sqlite")]
pub mod sqlite;
