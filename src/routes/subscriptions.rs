use axum::Form;
use axum::extract::State;
use axum::extract::rejection::FormRejection;
use axum::http::StatusCode;
use sqlx::PgPool;
use sqlx::types::chrono::Utc;
use std::sync::Arc;
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
    if let Ok(form) = sign_up {
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
        .await
        {
            Ok(_) => StatusCode::OK,
            Err(e) => {
                println!("Failed to execute query: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    } else {
        StatusCode::BAD_REQUEST
    }
}
