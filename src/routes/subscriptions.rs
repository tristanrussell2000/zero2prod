use crate::domain::NewSubscriber;
use crate::startup::AppState;
use axum::Form;
use axum::extract::State;
use axum::extract::rejection::FormRejection;
use axum::http::StatusCode;
use sqlx::PgPool;
use sqlx::types::chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

#[tracing::instrument(name = "Adding a new subscriber", skip(connection))]
pub async fn subscribe(
    State(connection): State<Arc<AppState>>,
    sign_up: Result<Form<FormData>, FormRejection>,
) -> StatusCode {
    let connection = &connection.0;
    match sign_up {
        Ok(Form(form_data)) => {
            let Ok(new_subscriber) = form_data.try_into() else {
                return StatusCode::BAD_REQUEST;
            };
            match insert_subscriber(connection, &new_subscriber).await {
                Ok(_) => {
                    tracing::info!("New subscriber details have been saved",);
                    StatusCode::OK
                }
                Err(e) => {
                    tracing::error!(" Failed to execute query: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
        Err(rejection) => {
            tracing::error!("Failed to parse json payload: {:?}", rejection);
            StatusCode::BAD_REQUEST
        }
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(connection, new_subscriber)
)]
pub async fn insert_subscriber(
    connection: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
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
    Ok(())
}
