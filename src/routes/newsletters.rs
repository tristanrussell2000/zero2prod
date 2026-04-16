use crate::domain::SubscriberEmail;
use crate::error::AppError;
use crate::startup::AppState;
use anyhow::Context;
use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use reqwest::header::HeaderMap;
use secrecy::{ExposeSecret, SecretString};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

#[tracing::instrument(
    name = "Publish newsletter", 
    skip(headers, app_state, body_data),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    headers: HeaderMap,
    State(app_state): State<Arc<AppState>>,
    body_data: Result<Json<BodyData>, JsonRejection>,
) -> Result<StatusCode, AppError> {
    let credentials = basic_authentication(&headers).map_err(AppError::AuthError)?;
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    let user_id = validate_credentials(&credentials, &app_state.db_pool)
        .await
        .map_err(AppError::AuthError)?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    let subscribers = get_confirmed_subscribers(&app_state.db_pool).await?;
    let body_data =
        body_data.map_err(|_| AppError::ValidationError("Invalid JSON payload".into()))?;
    for subscriber in subscribers {
        match subscriber {
            Ok(subscriber) => {
                app_state
                    .email_client
                    .send_email(
                        &subscriber.email,
                        &body_data.title,
                        &body_data.content.html,
                        &body_data.content.text,
                    )
                    .await
                    .with_context(|| format!("Failed to send email to {}", subscriber.email))?;
            }
            Err(error) => {
                tracing::warn!(error.cause_chain = ?error, "Skipping a confirmed subscriber. Their stored contact details are invalid.");
            }
        }
    }
    Ok(StatusCode::OK)
}

struct Credentials {
    username: String,
    password: SecretString,
}

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("Missing Authorization header")?
        .to_str()
        .context("The Authorization header was not a valid UTF8 string")?;

    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic'")?;

    let decoded_bytes = base64::decode_config(base64encoded_segment, base64::STANDARD)
        .context("Failed to base64-encode 'Basic' credentials")?;

    let decoded_credentials =
        String::from_utf8(decoded_bytes).context("Failed to decode base64-encoded credentials")?;

    let mut credentials = decoded_credentials.splitn(2, ':');
    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth."))?;
    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth."))?;

    Ok(Credentials {
        username: username.to_string(),
        password: SecretString::from(password),
    })
}

async fn validate_credentials(
    credentials: &Credentials,
    pg_pool: &PgPool,
) -> Result<uuid::Uuid, anyhow::Error> {
    let user_id: Option<_> = sqlx::query!(
        r#"SELECT user_id FROM users WHERE username = $1 AND password = $2"#,
        credentials.username,
        credentials.password.expose_secret()
    )
    .fetch_optional(pg_pool)
    .await
    .context("Failed to perform a query to validate auth credentials")?;

    user_id
        .map(|row| row.user_id)
        .ok_or_else(|| anyhow::anyhow!("Invalid credentials"))
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(db_pool))]
async fn get_confirmed_subscribers(
    db_pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers =
        sqlx::query!("SELECT email FROM subscriptions WHERE status = 'confirmed'")
            .fetch_all(db_pool)
            .await?
            .into_iter()
            .map(|r| match SubscriberEmail::parse(r.email) {
                Ok(email) => Ok(ConfirmedSubscriber { email }),
                Err(error) => Err(anyhow::anyhow!(error)),
            })
            .collect();

    Ok(confirmed_subscribers)
}
