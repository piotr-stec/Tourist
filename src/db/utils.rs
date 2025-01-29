use sqlx::{Pool, Sqlite};

use super::sql_lite::SqliteDb;
use crate::errors::Error;

impl SqliteDb {
    pub(crate) async fn check_table_exists(pool: &Pool<Sqlite>) -> Result<bool, Error> {
        let blocks_exist =
            sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='pins';")
                .fetch_optional(pool)
                .await?
                .is_some();
        let proofs_exist =
            sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='rates';")
                .fetch_optional(pool)
                .await?
                .is_some();
        Ok(blocks_exist && proofs_exist)
    }
}
