use crate::domain::subscription_token::SubscriptionToken;
use crate::error::AppError;
use crate::startup::AppState;
use anyhow::Context;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub struct Parameters {
    subscription_token: SubscriptionToken,
}
#[tracing::instrument(name = "Confirming a pending subscriber", skip(parameters, app_state))]
pub async fn confirm(
    parameters: Query<Parameters>,
    State(app_state): State<Arc<AppState>>,
) -> Result<StatusCode, AppError> {
    let id =
        get_subscriber_id_from_token(&app_state.db_pool, parameters.subscription_token.as_ref())
            .await?;

    match id {
        None => Ok(StatusCode::NOT_FOUND),
        Some(subscriber_id) => {
            confirm_subscriber(&app_state.db_pool, subscriber_id).await?;

            Ok(StatusCode::OK)
        }
    }
}

#[tracing::instrument(name = "Confirming a subscriber", skip(pool, subscriber_id))]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), AppError> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await
    .context("Failed to execute confirm subscriber query.")?;
    Ok(())
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(subscription_token, pool))]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, AppError> {
    let result = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1",
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .context("Failed to execute get subscriber id from token query.")?;
    Ok(result.map(|row| row.subscriber_id))
}
