use crate::startup::AppState;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret: &SecretString) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;
        let query_string = format!("error={}", urlencoding::Encoded::new(&self.error));
        let mut mac =
            Hmac::<sha2::Sha256>::new_from_slice(secret.expose_secret().as_bytes()).unwrap();
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;
        Ok(self.error)
    }
}

pub async fn login_form(
    params: Result<Query<QueryParams>, QueryRejection>,
    State(app_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let error_html = match params {
        Err(_) => "".into(),
        Ok(Query(query)) => match query.verify(&app_state.secret) {
            Ok(error) => {
                format!("<p><i>{}</i></p>", htmlescape::encode_minimal(&error))
            }
            Err(e) => {
                tracing::warn!(error.message = %e, error.cause_chain = ?e, "Failed to verify query parameters using the HMAC tag");
                "".into()
            }
        },
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html")
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta http-equiv="content-type" content="text/html; charset=utf-8">
<title>Login</title>
</head>
<body>
{error_html}
<form action="/login" method="post">
<label>Username
<input
type="text"
placeholder="Enter Username"
name="username"
>
</label>
<label>Password
<input
type="password"
placeholder="Enter Password"
name="password"
>
</label>
<button type="submit">Login</button>
</form>
</body>"#
        ))
        .unwrap()
}
