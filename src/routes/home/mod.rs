use axum::body::Body;
use axum::response::{IntoResponse, Response};

pub async fn home() -> impl IntoResponse {
    Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(Body::from(include_str!("home.html")))
        .unwrap()
}