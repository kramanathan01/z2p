use sqlx::PgPool;
use std::net::TcpListener;

use z2p::configuration::get_configuration;
use z2p::startup::run;
use z2p::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Logging Setup
    let subscriber = get_subscriber("z2p".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Setup http server and db connection pool
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to database");
    let listener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind port 8000");
    run(listener, connection_pool)?.await
}
