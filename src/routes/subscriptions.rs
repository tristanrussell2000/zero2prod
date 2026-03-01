use axum::extract::rejection::FormRejection;
use axum::Form;
use axum::http::StatusCode;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String
}

pub async fn subscribe(sign_up: Result<Form<FormData>, FormRejection>) -> StatusCode {
    match sign_up {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::BAD_REQUEST
    }
}