// Schema routes
// Handles routes for database schema inspection

use crate::services::schema_service;
use crate::AppState;
use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};

#[derive(Template)]
#[template(path = "components/schema-list.html")]
pub struct SchemaListTemplate {
    pub schemas: Vec<crate::models::Schema>,
}

/// Lists all schemas in the current database (returns HTML)
pub async fn list_schemas(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let schemas = schema_service::list_schemas(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = SchemaListTemplate { schemas };
    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Gets details about a specific schema
pub async fn schema_details(
    Path(schema_name): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let tables = schema_service::list_tables(&state.db_pool, &schema_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "name": schema_name,
        "tables": tables
    })))
}

/// Lists all schemas with their tables, views, and functions as tree structure (JSON)
#[allow(dead_code)]
pub async fn schema_tree(State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let schemas = schema_service::list_schemas(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut tree = Vec::new();

    for schema in schemas {
        let tables = schema_service::list_tables(&state.db_pool, &schema.name)
            .await
            .unwrap_or_default();

        let views = schema_service::list_views(&state.db_pool, &schema.name)
            .await
            .unwrap_or_default();

        let functions = schema_service::list_functions(&state.db_pool, &schema.name)
            .await
            .unwrap_or_default();

        tree.push(serde_json::json!({
            "name": schema.name,
            "owner": schema.owner,
            "tables": tables.iter().map(|t| &t.name).collect::<Vec<_>>(),
            "views": views,
            "functions": functions,
        }));
    }

    Ok(Json(tree))
}

#[derive(Template)]
#[template(path = "components/schema-tree.html")]
pub struct SchemaTreeTemplate {
    pub schemas: Vec<crate::models::Schema>,
}

/// Lists all schemas as a tree view (HTML)
pub async fn schema_tree_html(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let schemas = schema_service::list_schemas(&state.db_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = SchemaTreeTemplate { schemas };
    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Template)]
#[template(path = "components/tables-tree.html")]
pub struct TablesTreeTemplate {
    pub schema_name: String,
    pub tables: Vec<crate::models::TableInfo>,
}

/// Lists tables for a schema (HTML for tree)
pub async fn tables_list_html(
    Path(schema_name): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let tables = schema_service::list_tables(&state.db_pool, &schema_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = TablesTreeTemplate {
        schema_name,
        tables,
    };
    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Template)]
#[template(path = "components/views-tree.html")]
pub struct ViewsTreeTemplate {
    pub schema_name: String,
    pub views: Vec<String>,
}

/// Lists views for a schema (HTML for tree)
pub async fn views_list_html(
    Path(schema_name): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let views = schema_service::list_views(&state.db_pool, &schema_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = ViewsTreeTemplate { schema_name, views };
    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Template)]
#[template(path = "components/functions-tree.html")]
pub struct FunctionsTreeTemplate {
    pub functions: Vec<String>,
}

/// Lists functions for a schema (HTML for tree)
pub async fn functions_list_html(
    Path(schema_name): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let functions = schema_service::list_functions(&state.db_pool, &schema_name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = FunctionsTreeTemplate { functions };
    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
