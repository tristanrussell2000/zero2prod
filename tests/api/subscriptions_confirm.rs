use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    let test_app = spawn_app().await;

    let response = reqwest::get(&format!("{}/subscriptions/confirm", test_app.address))
        .await
        .unwrap();

    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    let test_app = spawn_app().await;
    let body = "name=John&email=john@gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.into()).await;
    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = test_app.get_confirmation_links(email_request);

    let response = reqwest::get(confirmation_links.html).await.unwrap();
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    let app = spawn_app().await;
    let body = "name=TestPerson&email=test@email.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "test@email.com");
    assert_eq!(saved.name, "TestPerson");
    assert_eq!(saved.status, "confirmed");
}

#[tokio::test]
async fn subscribing_twice_sends_valid_confirmation_email() {
    let test_app = spawn_app().await;
    let body = "name=John&email=john@gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.into()).await;
    test_app.post_subscriptions(body.into()).await;

    assert_eq!(
        2,
        test_app
            .email_server
            .received_requests()
            .await
            .unwrap()
            .len()
    );

    let email_request = &test_app.email_server.received_requests().await.unwrap()[1];
    let confirmation_links = test_app.get_confirmation_links(email_request);

    let response = reqwest::get(confirmation_links.html).await.unwrap();
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "john@gmail.com");
    assert_eq!(saved.name, "John");
    assert_eq!(saved.status, "confirmed");
}
