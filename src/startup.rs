use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};
use axum::Router;
use axum::http::Request;
use axum::routing::{get, post};
use axum::serve::Serve;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info_span;
use uuid::Uuid;

pub struct AppState(pub PgPool, pub EmailClient);

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Serve<TcpListener, Router, Router>, std::io::Error> {
    let app = Router::new()
        .route("/healthcheck", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(Arc::new(AppState(db_pool, email_client)))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                info_span!(
                    "http_request",
                    method = ?request.method(),
                    request_id = %Uuid::new_v4()
                )
            }),
        );

    let serve = axum::serve(listener, app);
    Ok(serve)
}

#[cfg(test)]
mod tests {
    use crate::routes::health_check;

    #[tokio::test]
    async fn health_check_handler_succeeds() {
        let response = health_check().await;
        assert!(response.is_success());
    }
}
