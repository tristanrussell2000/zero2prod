use axum::extract::Query;
use axum::http::StatusCode;

#[derive(serde::Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String,
}
#[tracing::instrument(name = "Confirming a pending subscriber", skip(_parameters))]
pub async fn confirm(_parameters: Query<Parameters>) -> StatusCode {
    StatusCode::OK
}
