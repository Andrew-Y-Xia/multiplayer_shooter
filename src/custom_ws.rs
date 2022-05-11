use crate::physics_engine::PhysicsEngine;
use crate::state::State;
use actix::{Actor, Addr, ArbiterHandle, AsyncContext, Context, Running, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors;
use actix_web_actors::ws;

/// Define HTTP actor
pub struct Ws {
    state: web::Data<State>,
}

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.state.register(ctx.address());
    }
}

/// Handler for ws::Message message
/// Processes requests to
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Ws {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                todo!();
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&web::Bytes::from(msg)),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

/// Handles and starts websocket connection
/// Passes pointer to app state to websocket actor
pub async fn index_ws(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<State>,
) -> Result<HttpResponse, Error> {
    let resp = actix_web_actors::ws::start(
        Ws {
            state: state.clone(),
        },
        &req,
        stream,
    );
    println!("Websocket started: {:?}", resp);
    resp
}
