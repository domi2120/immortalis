use actix::prelude::*;
use actix::{Actor, Addr, Handler, StreamHandler};
use uuid::Uuid;
use std::collections::hash_map::HashMap;
use std::sync::{Arc, RwLock};
use actix_web_actors::ws::{self};

#[derive(Clone)]
pub struct ScheduledArchivalsEventHandler {
    pub web_socket_connections: Arc<RwLock<HashMap<String, Addr<ScheduledArchivalsEventHandler>>>>,
}

impl Actor for ScheduledArchivalsEventHandler {
    type Context = actix_web_actors::ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.web_socket_connections
            .write()
            .unwrap()
            .insert(Uuid::new_v4().to_string(), ctx.address());
    }
}

impl Handler<Message> for ScheduledArchivalsEventHandler {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ScheduledArchivalsEventHandler {
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