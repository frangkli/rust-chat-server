use futures::{FutureExt, StreamExt};
use serde_json::from_str;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{filters::ws::Message, ws::WebSocket};

use crate::structs::{Client, Clients, TopicsRequest};

pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client) {
  let (client_ws_sender, mut client_ws_rcv) = ws.split();
  let (client_sender, client_rcv) = mpsc::unbounded_channel();

  let client_rcv = UnboundedReceiverStream::new(client_rcv);
  tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
    if let Err(e) = result {
      eprintln!("error sending websocket msg: {}", e);
    }
  }));

  client.sender = Some(client_sender);
  clients.lock().await.insert(id.clone(), client);

  println!("{} connected", id);

  while let Some(result) = client_ws_rcv.next().await {
    let msg = match result {
      Ok(msg) => msg,
      Err(e) => {
        eprintln!("error receiving ws messag for id: {}): {}", id.clone(), e);
        break;
      }
    };
    client_msg(&id, msg, &clients).await;
  }

  clients.lock().await.remove(&id);
  println!("{} disconnected", id);
}

async fn client_msg(id: &str, msg: Message, clients: &Clients) {
  println!("received message from {}: {:?}", id, msg);
  let message = match msg.to_str() {
    Ok(v) => v,
    Err(_) => return,
  };

  if message == "ping" || message == "ping\n" {
    return;
  }

  let topics_req: TopicsRequest = match from_str(&message) {
    Ok(v) => v,
    Err(e) => {
      eprintln!("error while parsing message to topics request: {}", e);
      return;
    }
  };

  let mut locked = clients.lock().await;
  match locked.get_mut(id) {
    Some(v) => {
      v.topics = topics_req.topics;
    }
    None => return,
  };
}
