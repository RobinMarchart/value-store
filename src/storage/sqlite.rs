use futures_util::TryStreamExt;
use sqlx::SqlitePool;

use crate::{storage::Storage, types::change::Hash, Result};

pub struct SqliteStorage {
    inner: SqlitePool,
}

impl SqliteStorage {
    pub async fn connect(url: &str) -> Result<Self> {
        Self::new(SqlitePool::connect(url).await?).await
    }
    pub async fn new(pool: SqlitePool) -> Result<Self> {
        sqlx::migrate!("migrations/sqlite").run(&pool).await?;
        Ok(Self { inner: pool })
    }
}

pub struct ChangeId(i64);
pub struct BranchId(i64);
pub struct RepoId(i64);

impl Storage for SqliteStorage {
    type ChangeId = ChangeId;
    type BranchId = BranchId;
    type RepoId = RepoId;

    async fn add_change(
        &self,
        hash: &Hash,
        content: &[u8],
        parents: &[Hash],
    ) -> Result<Self::ChangeId> {
        let mut trans = self.inner.begin().await?;
        let hash = hash.as_slice();
        let id = if let Some(Some(id)) = sqlx::query_scalar!(
            "INSERT OR IGNORE INTO changes (hash, content) VALUES (?, ?) RETURNING id",
            hash,
            content
        )
        .fetch_optional(trans.as_mut())
        .await?
        {
            for parent in parents {
                let parent = parent.as_slice();
                let parent = sqlx::query_scalar!("SELECT id FROM changes WHERE hash==?", parent)
                    .fetch_one(trans.as_mut())
                    .await?;
                sqlx::query!(
                    "INSERT INTO change_rels (parent,child) VALUES (?,?)",
                    parent,
                    id
                )
                .execute(trans.as_mut())
                .await?;
            }
            id
        } else {
            sqlx::query_scalar!("SELECT id FROM changes WHERE hash==?", hash)
                .fetch_one(trans.as_mut())
                .await?
        };
        Ok(ChangeId(id))
    }

    async fn get_change_id(&self, hash: Hash) -> Result<Option<Self::ChangeId>> {
        let hash = hash.as_slice();
        Ok(
            sqlx::query_scalar!("SELECT id FROM changes WHERE hash==?", hash)
                .fetch_optional(&self.inner)
                .await?
                .map(ChangeId),
        )
    }

    async fn get_change_rels(&self, id: Self::ChangeId) -> Result<Vec<Self::ChangeId>> {
        Ok(
                sqlx::query_scalar!(
                    "SELECT change_rels.parent FROM change_rels JOIN changes ON change_rels.parent == changes.id WHERE change_rels.child == ? ORDER BY changes.hash ASC",
                    id.0
                ).fetch(&self.inner)
                .map_ok(ChangeId)
                .try_collect().await?
            )
    }

    async fn get_change_content(&self, id: Self::ChangeId) -> Result<Vec<u8>> {
        Ok(
            sqlx::query_scalar!("SELECT content FROM changes WHERE id == ?", id.0)
                .fetch_one(&self.inner)
                .await?,
        )
    }
}
