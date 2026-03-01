use axum::Router;
use axum::routing::{get, post};
use axum::serve::Serve;
use tokio::net::TcpListener;
use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener) -> Result<Serve<TcpListener, Router, Router>, std::io::Error> {
    let app = Router::new()
        .route("/healthcheck", get(health_check))
        .route("/subscriptions", post(subscribe));
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