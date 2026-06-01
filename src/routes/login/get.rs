use axum::response::{Html, IntoResponse};

pub async fn login_form() -> impl IntoResponse {
    Html(include_str!("login.html"))
}
