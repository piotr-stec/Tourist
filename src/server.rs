use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::trace;

use crate::db::TouristDb;
use crate::db::sql_lite::SqliteDb;
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
pub struct AddCommentRequest {
    pub pin_id: i32,
    pub date: String,
    pub author: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct CommentResponse {
    pub id: i32,
    pub pin_id: i32,
    pub date: String,
    pub author: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct PinResponse {
    pub id: i32,
    pub r#type: String,
    pub title: String,
    pub description: String,
    pub x: f64,
    pub y: f64,
    pub comments_count: u64,
}

async fn ok_handler() -> &'static str {
    "OK"
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::permissive();

    // Build the router
    Router::new()
        .route("/get_pin/:id", get(get_pin))
        .route("/delete_pin/:id", delete(delete_pin))
        .route("/get_comments/:id", get(get_comments))
        .route("/", get(ok_handler))
        .route("/add_pin", post(add_pin))
        .route("/get_pins", get(get_pins))
        .route("/add_comment", post(add_comment))
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

async fn add_comment(
    State(state): State<AppState>,
    Json(payload): Json<AddCommentRequest>,
) -> Result<Json<String>, (StatusCode, String)> {
    trace!("Add comment");
    state
        .db
        .insert_comment(
            payload.pin_id,
            payload.date,
            payload.author,
            payload.content,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json("Comment added successfully.".to_string()))
}

async fn get_comments(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<CommentResponse>>, (StatusCode, String)> {
    let comments = state
        .db
        .get_pin_comments(id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    let response: Vec<CommentResponse> = comments
        .into_iter()
        .map(|comment| CommentResponse {
            id: comment.id,
            pin_id: comment.pin_id,
            date: comment.date,
            author: comment.author,
            content: comment.content,
        })
        .collect();

    Ok(Json(response))
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
            comments_count: pin.comments_count,
        })
        .collect();

    Ok(Json(response))
}

async fn get_pin(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<PinResponse>, (StatusCode, String)> {
    println!("xd1");
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
        comments_count: pin.comments_count,
    };

    Ok(Json(response))
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
