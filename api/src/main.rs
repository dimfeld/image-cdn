mod config;
mod panic_handler;
mod routes;
mod tracing_config;

use std::{
    error::Error,
    net::{IpAddr, SocketAddr},
};

use axum::{Extension, Router};
use clap::Parser;
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    request_id::MakeRequestUuid,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    ServiceBuilderExt,
};
use tracing::{event, Level};

use crate::tracing_config::HoneycombConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install()?;
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

    let production = (config.env != "development" && !cfg!(debug_assertions));

    let app = routes::configure_routes(Router::new()).layer(
        ServiceBuilder::new()
            .layer(Extension(db))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO))
                    .on_request(DefaultOnRequest::new().level(Level::INFO)),
            )
            .set_x_request_id(MakeRequestUuid)
            .propagate_x_request_id()
            .compression()
            .decompression()
            .layer(CatchPanicLayer::custom(move |err| {
                panic_handler::handle_panic(production, err)
            }))
            .into_inner(),
    );

    let bind_ip: IpAddr = config.host.parse()?;
    let addr = SocketAddr::from((bind_ip, config.port));
    let builder = axum::Server::bind(&addr);
    event!(Level::INFO, "Listening on {}:{}", config.host, config.port);

    builder.serve(app.into_make_service()).await?;

    tracing_config::teardown();

    Ok(())
}
