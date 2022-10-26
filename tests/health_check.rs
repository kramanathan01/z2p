use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use tracing::subscriber;
use uuid::Uuid;
use z2p::configuration::{get_configuration, DatabaseSettings};
use z2p::telemetry::{get_subscriber, init_subscriber};

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".into();
    let subscriber_name = "test".into();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    random_db_name: String,
}

async fn spawn_app() -> TestApp {
    // Setup logging
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    // Binding to Port 0 causes a scan for available port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // Get Connection Pool
    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    // Spawn Server
    let server = z2p::startup::run(listener, connection_pool.clone()).expect("Failed to bind");
    let _ = tokio::spawn(server);

    // Return Struct for use within test
    TestApp {
        address,
        db_pool: connection_pool,
        random_db_name: configuration.database.database_name,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres server");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to database");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database");
    connection_pool
}

pub async fn drop_database(db: String) {
    let configuration = get_configuration().expect("Failed to read configuration");
    let mut connection =
        PgConnection::connect(&configuration.database.connection_string_without_db())
            .await
            .expect("Failed to connect to Postgres server");
    connection
        .execute(format!(r#"DROP DATABASE "{}" with (FORCE);"#, db).as_str())
        .await
        .expect("Failed to drop database");
}

#[tokio::test]
async fn health_check_works() {
    // Test Setup
    let testapp = spawn_app().await;

    // Run Test
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health_check", testapp.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert Results
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

    // Cleanup
    // HTTP Server will automatically terminate because we used tokio::spawn
    drop_database(testapp.random_db_name).await;
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    // Test Setup
    let testapp = spawn_app().await;

    // Run Test
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &testapp.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert Results
    assert_eq!(200, response.status().as_u16()); // Assert HTTP Response code

    // Assert DB Persistence
    let saved = sqlx::query!("SELECT email, name from subscriptions",)
        .fetch_one(&testapp.db_pool)
        .await
        .expect("Failed to fetch saved subscription");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

    // Cleanup
    // HTTP Server will automatically terminate because we used tokio::spawn
    drop_database(testapp.random_db_name).await;
}

#[tokio::test]
async fn subscribe_returns_400_for_bad_form_data() {
    // Test Setup
    let testapp = spawn_app().await;

    // Run Test
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing email"),
        ("email=ursula_le_guin%40gmail.com", "missing name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &testapp.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when payload was {}",
            error_message
        )
    }

    // Cleanup
    drop_database(testapp.random_db_name).await;
}
