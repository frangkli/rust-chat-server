mod handler;
mod structs;
mod ws;

use std::{collections::HashMap, convert::Infallible, sync::Arc};
use structs::Clients;
use tokio::sync::Mutex;
use warp::Filter;

#[tokio::main]
async fn main() {
  let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

  // GET /health
  let health_route = warp::path!("health").and_then(handler::health_handler);

  // POST /register
  let register_route = warp::path("register")
    .and(warp::post())
    .and(warp::body::json())
    .and(with_clients(clients.clone()))
    .and_then(handler::register_handler);

  // DELETE  /register/{client_id}
  let unregister_route = warp::path("register")
    .and(warp::delete())
    .and(warp::path::param())
    .and(with_clients(clients.clone()))
    .and_then(handler::unregister_handler);

  // POST /publish
  let publish_route = warp::path!("publish")
    .and(warp::post())
    .and(warp::body::json())
    .and(with_clients(clients.clone()))
    .and_then(handler::publish_handler);

  // WS /ws/{uuid}
  let ws_route = warp::path("ws")
    .and(warp::ws())
    .and(warp::path::param())
    .and(with_clients(clients.clone()))
    .and_then(handler::ws_handler);

  let routes = health_route
    .or(register_route)
    .or(unregister_route)
    .or(publish_route)
    .or(ws_route)
    .with(warp::cors().allow_any_origin());

  // localhost:8000
  warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
  warp::any().map(move || clients.clone())
}
