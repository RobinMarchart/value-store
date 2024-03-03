use std::future::Future;

use futures_util::TryStreamExt;
use sqlx::SqlitePool;

use crate::{types::change::Hash, Result};

use super::Storage;

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

    fn add_change(
        &self,
        hash: &Hash,
        content: &[u8],
        parents: &[Hash],
    ) -> impl Future<Output = Result<Self::ChangeId>> + Send {
        async move {
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
                    let parent =
                        sqlx::query_scalar!("SELECT id FROM changes WHERE hash==?", parent)
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
    }

    fn get_change_id(
        &self,
        hash: Hash,
    ) -> impl Future<Output = Result<Option<Self::ChangeId>>> + Send {
        async move {
            let hash = hash.as_slice();
            Ok(
                sqlx::query_scalar!("SELECT id FROM changes WHERE hash==?", hash)
                    .fetch_optional(&self.inner)
                    .await?
                    .map(|id| ChangeId(id)),
            )
        }
    }

    fn get_change_rels(
        &self,
        id: Self::ChangeId,
    ) -> impl Future<Output = Result<Vec<Self::ChangeId>>> + Send {
        async move {
            Ok(
                sqlx::query_scalar!(
                    "SELECT change_rels.parent FROM change_rels JOIN changes ON change_rels.parent == changes.id WHERE change_rels.child == ? ORDER BY changes.hash ASC",
                    id.0
                ).fetch(&self.inner)
                .map_ok(|id|ChangeId(id))
                .try_collect().await?
            )
        }
    }

    fn get_change_content(
        &self,
        id: Self::ChangeId,
    ) -> impl Future<Output = Result<Vec<u8>>> + Send {
        async move {
            Ok(
                sqlx::query_scalar!("SELECT content FROM changes WHERE id == ?", id.0)
                    .fetch_one(&self.inner)
                    .await?,
            )
        }
    }
}
