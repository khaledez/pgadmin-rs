// Route modules
pub mod query;
pub mod tables;
pub mod schema;
pub mod auth;

use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,
}

pub async fn index() -> impl IntoResponse {
    let template = IndexTemplate {
        title: "pgAdmin-rs".to_string(),
    };
    HtmlTemplate(template)
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
