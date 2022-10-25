use sqlx::PgPool;
use std::net::TcpListener;
use z2p::configuration::get_configuration;
use z2p::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to database");
    let listener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind port 8000");
    run(listener, connection_pool)?.await
}
