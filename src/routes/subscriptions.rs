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
    email: String,
    name: String,
}

#[tracing::instrument(name = "Adding a new subscriber", skip(connection))]
pub async fn subscribe(
    State(connection): State<Arc<PgPool>>,
    sign_up: Result<Form<FormData>, FormRejection>,
) -> StatusCode {
    match sign_up {
        Ok(form) => match insert_subscriber(&connection, &form).await {
            Ok(_) => {
                tracing::info!("New subscriber details have been saved",);
                StatusCode::OK
            }
            Err(e) => {
                tracing::error!(" Failed to execute query: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        },
        Err(rejection) => {
            tracing::error!("Failed to parse json payload: {:?}", rejection);
            StatusCode::BAD_REQUEST
        }
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(connection)
)]
pub async fn insert_subscriber(connection: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
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
