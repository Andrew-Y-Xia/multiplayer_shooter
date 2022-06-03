use crate::custom_ws::{GameInstruction, PhysicsInstruction, Ws};
use crate::state::{InnerState, State};
use actix::Addr;
use actix::{Actor, AsyncContext, Context, Handler, Message};
use actix_web::web;
use rapier2d::prelude::*;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use std::sync::Arc;
use std::time::Duration;

struct CustomEventHandler;
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
    pub dir: f32,
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct PhysicsStateResponse {
    pub my_coords: Coords,
    pub enemies: Vec<EnemyInfo>,
    pub bullets: Vec<Coords>,
}

pub struct PhysicsPlayerInfo {
    pub handle: RigidBodyHandle,
    pub dir: f32,
    pub bullet_cooldown: i32,
}

impl EventHandler for CustomEventHandler {
    fn handle_collision_event(
        &self,
        bodies: &RigidBodySet,
        colliders: &ColliderSet,
        event: CollisionEvent,
        contact_pair: Option<&ContactPair>,
    ) {
        // TODO
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
            event_handler: CustomEventHandler {},
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
            &CustomEventHandler {},
        );
    }

    fn apply_force_from_dir(rigid_body: &mut RigidBody, direction: Vector<Real>) {
        rigid_body.apply_impulse(direction, true);
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
            let to_deleted: Vec<_> = s
                .bullet_handles
                .iter_mut()
                .map(|(handle, counter)| {
                    *counter += 1;
                    (handle, counter)
                })
                .filter(|(_handle, counter)| **counter > 500)
                .map(|(a, b)| (*a, *b))
                .collect();

            for (handle, _) in to_deleted {
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
            s.player_body_handles.iter_mut().for_each(|(_, PhysicsPlayerInfo { bullet_cooldown, ..})| {
                *bullet_cooldown -= 1;
                *bullet_cooldown = 0.max(*bullet_cooldown);
            });

            s.step();
            for (address, PhysicsPlayerInfo { handle, .. }) in s.player_body_handles.iter() {
                let rigid_body = s.rigid_body_set.get_mut(*handle).unwrap();
                let trans = rigid_body.translation();

                let r = PhysicsStateResponse {
                    my_coords: Coords {
                        x: trans.x,
                        y: trans.y,
                    },
                    // Iterate through all the players and register them as enemies, exluding our current address
                    enemies: (s
                        .player_body_handles
                        .iter()
                        .filter(|(inner_address, _)| *inner_address != address)
                        .map(|(inner_address, PhysicsPlayerInfo { handle, dir, .. })| {
                            let t = s.rigid_body_set.get_mut(*handle).unwrap().translation();
                            EnemyInfo {
                                coords: Coords { x: t.x, y: t.y },
                                ws_address: inner_address.clone(),
                                dir: *dir,
                            }
                        })
                        .collect()),
                    bullets: s
                        .bullet_handles
                        .iter_mut()
                        .map(|(handle, counter)| {
                            let t = s.rigid_body_set.get_mut(*handle).unwrap().translation();
                            let c = Coords { x: t.x, y: t.y };
                            c
                        })
                        .collect(),
                };
                address.do_send(r);
            }
        });
    }
}

impl Handler<PhysicsInstruction> for PhysicsEngine {
    type Result = ();

    fn handle(&mut self, msg: PhysicsInstruction, ctx: &mut Self::Context) -> Self::Result {
        match msg.game_instruction {
            // Register player body to rigid_body_set
            GameInstruction::JoinGame => {
                let rigid_body = RigidBodyBuilder::new(RigidBodyType::Dynamic)
                    .translation(vector![100.0, 100.0])
                    .linear_damping(self.state.settings.damping)
                    .ccd_enabled(true)
                    .build();
                let handle = self.rigid_body_set.insert(rigid_body);
                self.player_body_handles.insert(
                    msg.sent_from,
                    PhysicsPlayerInfo {
                        handle,
                        dir: 0.0,
                        bullet_cooldown: 0,
                    },
                );
                let collider = ColliderBuilder::ball(20.0)
                    .density(1.0)
                    .restitution(0.7)
                    .build();
                self.collider_set
                    .insert_with_parent(collider, handle, &mut self.rigid_body_set);
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
                    let trans = rigid_body.translation();
                    let rigid_body = RigidBodyBuilder::new(RigidBodyType::Dynamic)
                        .translation(vector![trans.x, trans.y] + unit_velocity * 30.0)
                        .linear_damping(0.9)
                        .ccd_enabled(true)
                        .linvel(unit_velocity * bullet_speed)
                        .build();
                    let handle = self.rigid_body_set.insert(rigid_body);
                    let collider = ColliderBuilder::ball(7.0)
                        .density(1.0)
                        .restitution(0.9)
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
