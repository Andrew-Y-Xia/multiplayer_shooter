use actix::{Actor, Context};
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
    pub(crate) fn new() -> Self {
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
}

impl Actor for PhysicsEngine {
    type Context = Context<Self>;
}
