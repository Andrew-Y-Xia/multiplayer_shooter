use crate::custom_ws::Ws;
use crate::physics_engine::PhysicsEngine;
use actix::Addr;
use dashmap::DashSet;

// State holds overall application state
// It records the currently connected players
// It holds an address to the physics engine, which websocket actors will access
pub struct State {
    connected_players: DashSet<Addr<Ws>>, // Lockless!
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

    // Removes websocket connection
    // Called when actor stops
    pub fn remove(&self, address: Addr<Ws>) {
        self.connected_players.remove(&address);
    }
}
