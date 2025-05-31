use super::debugger;
use crate::{
    assets::model_asset::ModelAsset,
    math_utils::{deg_to_rad, deg_vec_to_rad, rad_vec_to_deg},
    objects::Transform,
};
use bitmask_enum::bitmask;
use glam::{Quat, Vec3};
use nalgebra::{Quaternion, Vector3};
use rapier3d::{
    dynamics::{
        CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
        RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
    },
    geometry::{
        ActiveCollisionTypes, ColliderBuilder, ColliderHandle, ColliderSet,
        ColliderShape, InteractionGroups, NarrowPhase, Ray,
    },
    math::{Point, Real},
    na::vector,
    pipeline::{ActiveEvents, PhysicsPipeline, QueryFilter, QueryPipeline}, prelude::DefaultBroadPhase,
};

const GRAVITY: Vector3<f32> = vector![0.0, -9.81, 0.0];

pub struct PhysicsManager {
    pub rigid_body_set: RigidBodySet, // = RigidBodySet::new()
    pub collider_set: ColliderSet,    // = Lazy::new(|| ColliderSet::new());
    pub integration_parameters: IntegrationParameters, // = Lazy::new(|| {
    //IntegrationParameters::default()
    pub physics_pipeline: PhysicsPipeline, // = Lazy::new(|| PhysicsPipeline::new());
    pub island_manager: IslandManager,     // = Lazy::new(|| IslandManager::new());
    pub broad_phase: DefaultBroadPhase,           // = Lazy::new(|| BroadPhase::new());
    pub narrow_phase: NarrowPhase,         // = Lazy::new(|| NarrowPhase::new());
    pub impulse_joint_set: ImpulseJointSet, // = Lazy::new(|| ImpulseJointSet::new());
    pub multibody_joint_set: MultibodyJointSet, // =
    //    Lazy::new(|| MultibodyJointSet::new());
    pub ccd_solver: CCDSolver,         // = Lazy::new(|| CCDSolver::new());
    pub query_pipeline: QueryPipeline, // = Lazy::new(|| QueryPipeline::new());
}

impl Default for PhysicsManager {
    fn default() -> Self {
        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
        }
    }
}

impl PhysicsManager {
    pub fn update(&mut self) {
        self.physics_pipeline.step(
            &GRAVITY,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
        self.query_pipeline
            .update(&mut self.collider_set);
    }

    pub fn remove_rigid_body(&mut self, body_parameters: &mut ObjectBodyParameters) {
        if let Some(handle) = body_parameters.rigid_body_handle {
            self.rigid_body_set.remove(
                handle,
                &mut self.island_manager,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                true,
            );
            body_parameters.rigid_body_handle = None;
        }
    }

    pub fn remove_rigid_body_by_handle(&mut self, handle: RigidBodyHandle) {
        self.rigid_body_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true,
        );
    }

    pub fn remove_collider_by_handle(&mut self, handle: ColliderHandle) {
        self.collider_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.rigid_body_set,
            false,
        );
    }

    pub fn new_rigid_body(
        &mut self,
        body_type: BodyType,
        transform: Option<Transform>,
        mass: f32,
        id: u128,
        membership_groups: Option<CollisionGroups>,
        filter_groups: Option<CollisionGroups>,
    ) -> ObjectBodyParameters {
        let collider_option: Option<BodyColliderType>;
        let rigid_body_builder = match body_type {
            BodyType::Fixed(collider) => {
                collider_option = collider;
                RigidBodyBuilder::fixed()
            }
            BodyType::Dynamic(collider) => {
                collider_option = collider;
                RigidBodyBuilder::dynamic()
            }
            BodyType::VelocityKinematic(collider) => {
                collider_option = collider;
                RigidBodyBuilder::kinematic_velocity_based()
            }
            BodyType::PositionKinematic(collider) => {
                collider_option = collider;
                RigidBodyBuilder::kinematic_position_based()
            }
        };

        let rigid_body: RigidBody;

        match transform {
            Some(transform) => {
                rigid_body = rigid_body_builder
                    .additional_mass(mass)
                    .translation(transform.position.into())
                    .rotation(deg_vec_to_rad(transform.rotation).into())
                    .user_data(id)
                    .build();
            }
            None => {
                rigid_body = rigid_body_builder
                    .additional_mass(mass)
                    .user_data(id)
                    .build();
            }
        }

        let collider_builder: ColliderBuilder;

        match collider_option {
            Some(collider) => {
                let membership_groups = match membership_groups {
                    Some(groups) => groups,
                    None => CollisionGroups::Group1,
                };

                let filter_groups = match filter_groups {
                    Some(groups) => groups,
                    None => CollisionGroups::Group1,
                };

                collider_builder =
                    collider_type_to_collider_builder(collider, membership_groups, filter_groups);
            }
            None => {
                let rigid_body_handle = self.rigid_body_set.insert(rigid_body);
                let body_parameters = ObjectBodyParameters {
                    rigid_body_handle: Some(rigid_body_handle),
                    collider_handle: None,
                    render_collider_type: None,
                };

                return body_parameters;
            }
        }

        let mut collider = collider_builder
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .active_collision_types(
                ActiveCollisionTypes::default()
                    | ActiveCollisionTypes::FIXED_FIXED
                    | ActiveCollisionTypes::DYNAMIC_FIXED
                    | ActiveCollisionTypes::DYNAMIC_DYNAMIC
                    | ActiveCollisionTypes::DYNAMIC_KINEMATIC
                    | ActiveCollisionTypes::DYNAMIC_FIXED
                    | ActiveCollisionTypes::KINEMATIC_FIXED,
            )
            .build();
        collider.user_data = id;

        let rigid_body_handle = self.rigid_body_set.insert(rigid_body);
        let collider_handle = self.collider_set.insert_with_parent(
            collider,
            rigid_body_handle,
            &mut self.rigid_body_set,
        );
        let body_parameters = ObjectBodyParameters {
            rigid_body_handle: Some(rigid_body_handle),
            collider_handle: Some(collider_handle),
            render_collider_type: None,
        };

        body_parameters
    }

    fn get_body(&mut self, handle: &RigidBodyHandle) -> Option<&mut RigidBody> {
        self.rigid_body_set.get_mut(*handle)
    }

    pub fn get_body_transformations(
        &self,
        body_parameters: ObjectBodyParameters,
    ) -> Option<(Vec3, Vec3)> {
        if let Some(body) = body_parameters.rigid_body_handle {
            match self.rigid_body_set.get(body) {
                Some(body) => {
                    let position = (*body.translation()).into();
                    let rot_quat: Quat = (*body.rotation()).into();
                    let rotation = rad_vec_to_deg(rot_quat.to_euler(glam::EulerRot::XYZ).into());

                    Some((position, rotation))
                }
                None => {
                    debugger::error(&format!(
                        "get_body_transformations error\nfailed to get rigid body with handle {:?}",
                        body_parameters.rigid_body_handle
                    ));
                    None
                }
            }
        } else {
            debugger::error("get_body_transformations error\nrigid body is None");
            None
        }
    }

    pub fn set_body_transformations(
        &mut self,
        body_parameters: ObjectBodyParameters,
        position: Vec3,
        rotation: Vec3,
    ) {
        if let Some(body) = body_parameters.rigid_body_handle {
            match self.rigid_body_set.get_mut(body) {
                Some(_) => {
                    self.set_body_position(body_parameters, position);
                    self.set_body_rotation(body_parameters, rotation);
                }
                None => debugger::error(&format!(
                    "set_body_transformations error\nfailed to get rigid body with handle {:?}",
                    body_parameters.rigid_body_handle
                )),
            }
        } else {
            debugger::error(&format!(
                "set_body_transformations error\nfailed to get rigid body with handle {:?}",
                body_parameters.rigid_body_handle
            ));
        }
    }

    pub fn set_body_position(&mut self, body_parameters: ObjectBodyParameters, position: Vec3) {
        if let Some(body) = body_parameters.rigid_body_handle {
            match self.rigid_body_set.get_mut(body) {
                Some(body) => body.set_translation(position.into(), true),
                None => debugger::error(&format!(
                    "set_body_position error\nfailed to get rigid body with handle {:?}",
                    body_parameters.rigid_body_handle
                )),
            }
        } else {
            debugger::error(&format!(
                "set_body_position error\nfailed to get rigid body with handle {:?}",
                body_parameters.rigid_body_handle
            ));
        }
    }

    pub fn set_rigidbody_rotation(&mut self, body: RigidBodyHandle, rotation_deg: Vec3) {
        match self.rigid_body_set.get_mut(body) {
            Some(body) => {
                let quat = Quat::from_euler(
                    glam::EulerRot::XYZ,
                    deg_to_rad(rotation_deg.x),
                    deg_to_rad(rotation_deg.y),
                    deg_to_rad(rotation_deg.z),
                );
                body.set_rotation(quat.into(), true);
            }
            None => debugger::error(&format!(
                "set_rigidbody_rotation error\nfailed to get rigid body with handle {:?}",
                body
            )),
        }
    }

    pub fn set_rigidbody_position(&mut self, body: RigidBodyHandle, position: Vec3) {
        let position = Vec3::new(position.x, position.y, position.z);

        match self.rigid_body_set.get_mut(body) {
            Some(body) => body.set_translation(position.into(), true),
            None => debugger::error(&format!(
                "set_rigidbody_position error\nfailed to get rigid body with handle {:?}",
                body
            )),
        }
    }

    pub fn get_body_position(&mut self, body_parameters: ObjectBodyParameters) -> Option<Vec3> {
        if let Some(body) = body_parameters.rigid_body_handle {
            match self.rigid_body_set.get(body) {
                Some(body) => {
                    Some((*body.translation()).into())
                },
                None => {
                    debugger::error(&format!(
                        "get_rigidbody_position error\nfailed to get rigid body with handle {:?}",
                        body_parameters.rigid_body_handle
                    ));
                    None
                }
            }
        } else {
            debugger::error(&format!(
                "get_body_position error\nfailed to get rigid body with handle {:?}",
                body_parameters.rigid_body_handle
            ));
            None
        }
    }

    pub fn set_body_rotation(&mut self, body_parameters: ObjectBodyParameters, rotation_deg: Vec3) {
        if let Some(body) = body_parameters.rigid_body_handle {
            match self.rigid_body_set.get_mut(body) {
                Some(body) => {
                    let quat = Quat::from_euler(
                        glam::EulerRot::XYZ,
                        deg_to_rad(rotation_deg.x),
                        deg_to_rad(rotation_deg.y),
                        deg_to_rad(rotation_deg.z),
                    );
                    body.set_rotation(quat.into(), true);
                }
                None => {
                    debugger::error(&format!(
                        "set_body_rotation error\nfailed to get rigid body with handle {:?}",
                        body_parameters.rigid_body_handle
                    ));
                }
            }
        } else {
            debugger::error(&format!(
                "set_body_rotation error\nfailed to get rigid body with handle {:?}",
                body_parameters.rigid_body_handle
            ));
        }
    }

    pub fn get_body_rotation(&mut self, body_parameters: ObjectBodyParameters) -> Option<Vec3> {
        if let Some(body) = body_parameters.rigid_body_handle {
            match self.rigid_body_set.get(body) {
                Some(body) => {
                    let rot_quat: Quat = (*body.rotation()).into();
                    let rotation = rad_vec_to_deg(rot_quat.to_euler(glam::EulerRot::XYZ).into());

                    Some(rotation)
                }
                None => {
                    debugger::error(&format!(
                        "get_body_rotation error\nfailed to get rigid body with handle {:?}",
                        body_parameters.rigid_body_handle
                    ));
                    None
                }
            }
        } else {
            debugger::error(&format!(
                "get_body_rotation error\nfailed to get rigid body with handle {:?}",
                body_parameters.rigid_body_handle
            ));
            None
        }
    }

    pub fn is_ray_intersecting(&self, ray: Ray, toi: f32, query_filter: QueryFilter) -> bool {
        if let Some(_) = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            toi,
            true,
            query_filter,
        ) {
            true
        } else {
            false
        }
    }

    pub fn get_ray_intersaction_position(
        &self,
        ray: Ray,
        toi: f32,
        query_filter: QueryFilter,
    ) -> Option<Vec3> {
        if let Some((_, toi)) = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            toi,
            true,
            query_filter,
        ) {
            let hit_point = ray.point_at(toi);
            return Some(Vec3::new(hit_point.x, hit_point.y, hit_point.z));
        }
        None
    }

    pub fn get_ray_intersaction_object_id(
        &self,
        ray: Ray,
        toi: f32,
        query_filter: QueryFilter,
    ) -> Option<u128> {
        if let Some((collider_handle, _)) = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            toi,
            true,
            query_filter,
        ) {
            return match self.collider_set.get(collider_handle) {
                Some(collider) => Some(collider.user_data),
                None => None,
            }
        }

        None
    }
}

/// To use several CollisionGroups at once, use "|" between them.
///
/// Example: `CollisionGroups::Group1 | CollisionGroups::Group3` <- using groups 1 and 3 here
#[bitmask(u32)]
pub enum CollisionGroups {
    Group1,
    Group2,
    Group3,
    Group4,
    Group5,
    Group6,
    Group7,
    Group8,
    Group9,
    Group10,
    Group11,
    Group12,
    Group13,
    Group14,
    Group15,
    Group16,
    Group17,
    Group18,
    Group19,
    Group20,
    Group21,
    Group22,
    Group23,
    Group24,
    Group25,
    Group26,
    Group27,
    Group28,
    Group29,
    Group30,
    Group31,
    Group32,
}

pub fn collider_type_to_render_collider(
    collider: &BodyColliderType,
    is_sensor: bool,
) -> Option<RenderColliderType> {
    match collider {
        BodyColliderType::Ball(radius) => {
            Some(RenderColliderType::Ball(None, None, *radius, is_sensor))
        }
        BodyColliderType::Cuboid(x, y, z) => Some(RenderColliderType::Cuboid(
            None, None, *x, *y, *z, is_sensor,
        )),
        BodyColliderType::Capsule(radius, height) => Some(RenderColliderType::Capsule(
            None, None, *radius, *height, is_sensor,
        )),
        BodyColliderType::Cylinder(radius, height) => Some(RenderColliderType::Cylinder(
            None, None, *radius, *height, is_sensor,
        )),
        BodyColliderType::TriangleMesh(_) => None,
    }
}

pub fn collider_type_to_rapier_shape(collider: &BodyColliderType) -> Option<ColliderShape> {
    match collider {
        BodyColliderType::Ball(radius) => Some(ColliderShape::ball(*radius)),
        BodyColliderType::Cuboid(x, y, z) => Some(ColliderShape::cuboid(*x, *y, *z)),
        BodyColliderType::Capsule(radius, height) => {
            Some(ColliderShape::capsule_y(*height, *radius))
        }
        BodyColliderType::Cylinder(radius, height) => {
            Some(ColliderShape::cylinder(*height, *radius))
        }
        BodyColliderType::TriangleMesh(_) => None,
    }
}

#[derive(Debug)]
pub struct RenderRay {
    pub origin: Vec3,
    pub direction: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct ObjectBodyParameters {
    pub rigid_body_handle: Option<RigidBodyHandle>,
    pub collider_handle: Option<ColliderHandle>,
    pub render_collider_type: Option<RenderColliderType>,
}

impl ObjectBodyParameters {
    pub fn empty() -> ObjectBodyParameters {
        ObjectBodyParameters {
            rigid_body_handle: None,
            collider_handle: None,
            render_collider_type: None,
        }
    }

    pub fn set_render_collider(&mut self, render_collider: Option<RenderColliderType>) {
        self.render_collider_type = render_collider;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RenderColliderType {
    /// position, rotation, f32 is radius, bool is sensor
    Ball(Option<Vec3>, Option<Vec3>, f32, bool),
    /// position, rotation, f32s are half-x, half-y and half-z size of collider, bool is sensor
    Cuboid(Option<Vec3>, Option<Vec3>, f32, f32, f32, bool),
    /// position, rotation, first is radius, second is height, bool is sensor
    Capsule(Option<Vec3>, Option<Vec3>, f32, f32, bool),
    /// position, rotation, first is radius, second is height, bool is sensor
    Cylinder(Option<Vec3>, Option<Vec3>, f32, f32, bool),
}

impl RenderColliderType {
    pub fn set_transform(&mut self, position: Vec3, rotation: Vec3) {
        match self {
            RenderColliderType::Ball(col_pos, col_rot, _, _) => {
                *col_pos = Some(position);
                *col_rot = Some(rotation);
            }
            RenderColliderType::Cuboid(col_pos, col_rot, _, _, _, _) => {
                *col_pos = Some(position);
                *col_rot = Some(rotation);
            }
            RenderColliderType::Capsule(col_pos, col_rot, _, _, _) => {
                *col_pos = Some(position);
                *col_rot = Some(rotation);
            }
            RenderColliderType::Cylinder(col_pos, col_rot, _, _, _) => {
                *col_pos = Some(position);
                *col_rot = Some(rotation);
            }
        }
    }
}
pub enum BodyType {
    Fixed(Option<BodyColliderType>),
    Dynamic(Option<BodyColliderType>),
    VelocityKinematic(Option<BodyColliderType>),
    PositionKinematic(Option<BodyColliderType>),
}

#[derive(Debug)]
pub enum BodyColliderType {
    /// f32 is radius,
    Ball(f32),
    /// f32s are half-x, half-y and half-z size of collider,
    Cuboid(f32, f32, f32),
    /// first is radius, second is height,
    Capsule(f32, f32),
    /// first is radius, second is height,
    Cylinder(f32, f32),
    /// just uses first object it finds
    TriangleMesh(ModelAsset),
}

pub fn collider_type_to_collider_builder(
    collider: BodyColliderType,
    membership_groups: CollisionGroups,
    filter_groups: CollisionGroups,
) -> ColliderBuilder {
    let mut collider_builder: ColliderBuilder = ColliderBuilder::cuboid(0.1, 0.1, 0.1);

    match collider {
        BodyColliderType::Ball(radius) => collider_builder = ColliderBuilder::ball(radius),
        BodyColliderType::Cuboid(x, y, z) => collider_builder = ColliderBuilder::cuboid(x, y, z),
        BodyColliderType::Capsule(radius, height) => {
            collider_builder = ColliderBuilder::capsule_y(height / 2.0, radius)
        }
        BodyColliderType::Cylinder(radius, height) => {
            collider_builder = ColliderBuilder::cylinder(height / 2.0, radius)
        }
        BodyColliderType::TriangleMesh(asset) => {
            let mut indices: Vec<[u32; 3]> = Vec::new();
            let mut temp_indices: Vec<u32> = Vec::new();
            let mut positions_nalgebra: Vec<Point<Real>> = Vec::new();
            // checking if the root has the mesh first, otherwise search it in the children
            if asset.root.render_data.len() >= 1 {
                for primitive in &asset.root.render_data {
                    primitive.vertices.iter().for_each(|vert| {
                        positions_nalgebra.push(
                            Vec3::new(
                                vert.position[0],// * 2.0,
                                vert.position[1],// * 2.0,
                                vert.position[2],// * 2.0,
                            )
                            .into(),
                        )
                    });
                    primitive.indices.iter().for_each(|ind| {
                        if temp_indices.len() < 3 {
                            temp_indices.push(*ind as u32);
                        } else {
                            indices.push([temp_indices[0], temp_indices[1], temp_indices[2]]);
                            temp_indices.clear();
                            temp_indices.push(*ind as u32);
                        }
                    });
                    collider_builder = ColliderBuilder::trimesh(positions_nalgebra, indices);

                    break
                }
            } else {
                for (_,object) in &asset.root.children {
                    for primitive in &object.render_data {
                        primitive.vertices.iter().for_each(|vert| {
                            positions_nalgebra.push(
                                Vec3::new(
                                    vert.position[0],// * 2.0,
                                    vert.position[1],// * 2.0,
                                    vert.position[2],// * 2.0,
                                )
                                .into(),
                            )
                        });
                        primitive.indices.iter().for_each(|ind| {
                            if temp_indices.len() < 3 {
                                temp_indices.push(*ind as u32);
                            } else {
                                indices.push([temp_indices[0], temp_indices[1], temp_indices[2]]);
                                temp_indices.clear();
                                temp_indices.push(*ind as u32);
                            }
                        });
                        break
                    }
                }
                collider_builder = ColliderBuilder::trimesh(positions_nalgebra, indices);
            }
        }
    }

    collider_builder = collider_builder.solver_groups(InteractionGroups::new(
        membership_groups.bits.into(),
        filter_groups.bits.into(),
    ));

    collider_builder
}
