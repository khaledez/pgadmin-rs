// Route modules
pub mod query;
pub mod tables;
pub mod schema;
pub mod export;
pub mod schema_ops;
pub mod stats;
pub mod table_view;

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate;

#[derive(Template)]
#[template(path = "query.html")]
pub struct QueryTemplate;

#[derive(Template)]
#[template(path = "browser.html")]
pub struct BrowserTemplate;

pub async fn index() -> impl IntoResponse {
    HtmlTemplate(DashboardTemplate)
}

pub async fn page_query() -> impl IntoResponse {
    HtmlTemplate(QueryTemplate)
}

pub async fn page_browser() -> impl IntoResponse {
    HtmlTemplate(BrowserTemplate)
}

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", err),
            )
                .into_response(),
        }
    }
}
