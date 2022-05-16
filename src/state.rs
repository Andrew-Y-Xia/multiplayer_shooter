use crate::custom_ws::Ws;
use crate::physics_engine::PhysicsEngine;
use actix::Addr;
use dashmap::DashMap;

pub struct PlayerInfo {}

// State holds overall application state
// It records the currently connected players
// It holds an address to the physics engine, which websocket actors will access
pub struct State {
    connected_players: DashMap<Addr<Ws>, PlayerInfo>, // Lockless!
    physics_engine_address: Addr<PhysicsEngine>,
}

impl State {
    pub fn new(physics_engine_address: Addr<PhysicsEngine>) -> State {
        State {
            connected_players: DashMap::new(),
            physics_engine_address,
        }
    }

    // Registers new websocket connection to the game server
    pub fn register(&self, address: Addr<Ws>) {
        self.connected_players.insert(address, PlayerInfo {});
    }

    // Removes websocket connection
    // Called when actor stops
    pub fn remove(&self, address: Addr<Ws>) {
        self.connected_players.remove(&address);
    }

    pub fn get_physics_engine(&self) -> &Addr<PhysicsEngine> {
        &self.physics_engine_address
    }
}
