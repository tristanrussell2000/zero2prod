use crate::domain::NewSubscriber;
use crate::email_client::EmailClient;
use crate::startup::AppState;
use axum::Form;
use axum::extract::State;
use axum::extract::rejection::FormRejection;
use axum::http::StatusCode;
use rand::distr::Alphanumeric;
use rand::{RngExt, rng};
use sqlx::PgPool;
use sqlx::types::chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

/// Generates a random 25-character long case-sensitive subscription token.
fn generate_subscription_token() -> String {
    let mut rng = rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(name = "Adding a new subscriber", skip(app_state))]
pub async fn subscribe(
    State(app_state): State<Arc<AppState>>,
    sign_up: Result<Form<FormData>, FormRejection>,
) -> StatusCode {
    let connection = &app_state.db_pool;
    let email_client = &app_state.email_client;
    match sign_up {
        Ok(Form(form_data)) => {
            let Ok(new_subscriber) = form_data.try_into() else {
                return StatusCode::BAD_REQUEST;
            };
            let subscriber_id = match insert_subscriber(connection, &new_subscriber).await {
                Ok(subscriber_id) => subscriber_id,
                Err(e) => {
                    tracing::error!(" Failed to execute query: {}", e);
                    return StatusCode::INTERNAL_SERVER_ERROR;
                }
            };
            let subscription_token = generate_subscription_token();
            if store_token(connection, subscriber_id, &subscription_token)
                .await
                .is_err()
            {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }

            if send_confirmation_email(
                email_client,
                new_subscriber,
                &app_state.base_url,
                &subscription_token,
            )
            .await
            .is_err()
            {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        }
        Err(rejection) => {
            tracing::error!("Failed to parse json payload: {:?}", rejection);
            StatusCode::BAD_REQUEST
        }
    }
}

#[tracing::instrument(name = "Store a new subscription token", skip(pool, subsriber_id))]
pub async fn store_token(
    pool: &PgPool,
    subsriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2)"#,
        subscription_token,
        subsriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            &format!(
                "Welcome to our newsletter!<br />\
                            Click <a href=\"{}\">here</a> to confirm your subscription.",
                confirmation_link
            ),
            &format!(
                "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
                confirmation_link
            ),
        )
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(connection, new_subscriber)
)]
pub async fn insert_subscriber(
    connection: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES($1, $2, $3, $4, 'pending_confirmation')
    "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(connection)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(subscriber_id)
}
