use crate::{model::ModelController, routes::routes};
use std::net::SocketAddr;

pub use self::error::{Error, Result};
use axum::{
    extract::Query,
    middleware,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;

mod error;
mod model;
mod routes;

#[tokio::main]
async fn main() -> Result<()> {
    let mc = ModelController::new().await?;
    let routes_all = Router::new()
        .merge(routes_ping())
        .layer(middleware::map_response(main_response_mapper))
        .nest("/api", routes(mc));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8001));
    println!("->> LISTENING on {addr}");
    println!();
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();
    Ok(())
}

async fn main_response_mapper(res: Response) -> Response {
    println!("->> {:12} - main_response_mapper {res:?}", "RES_MAPPER");

    println!();
    res
}

#[derive(Debug, Deserialize)]
struct PingParams {
    pub echo: Option<String>,
}

fn routes_ping() -> Router {
    Router::new().route("/ping", get(handler_ping))
}

async fn handler_ping(Query(params): Query<PingParams>) -> impl IntoResponse {
    println!(" ->> {:12} - handler_ping", "HANDLER");

    Html(format!("{}", params.echo.as_deref().unwrap_or("pong")))
}
