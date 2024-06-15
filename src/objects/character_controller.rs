use super::{gen_object_id, Object, ObjectGroup, Transform};
use crate::{
    framework::{self, Framework},
    managers::{
        debugger, navigation,
        physics::{self, BodyColliderType, CollisionGroups, ObjectBodyParameters},
    },
    math_utils::deg_to_rad,
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
    last_path_point: Option<Vec3>,
}

#[derive(Debug)]
pub struct CharacterControllerMovement {
    pub target: Vec3,
    pub speed: f32,
}

impl CharacterController {
    pub fn new(
        name: &str,
        shape: BodyColliderType,
        membership_groups: Option<CollisionGroups>,
        mask: Option<CollisionGroups>,
    ) -> Self {
        let mut controller = KinematicCharacterController::default();
        controller.max_slope_climb_angle = deg_to_rad(45.0);
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

        let collider_handle = unsafe { physics::COLLIDER_SET.insert(collider) };

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
            last_path_point: None,
        }
    }
}

impl Object for CharacterController {
    fn start(&mut self) {}

    fn update(&mut self, _: &mut Framework) {
        if let Some(movement) = &self.movement {
            let pos = self.global_transform().position;
            //dbg!(self.local_transform());
            let speed = movement.speed;
            if let Some(next_pos) = self.last_path_point {
                let direction = self.get_direction(next_pos);
                self.move_controller(direction * speed);
                self.last_path_point = None;
            } else {
                let target = movement.target;
                let next_pos = navigation::find_next_path_point(
                    Vec2::new(pos.x, pos.z),
                    Vec2::new(target.x, target.z),
                );
                //dbg!(self.local_transform());
                //dbg!(next_pos);
                match next_pos {
                    Some(next_pos) => {
                        let full_pos = Vec3::new(next_pos.x, pos.y, next_pos.y);
                        let direction = self.get_direction(full_pos);
                        self.move_controller(direction * speed);
                        self.last_path_point = Some(full_pos);
                    }
                    None => {
                        //println!("done walking");
                        self.last_path_point = None;
                        self.movement = None;
                    }
                }
            }
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

    fn inspector_ui(&mut self, ui: &mut egui_glium::egui_winit::egui::Ui) {
        ui.heading("CameraPosition parameters");
        ui.label("this object type is made specifically for servers so there's noting to change here ._.");
    }
}

impl CharacterController {
    fn get_direction(&self, next_pos: Vec3) -> Vec3 {
        let global_pos = self.global_transform().position;

        let direction = global_pos - next_pos;
        let direction = direction.normalize();

        direction
    }

    pub fn move_controller(&mut self, direction: Vec3) {
        unsafe {
            let collider = physics::COLLIDER_SET.get_mut(self.collider);
            if let Some(collider) = collider {
                let global_position = self.global_transform().position;
                let direction = Vec3 {
                    x: -direction.x,
                    y: direction.y,
                    z: -direction.z,
                };

                let movement = self.controller.move_shape(
                    framework::get_delta_time().as_secs_f32(),
                    &physics::RIGID_BODY_SET,
                    &physics::COLLIDER_SET,
                    &physics::QUERY_PIPELINE,
                    collider.shape(),
                    &global_position.into(),
                    direction.into(),
                    QueryFilter::new().exclude_sensors(),
                    |_| {},
                );

                let translation = movement.translation;
                let new_position: Vec3 = self.local_transform().position
                    + Vec3::new(translation.x, translation.y, translation.z);
                self.set_position(new_position, false);
                collider.set_position(new_position.into());

                //let total_elapsed = timer.elapsed();
                //dbg!(total_elapsed);
            } else {
                debugger::error(
                    "CharacterController's move_controller error!\nfailed to get collider",
                );
            }
        }
    }

    pub fn walk_to(&mut self, target: Vec3, speed: f32) {
        let movement = CharacterControllerMovement { target, speed };
        //dbg!(&movement);
        self.movement = Some(movement);
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
