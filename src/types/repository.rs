use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repository {
    pub id: Uuid,
    pub descr: String,
}
