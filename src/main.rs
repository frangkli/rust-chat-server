mod handler;
mod ws;

use std::{collections::HashMap, convert::Infallible, sync::Arc};

use tokio::sync::{mpsc, Mutex};
use warp::{filters::ws::Message, reject::Rejection, Error, Filter};

#[derive(Clone)]
pub struct Client {
  pub user_id: usize,
  pub topics: Vec<String>,
  pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, Error>>>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RegisterRequest {
  user_id: usize,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RegisterResponse {
  url: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Event {
  topic: String,
  user_id: Option<usize>,
  message: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TopicsRequest {
  topics: Vec<String>,
}

type Result<T> = std::result::Result<T, Rejection>;
type Clients = Arc<Mutex<HashMap<String, Client>>>;

#[tokio::main]
async fn main() {
  let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

  // GET /health
  let health_route = warp::path!("health").and_then(handler::health_handler);

  // POST /register
  // DELETE  /register/{client_id}
  let register = warp::path("register");
  let register_routes = register
    .and(warp::post())
    .and(warp::body::json())
    .and(with_clients(clients.clone()))
    .and_then(handler::register_handler)
    .or(
      register
        .and(warp::delete())
        .and(warp::path::param())
        .and(with_clients(clients.clone()))
        .and_then(handler::unregister_handler),
    );

  // POST /publish
  let publish = warp::path!("publish")
    .and(warp::body::json())
    .and(with_clients(clients.clone()))
    .and_then(handler::publish_handler);

  // GET /ws
  let ws_route = warp::path("ws")
    .and(warp::ws())
    .and(warp::path::param())
    .and(with_clients(clients.clone()))
    .and_then(handler::ws_handler);

  let routes = health_route
    .or(register_routes)
    .or(publish)
    .or(ws_route)
    .with(warp::cors().allow_any_origin());

  warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
  warp::any().map(move || clients.clone())
}
