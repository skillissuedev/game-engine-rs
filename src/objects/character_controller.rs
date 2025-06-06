use std::collections::HashMap;

use super::{gen_object_id, Object, ObjectGroup, Transform};
use crate::{
    framework::Framework,
    managers::{
        debugger,
        physics::{self, BodyColliderType, CollisionGroups, ObjectBodyParameters, PhysicsManager}, systems,
    },
    math_utils::{self, deg_to_rad},
};
use glam::{Vec2, Vec3};
use rapier3d::{
    control::{CharacterLength, KinematicCharacterController},
    geometry::{ActiveCollisionTypes, ColliderHandle},
    pipeline::{ActiveEvents, QueryFilter},
};

pub struct CharacterController {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    id: u128,
    groups: Vec<ObjectGroup>,
    controller: KinematicCharacterController,
    collider: ColliderHandle,
    movement: Option<CharacterControllerMovement>,
    object_properties: HashMap<String, Vec<crate::managers::systems::SystemValue>>
}

#[derive(Debug)]
pub struct CharacterControllerMovement {
    pub target: Vec3,
    pub speed: f32,
    pub path: Option<Vec<Vec2>>,
    pub path_point_index: usize,
}

impl CharacterController {
    pub fn new(
        physics: &mut PhysicsManager,
        name: &str,
        shape: BodyColliderType,
        membership_groups: Option<CollisionGroups>,
        mask: Option<CollisionGroups>,
    ) -> Self {
        let mut controller = KinematicCharacterController::default();
        controller.max_slope_climb_angle = deg_to_rad(65.0);
        controller.up = nalgebra::Vector::y_axis();
        controller.offset = CharacterLength::Absolute(0.2);

        let mask = match mask {
            Some(mask) => mask,
            None => CollisionGroups::full(), // maybe use all if this won't work?
        };

        let membership_groups = match membership_groups {
            Some(group) => group,
            None => CollisionGroups::Group1,
        };

        let id = gen_object_id();

        let collider = physics::collider_type_to_collider_builder(shape, membership_groups, mask)
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
            .user_data(id)
            .sensor(true)
            .build();

        let collider_handle = physics.collider_set.insert(collider);

        CharacterController {
            transform: Transform::default(),
            children: vec![],
            name: name.to_string(),
            parent_transform: None,
            groups: vec![],
            controller,
            collider: collider_handle,
            id,
            movement: None,
            object_properties: HashMap::new()
        }
    }
}

impl Object for CharacterController {
    fn start(&mut self) {}

    fn update(&mut self, framework: &mut Framework) {
        let mut reset_movement = false;
        let pos = self.global_transform().position;
        if let Some(movement) = &mut self.movement {
            let target = movement.target;

            if let None = movement.path {
                match framework.navigation.find_path(Vec2::new(pos.x, pos.z), Vec2::new(target.x, target.z)) {
                    Some(path) => movement.path = Some(path),
                    None => reset_movement = true,
                }
            }
            if let Some(path) = &movement.path {
                let speed = movement.speed;

                if path[movement.path_point_index].distance(Vec2::new(pos.x, pos.z)) <= 1.0 {
                    movement.path_point_index += 1;
                }

                match path.get(movement.path_point_index) {
                    Some(point) => {
                        let pos = Vec3::new(pos.x, 0.0, pos.z);
                        let direction = Self::get_direction(pos, Vec3::new(point.x, 0.0, point.y));
                        self.move_controller(framework, direction * speed);
                    },
                    None => reset_movement = true,
                }
            }
        }


        if reset_movement {
            self.movement = None;
        }
    }

    fn children_list(&self) -> &Vec<Box<dyn Object>> {
        &self.children
    }

    fn children_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.children
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn object_type(&self) -> &str {
        "CharacterController"
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn local_transform(&self) -> Transform {
        self.transform
    }

    fn set_local_transform(&mut self, transform: Transform) {
        self.transform = transform
    }

    fn parent_transform(&self) -> Option<Transform> {
        self.parent_transform
    }

    fn set_parent_transform(&mut self, transform: Transform) {
        self.parent_transform = Some(transform);
    }

    fn set_body_parameters(&mut self, _rigid_body: Option<ObjectBodyParameters>) {
        debugger::error("character controller error!\ncan't set body parameters in CharacterController objects.");
    }

    fn body_parameters(&self) -> Option<ObjectBodyParameters> {
        None
    }

    fn object_id(&self) -> &u128 {
        &self.id
    }

    fn groups_list(&mut self) -> &mut Vec<super::ObjectGroup> {
        &mut self.groups
    }

    fn call(&mut self, _name: &str, _args: Vec<&str>) -> Option<String> {
        None
    }

    fn inspector_ui(&mut self, _: &mut Framework, ui: &mut egui_glium::egui_winit::egui::Ui) {
        ui.heading("CharacterController parameters");
        ui.label("this object type is made specifically for servers so there's noting to change here ._.");
    }

    fn set_object_properties(&mut self, properties: HashMap<String, Vec<crate::managers::systems::SystemValue>>) {
        self.object_properties = properties.clone();
        crate::managers::systems::register_object_id_properties(self.object_id().to_owned(), properties);
    }

    fn object_properties(&self) -> &HashMap<String, Vec<crate::managers::systems::SystemValue>> {
        &self.object_properties
    }
}

impl CharacterController {
    fn get_direction(global_position: Vec3, next_pos: Vec3) -> Vec3 {
        let direction = -(global_position - next_pos);
        let direction = direction.normalize();

        direction
    }

    pub fn move_controller(&mut self, framework: &mut Framework, direction: Vec3) {
        let mut new_position = None;
        {
            let collider = framework.physics.collider_set.get(self.collider);
            if let Some(collider) = collider {
                let shape = collider.shape();
                let global_transform = self.global_transform();
                let global_position = global_transform.position;
                let global_rotation = global_transform.rotation;

                let direction = math_utils::rotate_vector(direction, global_rotation);

                let movement = self.controller.move_shape(
                    framework.delta_time().as_secs_f32(),
                    &framework.physics.rigid_body_set,
                    &framework.physics.collider_set,
                    &framework.physics.query_pipeline,
                    shape,
                    &global_position.into(),
                    direction.into(),
                    QueryFilter::new().exclude_sensors(),
                    |_| {},
                );

                let object_position = self.local_transform().position
                    + Vec3::new(
                        movement.translation.x,
                        movement.translation.y,
                        movement.translation.z,
                    );
                self.set_position(framework, object_position, false);
                new_position = Some(object_position);
            } else {
                debugger::error(
                    "CharacterController's move_controller error!\nfailed to get collider",
                );
            }
        }

        {
            if let Some(new_position) = new_position {
                let collider = framework.physics.collider_set.get_mut(self.collider);
                if let Some(collider) = collider {
                    collider.set_position(new_position.into());
                }
            } else {
                debugger::error(
                    "CharacterController's move_controller error!\nfailed to get collider",
                );
            }
        }
    }

    pub fn walk_to(&mut self, target: Vec3, speed: f32) {
        let movement = CharacterControllerMovement { target, speed, path: None, path_point_index: 1 };
        self.movement = Some(movement);
    }

    pub fn stop_walking(&mut self) {
        self.movement = None;
    }

    pub fn next_path_position(&self) -> Option<Vec3> {
        match &self.movement {
            Some(movement) => {
                match &movement.path {
                    Some(path) => {
                        match path.get(movement.path_point_index) {
                            Some(point) => Some(Vec3::new(point.x, 0.0, point.y)),
                            None => None,
                        }
                    },
                    None => None,
                }
            },
            None => None,
        }
    }
}

impl std::fmt::Debug for CharacterController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CharacterController")
            .field("name", &self.name())
            .field("object_type", &self.object_type())
            .field("children", &self.children_list())
            .finish()
    }
}
