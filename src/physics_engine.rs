use crate::custom_ws::{GameInstruction, PhysicsInstruction, Ws};
use crate::state::InnerState;
use actix::Addr;
use actix::{Actor, AsyncContext, Context, Handler, Message};

use rapier2d::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct CustomEventHandler {
    handles_to_decrement_health: Arc<Mutex<Vec<RigidBodyHandle>>>
}
struct CustomPhysicsHooks;

#[derive(Debug, Serialize, Clone, Copy)]
pub struct Coords {
    pub x: Real,
    pub y: Real,
}

#[derive(Debug)]
pub struct EnemyInfo {
    pub ws_address: Addr<Ws>,
    pub coords: Coords,
    pub health: f32,
    pub dir: f32,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct PhysicsStateResponse {
    pub my_coords: Coords,
    pub health: f32,
    pub enemies: Vec<EnemyInfo>,
    pub bullets: Vec<Coords>,
}

pub struct PhysicsPlayerInfo {
    pub handle: RigidBodyHandle,
    pub dir: f32,
    pub bullet_cooldown: i32,
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct GameOver;

impl EventHandler for CustomEventHandler {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        colliders: &ColliderSet,
        event: CollisionEvent,
        _contact_pair: Option<&ContactPair>,
    ) {
        if let CollisionEvent::Started(handle1, handle2, _flags) = event {
            let mut v = self.handles_to_decrement_health.lock().unwrap();
            if let Some(h) = colliders.get(handle1).unwrap().parent() {
                v.push(h);
            }
            if let Some(h) = colliders.get(handle2).unwrap().parent() {
                v.push(h);
            }
        }
    }
}

impl PhysicsHooks for CustomPhysicsHooks {}

pub struct PhysicsEngine {
    gravity: Vector<Real>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    _physics_hooks: CustomPhysicsHooks,
    event_handler: CustomEventHandler,

    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,

    player_body_handles: HashMap<Addr<Ws>, PhysicsPlayerInfo>,
    bullet_handles: HashMap<RigidBodyHandle, u32>,

    state: Arc<InnerState>,
}

impl PhysicsEngine {
    pub fn new(state: Arc<InnerState>) -> Self {
        PhysicsEngine {
            gravity: vector![0.0, 0.0],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            _physics_hooks: CustomPhysicsHooks {},
            event_handler: CustomEventHandler {handles_to_decrement_health: Arc::from(Mutex::from(vec![]))},
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            player_body_handles: HashMap::new(),
            bullet_handles: HashMap::new(),
            state,
        }
    }

    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &CustomPhysicsHooks {},
            &self.event_handler,
        );
    }

    fn apply_force_from_dir(rigid_body: &mut RigidBody, direction: Vector<Real>) {
        rigid_body.apply_impulse(direction, true);
    }

    fn decrement_health(&mut self) {
        let mut v = self.event_handler.handles_to_decrement_health.lock().unwrap();
        let damage = self.state.settings.bullet_damage;
        for handle in v.iter() {
            let body = &mut self.rigid_body_set.get_mut(*handle).unwrap();
            if damage < body.user_data {
                body.user_data -= damage;
            }
        }
        v.clear();
    }

}


impl Actor for PhysicsEngine {
    type Context = Context<Self>;

    // Send state every tick
    fn started(&mut self, ctx: &mut Self::Context) {
        // Bounding box
        let (w, h) = (
            self.state.settings.arena_width,
            self.state.settings.arena_height,
        );
        for collider in [
            ColliderBuilder::cuboid(w, 0.1)
                .translation(vector![0.0, 0.0])
                .build(),
            ColliderBuilder::cuboid(w, 0.1)
                .translation(vector![0.0, h])
                .build(),
            ColliderBuilder::cuboid(w, 0.1).rotation(PI / 2.0).build(),
            ColliderBuilder::cuboid(w, 0.1)
                .rotation(PI / 2.0)
                .translation(vector![w, 0.0])
                .build(),
        ] {
            self.collider_set.insert(collider);
        }

        // Every 128th of a second, run an iteration of the physics engine and send state data to clients
        ctx.run_interval(Duration::new(0, 7812500), |s, _| {
            let to_delete: Vec<_> = s
                .bullet_handles
                .iter_mut()
                .map(|(handle, counter)| {
                    *counter += 1;
                    (handle, counter)
                })
                .filter(|(_handle, counter)| **counter > 500)
                .map(|(a, b)| (*a, *b))
                .collect();

            for (handle, _) in to_delete {
                s.rigid_body_set.remove(
                    handle,
                    &mut s.island_manager,
                    &mut s.collider_set,
                    &mut s.impulse_joint_set,
                    &mut s.multibody_joint_set,
                    true,
                );
                s.bullet_handles.remove(&handle);
            }

            // Decrement bullet cooldowns
            s.player_body_handles.iter_mut().for_each(
                |(
                    _,
                    PhysicsPlayerInfo {
                        bullet_cooldown, ..
                    },
                )| {
                    *bullet_cooldown -= 1;
                    *bullet_cooldown = 0.max(*bullet_cooldown);
                },
            );

            // Decrement health
            s.decrement_health();

            let mut to_delete: Vec<RigidBodyHandle> = vec![];

            s.step();
            for (address, PhysicsPlayerInfo { handle, .. }) in s.player_body_handles.iter() {
                let rigid_body = s.rigid_body_set.get_mut(*handle).unwrap();
                let trans = rigid_body.translation();
                
                // Game over
                if rigid_body.user_data <= 5000 {
                    address.do_send(GameOver{});
                    to_delete.push(*handle);
                    continue;
                }

                let r = PhysicsStateResponse {
                    my_coords: Coords {
                        x: trans.x,
                        y: trans.y,
                    },
                    health: health_convert(rigid_body.user_data),
                    // Iterate through all the players and register them as enemies, exluding our current address
                    enemies: (s
                        .player_body_handles
                        .iter()
                        .filter(|(inner_address, _)| *inner_address != address)
                        .map(|(inner_address, PhysicsPlayerInfo { handle, dir, .. })| {
                            let rigid_body = s.rigid_body_set.get_mut(*handle).unwrap();
                            let t = rigid_body.translation();
                            EnemyInfo {
                                coords: Coords { x: t.x, y: t.y },
                                health: health_convert(rigid_body.user_data),
                                ws_address: inner_address.clone(),
                                dir: *dir,
                            }
                        })
                        .collect()),
                    bullets: s
                        .bullet_handles
                        .iter_mut()
                        .map(|(handle, _counter)| {
                            let t = s.rigid_body_set.get_mut(*handle).unwrap().translation();
                            let c = Coords { x: t.x, y: t.y };
                            c
                        })
                        .collect(),
                };
                address.do_send(r);
            }
            for handle in to_delete.iter() {
                s.rigid_body_set.remove(
                    *handle,
                    &mut s.island_manager,
                    &mut s.collider_set,
                    &mut s.impulse_joint_set,
                    &mut s.multibody_joint_set,
                    true,
                );
                s.bullet_handles.remove(&handle);
            }
        });
    }
}

impl Handler<PhysicsInstruction> for PhysicsEngine {
    type Result = ();

    fn handle(&mut self, msg: PhysicsInstruction, _ctx: &mut Self::Context) -> Self::Result {
        match msg.game_instruction {
            // Register player body to rigid_body_set
            GameInstruction::JoinGame => {
                let mut rigid_body = RigidBodyBuilder::new(RigidBodyType::Dynamic)
                    .translation(vector![100.0, 100.0])
                    .linear_damping(self.state.settings.damping)
                    .ccd_enabled(true)
                    .build();
                rigid_body.user_data = 10000;
                let handle = self.rigid_body_set.insert(rigid_body);
                self.player_body_handles.insert(
                    msg.sent_from,
                    PhysicsPlayerInfo {
                        handle,
                        dir: 0.0,
                        bullet_cooldown: 0,
                    },
                );
                let collider = ColliderBuilder::ball(self.state.settings.ball_size)
                    .density(1.0)
                    .restitution(0.7)
                    .active_events(ActiveEvents::COLLISION_EVENTS)
                    .build();
                self.collider_set
                    .insert_with_parent(collider, handle, &mut self.rigid_body_set);
            }
            GameInstruction::ExitGame => {
                let PhysicsPlayerInfo {
                    handle,
                    dir: _mut_dir,
                    bullet_cooldown: _,
                    ..
                } = self.player_body_handles.get_mut(&msg.sent_from).unwrap();
                self.rigid_body_set.remove(
                    *handle,
                    &mut self.island_manager,
                    &mut self.collider_set,
                    &mut self.impulse_joint_set,
                    &mut self.multibody_joint_set,
                    true,
                );
                self.player_body_handles.remove(&msg.sent_from);
            }
            GameInstruction::GameAction {
                w,
                a,
                s,
                d,
                click,
                dir,
            } => {
                // TODO: Actually handle this error
                // If a game action was sent, the player_body should be registered
                let PhysicsPlayerInfo {
                    handle,
                    dir: mut_dir,
                    bullet_cooldown,
                    ..
                } = self.player_body_handles.get_mut(&msg.sent_from).unwrap();
                let rigid_body = self.rigid_body_set.get_mut(*handle).unwrap();
                let force: f32 = self.state.settings.impulse_force;
                *mut_dir = dir;

                if w {
                    PhysicsEngine::apply_force_from_dir(rigid_body, vector![0.0, -force])
                }
                if a {
                    PhysicsEngine::apply_force_from_dir(rigid_body, vector![-force, 0.0])
                }
                if s {
                    PhysicsEngine::apply_force_from_dir(rigid_body, vector![0.0, force])
                }
                if d {
                    PhysicsEngine::apply_force_from_dir(rigid_body, vector![force, 0.0])
                }

                if click && *bullet_cooldown <= 0 {
                    let dir = dir + PI / 2.0;
                    let bullet_speed = self.state.settings.bullet_speed;
                    let unit_velocity = vector![dir.cos(), dir.sin()];
                    let trans = rigid_body.translation().clone();
                    PhysicsEngine::apply_force_from_dir(rigid_body, unit_velocity * self.state.settings.impulse_force * -5.0);
                    let rigid_body = RigidBodyBuilder::new(RigidBodyType::Dynamic)
                        .translation(vector![trans.x, trans.y] + unit_velocity * 30.0)
                        .linear_damping(0.25)
                        .ccd_enabled(true)
                        .linvel(unit_velocity * bullet_speed)
                        .build();
                    let handle = self.rigid_body_set.insert(rigid_body);
                    let collider = ColliderBuilder::ball(self.state.settings.bullet_size)
                        .density(1.0)
                        .restitution(0.93)
                        .build();
                    self.collider_set.insert_with_parent(
                        collider,
                        handle,
                        &mut self.rigid_body_set,
                    );
                    self.bullet_handles.insert(handle, 0);

                    *bullet_cooldown = 25;                  
                }
            }
        }
    }
}


fn health_convert(num: u128) -> f32 {
    let n = num.max(5000);
    let n = ((n - 5000) as f32) / 5000.0;
    n
}
