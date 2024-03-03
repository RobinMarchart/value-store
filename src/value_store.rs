use uuid::Uuid;

use crate::{
    types::change::{Change, ChangeContent, Hash},
    Result,
};

struct ValueStore {}

#[derive(Debug)]
struct BranchId(pub Uuid);

#[derive(Debug)]
struct RepoId(pub Uuid);

impl ValueStore {
    pub async fn add_change(
        branch: BranchId,
        repo: RepoId,
        ignore_hook: Option<u64>,
        change: &Change,
    ) -> Result<()> {
        Ok(())
    }
    pub async fn add_chage_sets(
        branch: BranchId,
        repo: RepoId,
        ignore_hook: Option<u64>,
        changes: &[ChangeContent],
    ) -> Result<Hash> {
        todo!()
    }
}
