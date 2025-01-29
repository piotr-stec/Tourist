use std::fs;
use std::path::Path;

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{query, Pool, Sqlite};
use tracing::trace;

use crate::errors::Error;

use super::{Pin, TouristDb};

#[derive(Clone)]
pub struct SqliteDb {
    pub(crate) pool: Pool<Sqlite>,
}

impl SqliteDb {
    pub async fn new(path: &str) -> Result<Self, Error> {
        // Check if there is a database file at the path
        if !Path::new(path).try_exists()? {
            trace!(
                "Database file not found. A new one will be created at: {}",
                path
            );
            fs::File::create(path)?;
        } else {
            trace!("Database file found at: {}", path);
        }

        let pool = SqlitePoolOptions::new()
            .connect(&format!("sqlite:{}", path))
            .await?;

        let table_exists = Self::check_table_exists(&pool).await?;

        if !table_exists {
            Self::create_pins_table(&pool).await?;
            Self::create_rates_table(&pool).await?;
        } else {
            trace!("Table 'pins' with correct structure found.");
        }
        Ok(Self { pool })
    }

    pub async fn create_pins_table(pool: &Pool<Sqlite>) -> Result<(), Error> {
        query(
            "CREATE TABLE pins (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                type TEXT NOT NULL,
                title TEXT NOT NULL CHECK(LENGTH(title) <= 32),
                description TEXT NOT NULL,
                x REAL NOT NULL CHECK(x BETWEEN -180.0 AND 180.0),
                y REAL NOT NULL CHECK(y BETWEEN -90.0 AND 90.0),
                average_rate REAL DEFAULT 0
            );",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn create_rates_table(pool: &Pool<Sqlite>) -> Result<(), Error> {
        query(
            "CREATE TABLE rates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                point_id INTEGER NOT NULL,
                rate INTEGER NOT NULL CHECK(rate BETWEEN 1 AND 5),
                FOREIGN KEY (point_id) REFERENCES pins (id) ON DELETE CASCADE
            );",
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

impl TouristDb for SqliteDb {
    async fn insert_pin(
        &self,
        pin_type: String,
        title: String,
        description: String,
        x: f64,
        y: f64,
    ) -> Result<(), Error> {
        let query = r#"
            INSERT INTO pins (type, title, description, x, y, average_rate)
            VALUES (?, ?, ?, ?, ?, 0)
        "#;
        sqlx::query(query)
            .bind(pin_type)
            .bind(title)
            .bind(description)
            .bind(x)
            .bind(y)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_all_pins(&self) -> Result<Vec<super::Pin>, Error> {
        let query = r#"
        SELECT id, type, title, description, x, y, average_rate
        FROM pins
    "#;
        let pins = sqlx::query_as::<_, Pin>(query)
            .fetch_all(&self.pool)
            .await?;
        Ok(pins)
    }

    async fn get_pin_by_id(&self, id: i32) -> Result<super::Pin, Error> {
        let query = r#"
        SELECT id, type, title, description, x, y, average_rate
        FROM pins
        WHERE id = ?
    "#;
        let pin = sqlx::query_as::<_, Pin>(query)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(pin)
    }

    async fn insert_rating(&self, point_id: i32, rate: i32) -> Result<(), Error> {
        let query = r#"
        INSERT INTO rates (point_id, rate)
        VALUES (?, ?)
    "#;
        sqlx::query(query)
            .bind(point_id)
            .bind(rate)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_average_rating(&self, point_id: i32) -> Result<(), Error> {
        let query = r#"
        SELECT AVG(rate) as average_rate
        FROM rates
        WHERE point_id = ?
    "#;
        let average_rate: Option<f64> = sqlx::query_scalar(query)
            .bind(point_id)
            .fetch_one(&self.pool)
            .await?;

        if let Some(avg_rate) = average_rate {
            let update_query = r#"
            UPDATE pins
            SET average_rate = ?
            WHERE id = ?
        "#;
            sqlx::query(update_query)
                .bind(avg_rate)
                .bind(point_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    async fn delete_pin(&self, id: i32) -> Result<(), Error> {
        let query = r#"
        DELETE FROM pins
        WHERE id = ?
    "#;
        sqlx::query(query).bind(id).execute(&self.pool).await?;
        Ok(())
    }
}
