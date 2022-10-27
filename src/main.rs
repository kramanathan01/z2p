use sqlx::postgres::PgPoolOptions;
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

    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
