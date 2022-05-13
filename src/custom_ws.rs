use crate::physics_engine::PhysicsStateResponse;
use crate::state::State;
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, Running, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors;
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use serde_json;

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

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum GameInstruction {
    JoinGame,
    GameAction { w: bool, a: bool, s: bool, d: bool },
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct PhysicsInstruction {
    pub game_instruction: GameInstruction,
    pub sent_from: Addr<Ws>,
}

/// Handler for ws::Message message
/// Processes requests to
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Ws {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let action: GameInstruction = serde_json::from_slice(text.as_ref()).unwrap();
                let physics_instruction = PhysicsInstruction {
                    game_instruction: action,
                    sent_from: ctx.address(),
                };
                // println!("Serde: {:?}", physics_instruction);
                self.state
                    .get_ref()
                    .get_physics_engine()
                    .do_send(physics_instruction);
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&web::Bytes::from(msg)),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

impl Handler<PhysicsStateResponse> for Ws {
    type Result = ();

    fn handle(&mut self, msg: PhysicsStateResponse, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(serde_json::to_string(&msg).unwrap())
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
