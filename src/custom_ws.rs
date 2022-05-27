use crate::physics_engine::EnemyInfo;
use crate::state::State;
use crate::{physics_engine::PhysicsStateResponse, state::PlayerInfo};
use actix::{Actor, Addr, AsyncContext, Handler, Message, StreamHandler};
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
pub enum ClientInstruction {
    JoinGame {
        username: String,
    },
    GameAction {
        w: bool,
        a: bool,
        s: bool,
        d: bool,
        dir: f32,
    },
}

#[derive(Debug)]
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
/// Processes requests to Physics Engine
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Ws {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // Parse JSON from client
                // TODO: Error handle this properly
                let mut action: ClientInstruction = serde_json::from_slice(text.as_ref()).unwrap();

                // If the name is blank then change name to 'Unnamed'
                if let ClientInstruction::JoinGame { username: s } = &mut action {
                    *s = String::from("Unnamed");
                }

                let mut player_info = self
                    .state
                    .connected_players
                    .get_mut(&ctx.address())
                    .unwrap();

                let action = match action {
                    ClientInstruction::JoinGame { username } => {
                        // Now save our username
                        // Move the username out and construct new game instruction
                        player_info.username = username;
                        GameInstruction::JoinGame
                    }
                    ClientInstruction::GameAction { w, a, s, d, dir } => {
                        // Save the dir
                        player_info.dir = dir;
                        GameInstruction::GameAction { w, a, s, d }
                    }
                };

                // Wrap instruction with our Actor Address (so that the physics engine can remember who's who)
                let physics_instruction = PhysicsInstruction {
                    game_instruction: action,
                    sent_from: ctx.address(),
                };

                // Finally, send the data
                self.state
                    .get_ref()
                    .get_physics_engine()
                    .do_send(physics_instruction);
            }
            _ => (),
        }
    }
}

// Routes state info from the physics engine back to the client
impl Handler<PhysicsStateResponse> for Ws {
    type Result = ();

    fn handle(&mut self, mut msg: PhysicsStateResponse, ctx: &mut Self::Context) -> Self::Result {
        let address = ctx.address();
        for EnemyInfo { dir , .. } in msg.enemies.iter_mut() {
            *dir = self.state.connected_players.get(&address).unwrap().dir;
        }
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
