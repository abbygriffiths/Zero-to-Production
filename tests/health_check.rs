use std::net::TcpListener;

use sqlx::{Connection, PgConnection};
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{address}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_when_valid_data_present() {
    // Arrange
    let app_address = spawn_app();
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();

    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to database");
    let client = reqwest::Client::new();

    // Act
    let body = "name=bunny%20mcbunbun&email=mewsbunny%40mewbun.com";
    let response = client
        .post(&format!("{app_address}/subscriptions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send POST request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let saved = sqlx::query!("SELECT name, email FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "mewsbunny@mewbun.com");
    assert_eq!(saved.name, "bunny mcbunbun");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_missing() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=bunny%20mcbunbun", "missing email"),
        ("email=mewsbunny%40mewbun.com", "missing email"),
        ("", "missing email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{address}/subscriptions"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to send POST request");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "API did not return 400 when payload was {}",
            error_message
        );
    }
}

fn spawn_app() -> String {
    let listener =
        TcpListener::bind("127.0.0.1:0").expect("Failed to bind listener to random port.");

    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to establish server");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{port}")
}
