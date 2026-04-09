use crate::domain::SubscriberEmail;
use crate::error::AppError;
use crate::startup::AppState;
use anyhow::Context;
use axum::Json;
use axum::extract::State;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
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
pub async fn publish_newsletter(
    State(app_state): State<Arc<AppState>>,
    body_data: Result<Json<BodyData>, JsonRejection>,
) -> Result<StatusCode, AppError> {
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
