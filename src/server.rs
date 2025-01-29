use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::sql_lite::SqliteDb;
use crate::db::TouristDb;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<SqliteDb>,
}

#[derive(Deserialize)]
pub struct AddPinRequest {
    pub r#type: String,
    pub title: String,
    pub description: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Deserialize)]
pub struct AddRatingRequest {
    pub point_id: i32,
    pub rate: i32,
}

#[derive(Serialize)]
pub struct PinResponse {
    pub id: i32,
    pub r#type: String,
    pub title: String,
    pub description: String,
    pub x: f64,
    pub y: f64,
    pub average_rate: f64,
}

async fn ok_handler() -> &'static str {
    "OK"
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::permissive();

    // Build the router
    Router::new()
        .route("/", get(ok_handler))
        .route("/add_pin", post(add_pin))
        .route("/get_pins", get(get_pins))
        .route("/get_pin/{id}", get(get_pin))
        .route("/add_rate", post(add_rate))
        .route("/delete_pin/{id}", delete(delete_pin))
        .with_state(state)
        .layer(ServiceBuilder::new().layer(cors))
}

async fn add_pin(
    State(state): State<AppState>,
    Json(payload): Json<AddPinRequest>,
) -> Result<Json<String>, (StatusCode, String)> {
    state
        .db
        .insert_pin(
            payload.r#type,
            payload.title,
            payload.description,
            payload.x,
            payload.y,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json("Pin added successfully.".to_string()))
}

async fn get_pins(
    State(state): State<AppState>,
) -> Result<Json<Vec<PinResponse>>, (StatusCode, String)> {
    let pins = state
        .db
        .get_all_pins()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response: Vec<PinResponse> = pins
        .into_iter()
        .map(|pin| PinResponse {
            id: pin.id,
            r#type: pin.r#type,
            title: pin.title,
            description: pin.description,
            x: pin.x,
            y: pin.y,
            average_rate: pin.average_rate,
        })
        .collect();

    Ok(Json(response))
}

async fn get_pin(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<PinResponse>, (StatusCode, String)> {
    let pin = state
        .db
        .get_pin_by_id(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    let response = PinResponse {
        id: pin.id,
        r#type: pin.r#type,
        title: pin.title,
        description: pin.description,
        x: pin.x,
        y: pin.y,
        average_rate: pin.average_rate,
    };

    Ok(Json(response))
}

async fn add_rate(
    State(state): State<AppState>,
    Json(payload): Json<AddRatingRequest>,
) -> Result<Json<String>, (StatusCode, String)> {
    state
        .db
        .insert_rating(payload.point_id, payload.rate)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state
        .db
        .update_average_rating(payload.point_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json("Rating added successfully.".to_string()))
}

pub async fn delete_pin(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .db
        .delete_pin(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
