// Database routes
// Handles routes for database-level operations

use crate::services::database_service;
use crate::AppState;
use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "components/database-list.html")]
pub struct DatabaseListTemplate {
    pub databases: Vec<crate::models::Database>,
}

/// Lists all databases on the PostgreSQL server (returns HTML)
pub async fn list_databases(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let databases = database_service::list_databases(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = DatabaseListTemplate { databases };
    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Lists all databases (returns JSON)
pub async fn list_databases_json(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let databases = database_service::list_databases(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(databases))
}

/// Gets details about a specific database
pub async fn get_database(
    Path(db_name): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let database = database_service::get_database_info(&state.db_pool, &db_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(database))
}

#[derive(Deserialize)]
pub struct CreateDatabaseRequest {
    pub name: String,
    pub owner: Option<String>,
}

/// Creates a new database
pub async fn create_database(
    State(state): State<AppState>,
    Json(req): Json<CreateDatabaseRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let owner = req.owner.as_deref();

    database_service::create_database(&state.db_pool, &req.name, owner)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create database: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Database created: {}", req.name);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Database '{}' created successfully", req.name)
    })))
}

#[derive(Deserialize)]
pub struct DropDatabaseRequest {
    pub name: String,
}

/// Drops a database
pub async fn drop_database(
    State(state): State<AppState>,
    Json(req): Json<DropDatabaseRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    database_service::drop_database(&state.db_pool, &req.name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to drop database: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Database dropped: {}", req.name);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Database '{}' dropped successfully", req.name)
    })))
}
