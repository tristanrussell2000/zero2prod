use std::sync::Arc;
use axum::extract::State;
use axum::Form;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use secrecy::SecretString;
use crate::authentication::{validate_credentials, AuthError, Credentials};
use crate::error::AppError;
use crate::startup::AppState;

#[derive(serde::Deserialize)]
pub struct LoginFormData {
    username: String,
    password: SecretString,
}

#[tracing::instrument(
    skip(form, app_state), 
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(State(app_state): State<Arc<AppState>>, Form(form): Form<LoginFormData>) -> Result<impl IntoResponse, AppError> {
    let pool = &app_state.db_pool;
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };
    tracing::Span::current()
        .record("username", &tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => AppError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => AppError::UnexpectedError(e.into()),
        })?;
    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
    
    Ok(Response::builder()
        .header("Location", "/")
        .status(StatusCode::SEE_OTHER)
        .body(axum::body::Body::empty())
        .unwrap()
    )
}
