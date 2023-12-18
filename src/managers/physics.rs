use glam::{Quat, Vec3};
use nalgebra::Vector3;
use once_cell::sync::Lazy;
use rapier3d::{
    dynamics::{RigidBodySet, IntegrationParameters, IslandManager, ImpulseJointSet, MultibodyJointSet, CCDSolver, RigidBody, RigidBodyBuilder, RigidBodyHandle},
    geometry::{ColliderSet, BroadPhase, NarrowPhase, ColliderBuilder, ColliderHandle}, 
    na::vector,
    pipeline::{PhysicsPipeline, QueryPipeline},
    math::{Point, Real}
};
use crate::{objects::Transform, math_utils::{deg_vec_to_rad, rad_vec_to_deg, deg_to_rad}};
use super::debugger;


const GRAVITY: Vector3<f32> = vector![0.0, -9.81, 0.0];
static mut RIGID_BODY_SET: Lazy<RigidBodySet> = Lazy::new(|| RigidBodySet::new());
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


pub fn update() {
    unsafe {
        PHYSICS_PIPELINE.step(
            &GRAVITY, 
            &INTEGRATION_PARAMETERS, 
            &mut ISLAND_MANAGER, 
            &mut BROAD_PHASE,
            &mut NARROW_PHASE,
            &mut RIGID_BODY_SET,
            &mut COLLIDER_SET,
            &mut IMPULSE_JOINT_SET, 
            &mut MULTIBODY_JOINT_SET,
            &mut CCD_SOLVER,
            Some(&mut QUERY_PIPELINE),
            &(), &());
        QUERY_PIPELINE.update(&mut RIGID_BODY_SET, &mut COLLIDER_SET);
    }
}

pub fn remove_rigid_body(body_parameters: ObjectBodyParameters) {
    unsafe {
        RIGID_BODY_SET.remove(body_parameters.rigid_body_handle, &mut ISLAND_MANAGER, &mut COLLIDER_SET, &mut IMPULSE_JOINT_SET, &mut MULTIBODY_JOINT_SET, true);
    }
}

pub fn new_rigid_body(body_type: BodyType, transform: Option<Transform>, mass: f32) -> ObjectBodyParameters {
    let mut collider_option: Option<BodyColliderType> = None;
    let rigid_body_builder = match body_type {
        BodyType::Fixed(collider) => {
            collider_option = collider;
            RigidBodyBuilder::fixed()
        },
        BodyType::Dynamic(collider) => {
            collider_option = collider;
            RigidBodyBuilder::dynamic()
        },
        BodyType::VelocityKinematic(collider) => {
            collider_option = collider;
            RigidBodyBuilder::kinematic_velocity_based()
        },
        BodyType::PositionKinematic(collider) => {
            collider_option = collider;
            RigidBodyBuilder::kinematic_position_based()
        },
    };

    let rigid_body: RigidBody;

    match transform {
        Some(transform) => {
            rigid_body = rigid_body_builder
                .additional_mass(mass)
                .translation(transform.position.into())
                .rotation(transform.rotation.into())
                .build();
        },
        None => {
            rigid_body = rigid_body_builder
                .additional_mass(mass)
                .build();
        },
    }

    let collider_builder: ColliderBuilder;
    match collider_option {
        Some(collider) => {
            match collider {
                BodyColliderType::Ball(radius) => collider_builder = ColliderBuilder::ball(radius),
                BodyColliderType::Cuboid(x, y, z) => collider_builder = ColliderBuilder::cuboid(x, y, z),
                BodyColliderType::Capsule(radius, height) => collider_builder = ColliderBuilder::capsule_y(height / 2.0, radius),
                BodyColliderType::Cylinder(radius, height) => collider_builder = ColliderBuilder::cylinder(height / 2.0, radius),
                BodyColliderType::TriangleMesh(verts_positions, indices) => {
                    let mut positions_nalgebra: Vec<Point<Real>> = Vec::new();
                    verts_positions.iter().for_each(|pos| positions_nalgebra.push((*pos).into()));
                    collider_builder = ColliderBuilder::trimesh(positions_nalgebra, indices.into());
                },
            }
        },
        None => {
            unsafe {
                let rigid_body_handle = RIGID_BODY_SET.insert(rigid_body);
                let body_parameters = ObjectBodyParameters {
                    rigid_body_handle,
                    collider_handle: None,
                };

                return body_parameters;
            }
        },
    }

    let collider = collider_builder.build();

    unsafe {
        let rigid_body_handle = RIGID_BODY_SET.insert(rigid_body);
        let collider_handle = COLLIDER_SET.insert_with_parent(collider, rigid_body_handle, &mut RIGID_BODY_SET);
        let body_parameters = ObjectBodyParameters {
            rigid_body_handle,
            collider_handle: Some(collider_handle),
        };

        println!("pos: {:?}", RIGID_BODY_SET.get(rigid_body_handle).unwrap().translation());
        return body_parameters;
    }
}


#[derive(Debug, Clone, Copy)]
pub struct ObjectBodyParameters {
    rigid_body_handle: RigidBodyHandle,
    collider_handle: Option<ColliderHandle>
}

impl ObjectBodyParameters {
    pub fn get_rigid_body_handle(&self) -> &RigidBodyHandle {
        &self.rigid_body_handle
    }

    pub fn get_rigid_body_handle_mut(&mut self) -> &mut RigidBodyHandle {
        &mut self.rigid_body_handle
    }

    pub fn get_collider_handle(&self) -> &Option<ColliderHandle> {
        &self.collider_handle
    }

    pub fn get_collider_handle_mut(&mut self) -> &mut Option<ColliderHandle> {
        &mut self.collider_handle
    }

    pub fn set_mass(&mut self, mass: f32) {
        match get_body(&self.rigid_body_handle) {
            Some(body) => body.set_additional_mass(mass, true),
            None => debugger::error(&format!("set_mass error!\nfailed to get rigid body with handle {:?}", self.rigid_body_handle)),
        }
    }

    pub fn get_mass(&self) -> Option<f32> {
        match get_body(&self.rigid_body_handle) {
            Some(body) => Some(body.mass().into()),
            None => {
                debugger::error(&format!("get_mass error!\nfailed to get rigid body with handle {:?}", self.rigid_body_handle));
                None
            },
        }
    }

    pub fn set_collider(&mut self, collider_type: BodyColliderType) {
        match get_body(&self.rigid_body_handle) {
            Some(_) => {
                match self.collider_handle {
                    Some(collider) => {
                        unsafe {
                            COLLIDER_SET.remove(collider, &mut ISLAND_MANAGER, &mut RIGID_BODY_SET, true);
                            let new_collider = collider_type_to_collider_builder(collider_type).build();

                            self.collider_handle = 
                                Some(COLLIDER_SET.insert_with_parent(new_collider, self.rigid_body_handle, &mut RIGID_BODY_SET));
                        }
                    },
                    None => (),
                }
            },
            None => debugger::error(&format!("set_collider error!\nfailed to get rigid body with handle {:?}", self.rigid_body_handle)),
        }
    }

    pub fn set_transform(&mut self, transform: Transform) {
        let body = get_body(&self.rigid_body_handle);

        match body {
            Some(body) => {
                body.set_position(transform.position.into(), false);
                let rad_rot = deg_vec_to_rad(transform.rotation);
                body.set_rotation(Quat::from_euler(glam::EulerRot::XYZ, rad_rot.x, rad_rot.y, rad_rot.z).into(), false);
            },
            None => debugger::error(&format!("set_transform error!\nfailed to get rigid body with handle {:?}", self.rigid_body_handle)),
        }
    }
}


fn get_body(handle: &RigidBodyHandle) -> Option<&mut RigidBody> {
    unsafe {
        RIGID_BODY_SET.get_mut(*handle)
    }
}

fn collider_type_to_collider_builder(collider: BodyColliderType) -> ColliderBuilder {
    let collider_builder: ColliderBuilder;
    match collider {
        BodyColliderType::Ball(radius) => collider_builder = ColliderBuilder::ball(radius),
        BodyColliderType::Cuboid(x, y, z) => collider_builder = ColliderBuilder::cuboid(x, y, z),
        BodyColliderType::Capsule(radius, height) => collider_builder = ColliderBuilder::capsule_y(height / 2.0, radius),
        BodyColliderType::Cylinder(radius, height) => collider_builder = ColliderBuilder::cylinder(height / 2.0, radius),
        BodyColliderType::TriangleMesh(verts_positions, indices) => {
            let mut positions_nalgebra: Vec<Point<Real>> = Vec::new();
            verts_positions.iter().for_each(|pos| positions_nalgebra.push((*pos).into()));
            collider_builder = ColliderBuilder::trimesh(positions_nalgebra, indices.into());
        },
    }

    collider_builder
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
    /// x y z scale
    Cuboid(f32, f32, f32),
    /// first is radius, second is height
    Capsule(f32, f32),
    /// first is radius, second is height
    Cylinder(f32, f32),
    /// first is verts position, second is indices
    TriangleMesh(Vec<[f32; 3]>, Vec<[u32; 3]>)
}

pub fn get_body_transformations(body_parameters: ObjectBodyParameters) -> Option<(Vec3, Vec3)> {
    unsafe {
        match RIGID_BODY_SET.get(body_parameters.rigid_body_handle) {
            Some(body) => {
                let position = (*body.translation()).into();
                let rot_quat: Quat = (*body.rotation()).into();
                let rotation = rad_vec_to_deg(rot_quat.to_euler(glam::EulerRot::XYZ).into());

                Some((position, rotation))
            },
            None => {
                debugger::error(&format!("get_body_transformations error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle));
                return None;
            }
        }
    }
}

pub fn set_body_transformations(body_parameters: ObjectBodyParameters, position: Vec3, rotation: Vec3) {
    unsafe {
        match RIGID_BODY_SET.get_mut(body_parameters.rigid_body_handle) {
            Some(body) => {
                set_body_position(body_parameters, position);
                set_body_rotation(body_parameters, position);
            },
            None => debugger::error(&format!("set_body_transformations error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle)),
        }
    }
}


pub fn set_body_position(body_parameters: ObjectBodyParameters, position: Vec3) {
    unsafe {
        match RIGID_BODY_SET.get_mut(body_parameters.rigid_body_handle) {
            Some(body) => body.set_translation(position.into(), true),
            None => debugger::error(&format!("set_body_position error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle)),
        }
    }
}

pub fn get_body_position(body_parameters: ObjectBodyParameters) -> Option<Vec3> {
    unsafe {
        match RIGID_BODY_SET.get(body_parameters.rigid_body_handle) {
            Some(body) => Some((*body.translation()).into()),
            None => {
                debugger::error(&format!("get_body_position error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle));
                None
            }
        }
    }
}

pub fn set_body_rotation(body_parameters: ObjectBodyParameters, rotation_deg: Vec3) {
    unsafe {
        match RIGID_BODY_SET.get_mut(body_parameters.rigid_body_handle) {
            Some(body) => {
                let quat = Quat::from_euler(glam::EulerRot::XYZ,
                    deg_to_rad(rotation_deg.x), deg_to_rad(rotation_deg.y), deg_to_rad(rotation_deg.z));
                body.set_rotation(quat.into(), true);
            }
            None => {
                debugger::error(&format!("set_body_rotation error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle));
            }
        }
    }
}

pub fn get_body_rotation(body_parameters: ObjectBodyParameters) -> Option<Vec3> {
    unsafe {
        match RIGID_BODY_SET.get(body_parameters.rigid_body_handle) {
            Some(body) => {
                let rot_quat: Quat = (*body.rotation()).into();
                let rotation = rad_vec_to_deg(rot_quat.to_euler(glam::EulerRot::XYZ).into());

                return Some(rotation);
            }
            None => {
                debugger::error(&format!("get_body_rotation error\nfailed to get rigid body with handle {:?}", body_parameters.rigid_body_handle));
                return None;
            },
        }
    }
}


