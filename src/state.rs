use crate::custom_ws::Ws;
use crate::physics_engine::PhysicsEngine;
use actix::Addr;
use dashmap::DashSet;

pub struct State {
    connected_players: DashSet<Addr<Ws>>,
    physics_engine_address: Addr<PhysicsEngine>,
}

impl State {
    pub fn new(physics_engine_address: Addr<PhysicsEngine>) -> State {
        State {
            connected_players: DashSet::new(),
            physics_engine_address,
        }
    }

    // Registers new websocket connection to the game server
    pub fn register(&self, address: Addr<Ws>) {
        self.connected_players.insert(address);
    }
}
