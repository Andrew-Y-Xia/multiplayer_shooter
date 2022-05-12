use crate::custom_ws::GameAction;
use actix::{Actor, Context, Handler};
use actix_web_actors::ws::Message;
use rapier2d::prelude::*;

struct CustomEventHandler;
struct CustomPhysicsHooks;

impl EventHandler for CustomEventHandler {
    fn handle_collision_event(
        &self,
        bodies: &RigidBodySet,
        colliders: &ColliderSet,
        event: CollisionEvent,
        contact_pair: Option<&ContactPair>,
    ) {
        todo!()
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
}

impl Actor for PhysicsEngine {
    type Context = Context<Self>;
}

impl Handler<GameAction> for PhysicsEngine {
    type Result = ();

    fn handle(&mut self, msg: GameAction, ctx: &mut Self::Context) -> Self::Result {
        
    }
}
