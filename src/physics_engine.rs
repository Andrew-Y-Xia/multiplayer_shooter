use crate::custom_ws::{GameInstruction, PhysicsInstruction, Ws};
use actix::Addr;
use actix::{Actor, ArbiterHandle, AsyncContext, Context, Handler, Message, Running};
use rapier2d::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::time::Duration;

struct CustomEventHandler;
struct CustomPhysicsHooks;

#[derive(Message, Serialize)]
#[rtype(result = "()")]
pub struct PhysicsStateResponse {
    x: Real,
    y: Real,
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
    physics_hooks: CustomPhysicsHooks,
    event_handler: CustomEventHandler,

    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,

    player_body_handles: HashMap<Addr<Ws>, RigidBodyHandle>,
}

impl PhysicsEngine {
    pub fn new() -> Self {
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
            physics_hooks: CustomPhysicsHooks {},
            event_handler: CustomEventHandler {},
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            player_body_handles: HashMap::new(),
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
        for collider in [
            ColliderBuilder::cuboid(1000.0, 0.1)
                .translation(vector![0.0, 0.0])
                .build(),
            ColliderBuilder::cuboid(1000.0, 0.1)
                .translation(vector![0.0, 500.0])
                .build(),
            ColliderBuilder::cuboid(1000.0, 0.1)
                .rotation(PI / 2.0)
                .build(),
            ColliderBuilder::cuboid(1000.0, 0.1)
                .rotation(PI / 2.0)
                .translation(vector![1000.0, 0.0])
                .build(),
        ] {
            self.collider_set.insert(collider);
        }

        ctx.run_interval(Duration::new(0, 10_000_000), |s, _| {
            s.step();
            for (address, handle) in s.player_body_handles.iter() {
                let rigid_body = s.rigid_body_set.get_mut(*handle).unwrap();
                let trans = rigid_body.translation();
                address.do_send(PhysicsStateResponse {
                    x: trans.x,
                    y: trans.y,
                });
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
                    .ccd_enabled(true)
                    .build();
                let handle = self.rigid_body_set.insert(rigid_body);
                self.player_body_handles.insert(msg.sent_from, handle);
                let collider = ColliderBuilder::ball(20.0)
                    .density(1.0)
                    .restitution(0.7)
                    .build();
                self.collider_set
                    .insert_with_parent(collider, handle, &mut self.rigid_body_set);
                println!("Joined Game");
            }
            GameInstruction::GameAction { w, a, s, d } => {
                // TODO: Actually handle this error
                // If a game action was sent, the player_body should be registered
                let handle = self.player_body_handles.get(&msg.sent_from).unwrap();
                let rigid_body = self.rigid_body_set.get_mut(*handle).unwrap();
                const FORCE: f32 = 50000.0;

                if w {
                    PhysicsEngine::apply_force_from_dir(rigid_body, vector![0.0, FORCE])
                }
                if a {
                    PhysicsEngine::apply_force_from_dir(rigid_body, vector![-FORCE, 0.0])
                }
                if s {
                    PhysicsEngine::apply_force_from_dir(rigid_body, vector![0.0, -FORCE])
                }
                if d {
                    PhysicsEngine::apply_force_from_dir(rigid_body, vector![FORCE, 0.0])
                }
            }
        }
    }
}
