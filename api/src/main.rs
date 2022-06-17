mod config;
mod routes;
mod tracing_config;

use std::net::{IpAddr, SocketAddr};

use axum::{Extension, Router};
use clap::Parser;
use tower::ServiceBuilder;
use tracing::{event, Level};

use crate::tracing_config::HoneycombConfig;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    let mut config = config::Config::parse();

    let db = pic_store_db::connect(config.database_url.as_str()).await?;

    let honeycomb_config = if let Some(team) = config.honeycomb_team.take() {
        Some(HoneycombConfig {
            team,
            dataset: std::mem::take(&mut config.honeycomb_dataset),
        })
    } else {
        None
    };

    tracing_config::configure(honeycomb_config)?;

    let app =
        routes::configure_routes(Router::new()).layer(ServiceBuilder::new().layer(Extension(db)));

    let bind_ip: IpAddr = config.host.parse()?;
    let addr = SocketAddr::from((bind_ip, config.port));
    let builder = axum::Server::bind(&addr);
    event!(Level::INFO, "Listening on {}:{}", config.host, config.port);

    builder.serve(app.into_make_service()).await?;

    tracing_config::teardown();

    Ok(())
}