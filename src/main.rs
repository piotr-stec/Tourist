pub mod db;
pub mod errors;
pub mod server;

use crate::db::sql_lite::SqliteDb;
use axum::Server;
use server::{create_router, AppState};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{trace, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    // init db
    let db = SqliteDb::new("tourist.db").await?;
    let state = AppState { db: Arc::new(db) };

    let app = create_router(state);
    
    // Start the server
    let address = SocketAddr::from(([0, 0, 0, 0], 3000));
    trace!("Listening on {}", address);

    Server::bind(&address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
