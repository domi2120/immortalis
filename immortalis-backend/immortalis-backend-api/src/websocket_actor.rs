use actix::prelude::*;
use actix::{Actor, Addr, Handler, StreamHandler};
use actix_web_actors::ws::{self};
use tracing::info;
use std::collections::hash_map::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Clone)]
pub struct WebSocketActor {
    pub web_socket_connections: Arc<RwLock<HashMap<String, Addr<WebSocketActor>>>>,
    id: Uuid
}

impl WebSocketActor {
    pub fn new (web_socket_connections: Arc<RwLock<HashMap<String, Addr<WebSocketActor>>>>) -> WebSocketActor {
        WebSocketActor { web_socket_connections, id: Uuid::new_v4() }
    }
}

impl Actor for WebSocketActor {
    type Context = actix_web_actors::ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Actor started for id {}", self.id);
        self.web_socket_connections
            .write()
            .unwrap()
            .insert(self.id.to_string(), ctx.address());
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.web_socket_connections.write().unwrap().remove(&self.id.to_string());
        info!("Actor stopped for id {}", self.id);
    }
}

impl Handler<Message> for WebSocketActor {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);
