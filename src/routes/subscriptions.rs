use axum::Form;
use axum::extract::State;
use axum::extract::rejection::FormRejection;
use axum::http::StatusCode;
use sqlx::PgPool;
use sqlx::types::chrono::Utc;
use std::sync::Arc;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    State(connection): State<Arc<PgPool>>,
    sign_up: Result<Form<FormData>, FormRejection>,
) -> StatusCode {
    match sign_up {
        Ok(form) => {
            let request_id = Uuid::new_v4();
            let request_span = tracing::info_span!(
                "Adding a new subscriber.",
                %request_id,
                subscriber_email = %form.email,
                subscriber_name = %form.name
            );
            let _request_span_guard = request_span.enter();

            let query_span = tracing::info_span!("Saving new subscriber details in the database");
            match sqlx::query!(
                r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES($1, $2, $3, $4)
        "#,
                Uuid::new_v4(),
                form.email,
                form.name,
                Utc::now()
            )
            .execute(connection.as_ref())
            .instrument(query_span)
            .await
            {
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
