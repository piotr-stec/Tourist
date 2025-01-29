pub mod sql_lite;
pub mod utils;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::errors::Error;

#[allow(async_fn_in_trait)]
pub trait TouristDb {
    /// Inserts a new pin into the database
    async fn insert_pin(
        &self,
        pin_type: String,
        title: String,
        description: String,
        x: f64,
        y: f64,
    ) -> Result<(), Error>;

    /// Retrieves all pins from the database
    async fn get_all_pins(&self) -> Result<Vec<Pin>, Error>;

    /// Retrieves a single pin by ID
    async fn get_pin_by_id(&self, id: i32) -> Result<Pin, Error>;

    /// Inserts a new rating for a pin
    async fn insert_rating(&self, point_id: i32, rate: i32) -> Result<(), Error>;

    /// Updates the average rating of a pin
    async fn update_average_rating(&self, point_id: i32) -> Result<(), Error>;

    /// Deletes a pin by ID (optional, in case you need deletion functionality)
    async fn delete_pin(&self, id: i32) -> Result<(), Error>;
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Pin {
    pub id: i32,
    pub r#type: String,
    pub title: String,
    pub description: String,
    pub x: f64,
    pub y: f64,
    pub average_rate: f64,
}
