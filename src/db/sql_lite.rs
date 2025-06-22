use std::fs;
use std::path::Path;

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite, query};
use tracing::trace;

use crate::db::Comment;
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
            Self::create_comments_table(&pool).await?;
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
                comments_count REAL DEFAULT 0
            );",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn create_comments_table(pool: &Pool<Sqlite>) -> Result<(), Error> {
        query(
            "CREATE TABLE comments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pin_id INTEGER NOT NULL,
                date TEXT NOT NULL,
                author TEXT NOT NULL,
                content TEXT NOT NULL,
                FOREIGN KEY (pin_id) REFERENCES pins (id) ON DELETE CASCADE
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
            INSERT INTO pins (type, title, description, x, y, comments_count)
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
        SELECT id, type, title, description, x, y, comments_count
        FROM pins
    "#;
        let pins = sqlx::query_as::<_, Pin>(query)
            .fetch_all(&self.pool)
            .await?;
        Ok(pins)
    }

    async fn get_pin_by_id(&self, id: i32) -> Result<super::Pin, Error> {
        let query = r#"
        SELECT id, type, title, description, x, y, comments_count
        FROM pins
        WHERE id = ?
    "#;
        let pin = sqlx::query_as::<_, Pin>(query)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(pin)
    }

    async fn delete_pin(&self, id: i32) -> Result<(), Error> {
        let query = r#"
        DELETE FROM pins
        WHERE id = ?
    "#;
        sqlx::query(query).bind(id).execute(&self.pool).await?;
        Ok(())
    }

    async fn insert_comment(
        &self,
        pin_id: i32,
        date: String,
        author: String,
        content: String,
    ) -> Result<(), Error> {
        let query = r#"
            INSERT INTO comments (pin_id, date, author, content)
            VALUES (?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(pin_id)
            .bind(date)
            .bind(author)
            .bind(content)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_pin_comments(&self, pin_id: i32) -> Result<Vec<Comment>, Error> {
        let query = r#"
            SELECT id, pin_id, date, author, content
            FROM comments
            WHERE pin_id = ?
            ORDER BY id ASC
        "#;

        let comments = sqlx::query_as::<_, Comment>(query)
            .bind(pin_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(comments)
    }
}
