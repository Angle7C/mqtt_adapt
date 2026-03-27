use std::net::{SocketAddr, SocketAddrV4};

use anyhow::Result;
use mqtt::db::connection::DatabaseConnection;
use sqlx::database;
use tokio::{net::ToSocketAddrs, runtime::Builder};
use tracing::Level;
#[tokio::main]
pub async fn main() -> Result<()> {
    init_tracing();
    let server = mqtt::server::Server::new("127.0.0.1:1883".parse().unwrap());
    let database = DatabaseConnection::new("sqlite://mqtt_adapt.db").await?;

    let server = server.with_database(database);
    server.start().await;

    Ok(())
}

pub fn init_tracing() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
}
