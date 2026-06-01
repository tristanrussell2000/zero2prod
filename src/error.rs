use axum::body::Body;
use axum::http::header::{LOCATION, WWW_AUTHENTICATE};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use crate::authentication::AuthError;

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

#[derive(thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    ValidationError(String),
    #[error("Failed to authenticate publish newsletter")]
    PublishAuthError(#[source] AuthError),
    #[error("Authentication failed")]
    AuthError(#[source] AuthError, SecretString),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::ValidationError(_) => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap(),
            AppError::AuthError(e, secret) => {
                let query_string = format!(
                    "error={}",
                    urlencoding::Encoded::new(e.to_string())
                );
                let hmac_tag = {
                    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(
                        secret.expose_secret().as_bytes()
                    ).unwrap();
                    mac.update(query_string.as_bytes());
                    mac.finalize().into_bytes()
                };
                Response::builder()
                    .status(StatusCode::SEE_OTHER)
                    .header(LOCATION, format!("/login?{query_string}&tag={hmac_tag:x}"))
                    .body(Body::empty())
                    .unwrap()
            }
            AppError::PublishAuthError(_) => {
                Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header(WWW_AUTHENTICATE, r#"Basic realm="publish""#)
                    .body(Body::empty())
                    .unwrap()
            }
            _ => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap(),
        };

        tracing::error!(exception.message = %self, exception.details = ?self, "Response failed");

        status
    }
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
