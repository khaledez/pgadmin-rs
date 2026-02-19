use askama::Template;
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use std::sync::Arc;

use crate::middleware::auth::{derive_session_token, AuthState};

use super::HtmlTemplate;

const SESSION_COOKIE_NAME: &str = "pgadmin_session";

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error: String,
}

#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub password: String,
}

pub async fn login_page() -> impl IntoResponse {
    HtmlTemplate(LoginTemplate {
        error: String::new(),
    })
}

pub async fn login_submit(
    State(auth): State<Arc<AuthState>>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    let expected_password = match &auth.session_token {
        Some(_) => {},
        None => return (jar, Redirect::to("/")).into_response(),
    };
    let _ = expected_password;

    // Derive token from submitted password and compare to expected
    let submitted_token = derive_session_token(&form.password);
    if Some(&submitted_token) == auth.session_token.as_ref() {
        let cookie = Cookie::build((SESSION_COOKIE_NAME, submitted_token))
            .path("/")
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .build();
        let jar = jar.add(cookie);
        return (jar, Redirect::to("/")).into_response();
    }

    HtmlTemplate(LoginTemplate {
        error: "Invalid password".to_string(),
    })
    .into_response()
}
