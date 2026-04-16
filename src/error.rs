use axum::http::header::WWW_AUTHENTICATE;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

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
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::ValidationError(_) => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(())
                .unwrap(),
            AppError::AuthError(_) => {
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();
                Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header(WWW_AUTHENTICATE, header_value)
                    .body(())
                    .unwrap()
            }
            _ => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(())
                .unwrap(),
        };

        tracing::error!(exception.message = %self, exception.details = ?self, "Response failed");

        (status, format!("{:?}", self)).into_response()
    }
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
