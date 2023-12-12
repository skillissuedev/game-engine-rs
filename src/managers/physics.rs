use nalgebra::Vector3;
use once_cell::sync::Lazy;
use rapier3d::{dynamics::{RigidBodySet, IntegrationParameters, IslandManager, ImpulseJointSet, MultibodyJointSet, CCDSolver, RigidBody, RigidBodyBuilder}, geometry::{ColliderSet, BroadPhase, NarrowPhase, ColliderType}, na::vector, pipeline::{PhysicsPipeline, QueryPipeline}};

use crate::objects::{Transform, ObjectId};

const GRAVITY: Vector3<f32> = vector![0.0, -9.81, 0.0];
static mut RIDID_BODY_SET: Lazy<RigidBodySet> = Lazy::new(|| RigidBodySet::new());
static mut COLLIDER_SET: Lazy<ColliderSet> = Lazy::new(|| ColliderSet::new());
static INTEGRATION_PARAMETERS: Lazy<IntegrationParameters> = Lazy::new(|| IntegrationParameters::default());
static mut PHYSICS_PIPELINE: Lazy<PhysicsPipeline> = Lazy::new(|| PhysicsPipeline::new());
static mut ISLAND_MANAGER: Lazy<IslandManager> = Lazy::new(|| IslandManager::new());
static mut BROAD_PHASE: Lazy<BroadPhase> = Lazy::new(|| BroadPhase::new());
static mut NARROW_PHASE: Lazy<NarrowPhase> = Lazy::new(|| NarrowPhase::new());
static mut IMPULSE_JOINT_SET: Lazy<ImpulseJointSet> = Lazy::new(|| ImpulseJointSet::new());
static mut MULTIBODY_JOINT_SET: Lazy<MultibodyJointSet> = Lazy::new(|| MultibodyJointSet::new());
static mut CCD_SOLVER: Lazy<CCDSolver> = Lazy::new(|| CCDSolver::new());
static mut QUERY_PIPELINE: Lazy<QueryPipeline> = Lazy::new(|| QueryPipeline::new());

static mut LAST_BODY_ID: u128 = 0;

pub fn update() {
    unsafe {
        PHYSICS_PIPELINE.step(
            &GRAVITY, 
            &INTEGRATION_PARAMETERS, 
            &mut ISLAND_MANAGER, 
            &mut BROAD_PHASE,
            &mut NARROW_PHASE,
            &mut RIDID_BODY_SET,
            &mut COLLIDER_SET,
            &mut IMPULSE_JOINT_SET, 
            &mut MULTIBODY_JOINT_SET,
            &mut CCD_SOLVER,
            Some(&mut QUERY_PIPELINE),
            &(), &());
        QUERY_PIPELINE.update(&mut RIDID_BODY_SET, &mut COLLIDER_SET);
    }
}

pub fn add_rigidbody(body_type: BodyType, id: ObjectId, transform: Option<Transform>) {
    let mut collider_type: Option<BodyColliderType> = None;
    let rigid_body_builder = match body_type {
        BodyType::Fixed(collider) => {
            collider_type = collider;
            RigidBodyBuilder::fixed()
        },
        BodyType::Dynamic(collider) => {
            collider_type = collider;
            RigidBodyBuilder::dynamic()
        },
        BodyType::VelocityKinematic(collider) => {
            collider_type = collider;
            RigidBodyBuilder::kinematic_velocity_based()
        },
        BodyType::PositionKinematic(collider) => {
            collider_type = collider;
            RigidBodyBuilder::kinematic_position_based()
        },
    };

    let rigid_body: RigidBody;

    match transform {
        Some(transform) => {
            rigid_body = rigid_body_builder
                .translation(transform.position.into())
                .rotation(transform.rotation.into())
                .user_data(id.raw())
                .build();
        },
        None => {
            rigid_body = rigid_body_builder
                .user_data(id.raw())
                .build();
        },
    }


    todo!()
}

pub enum BodyType {
    Fixed(Option<BodyColliderType>),
    Dynamic(Option<BodyColliderType>), 
    VelocityKinematic(Option<BodyColliderType>), 
    PositionKinematic(Option<BodyColliderType>), 
}

pub enum BodyColliderType {
    /// f32 is radius
    Ball(f32),
    Cuboid(f32, f32, f32),
    Capsule(f32, f32),
    Cylinder(f32, f32),
    /// first is verts position, second is indices
    TriangleMesh(Vec<[f32; 3]>, Vec<u32>)
}

