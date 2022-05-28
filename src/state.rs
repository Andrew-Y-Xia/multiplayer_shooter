use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

use crate::custom_ws::Ws;
use crate::physics_engine::PhysicsEngine;
use actix::Addr;
use dashmap::DashMap;
use rapier2d::prelude::RigidBodyHandle;
use serde::Deserialize;
use serde_json;

pub struct PlayerInfo {
    pub username: String,
    pub dir: f32, // Direction that the player is facing
}

impl Default for PlayerInfo {
    fn default() -> Self {
        PlayerInfo {
            username: String::new(),
            dir: 0.0,
        }
    }
}

#[derive(Deserialize)]
pub struct Settings {
    pub arena_width: f32,
    pub arena_height: f32,
    pub max_health: i32,
    pub bullet_damage: i32,
    pub impulse_force: f32,
    pub damping: f32,
}

pub struct InnerState {
    pub connected_players: DashMap<Addr<Ws>, PlayerInfo>, // Lockless!
    pub settings: Settings,
}

impl InnerState {
    pub fn new() -> Self {
        let file = File::open(PathBuf::from("./static/settings.json")).unwrap();
        let reader = BufReader::new(file);
        let settings = serde_json::from_reader(reader).unwrap();
        InnerState {
            connected_players: DashMap::new(),
            settings,
        }
    }
}

// State holds overall application state
// It records the currently connected players
// It holds an address to the physics engine, which websocket actors will access
pub struct State {
    pub inner: Arc<InnerState>,
    physics_engine_address: Addr<PhysicsEngine>,
}

impl State {
    pub fn new(physics_engine_address: Addr<PhysicsEngine>, inner: Arc<InnerState>) -> State {
        State {
            inner: inner,
            physics_engine_address,
        }
    }

    // Registers new websocket connection to the game server
    pub fn register(&self, address: Addr<Ws>) {
        self.connected_players
            .insert(address, PlayerInfo::default());
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

impl std::ops::Deref for State {
    type Target = InnerState;
    fn deref(&self) -> &InnerState {
        &self.inner
    }
}
