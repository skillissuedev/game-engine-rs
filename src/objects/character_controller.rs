use glam::Vec3;
use glium::Display;
use rapier3d::{control::{KinematicCharacterController, CharacterLength}, geometry::Collider, pipeline::QueryFilter};
use crate::{managers::{physics::{ObjectBodyParameters, BodyColliderType, self, CollisionGroups}, debugger}, math_utils::deg_to_rad, framework};

use super::{Object, Transform, gen_object_id};

pub struct CharacterController {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    controller: KinematicCharacterController,
    collider: Collider,
    id: u128
}

impl CharacterController {
    pub fn new(name: &str, shape: BodyColliderType, membership_groups: Option<CollisionGroups>, mask: Option<CollisionGroups>) -> Option<Self> {
        let mut controller = KinematicCharacterController::default();
        controller.max_slope_climb_angle = deg_to_rad(45.0);
        controller.up = nalgebra::Vector::y_axis();
        controller.offset = CharacterLength::Absolute(0.01);

        let mask = match mask {
            Some(mask) => mask,
            None => CollisionGroups::full(), // maybe use all if this won't work?
        };

        let membership_groups = match membership_groups {
            Some(group) => group,
            None => CollisionGroups::Group1, // maybe use all if this won't work?
        };

        let collider = physics::collider_type_to_collider_builder(shape, membership_groups, mask).build();
        
        Some(CharacterController {
            transform: Transform::default(),
            children: vec![],
            name: name.to_string(),
            parent_transform: None,
            controller,
            collider,
            id: gen_object_id()
        })
    }
}


impl Object for CharacterController {
    fn start(&mut self) { }

    fn update(&mut self) { }

    fn render(&mut self, _display: &mut Display, _target: &mut glium::Frame) { }

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

    fn groups_list(&self) -> Vec<super::ObjectGroup> {
        todo!()
    }

    fn call(&mut self, name: &str, args: Vec<&str>) -> Option<String> {
        if name == "move_controller" {
            
        }

        None
    }
}

impl CharacterController {
    pub fn move_controller(&mut self, direction: Vec3) {
        unsafe {
            let movement = self.controller.move_shape(
                framework::get_delta_time().as_secs_f32(),              // The timestep length (can be set to SimulationSettings::dt).
                &physics::RIGID_BODY_SET,        // The RigidBodySet.
                &physics::COLLIDER_SET,      // The ColliderSet.
                &physics::QUERY_PIPELINE,        // The QueryPipeline.
                self.collider.shape(), // The character’s shape.
                &self.global_transform().position.into(),   // The character’s initial position.
                direction.into(),
                QueryFilter::default(),
                |_| { }
            );

            let translation = movement.translation;
            let new_position: Vec3 = self.local_transform().position + Vec3::new(translation.x, translation.y, translation.z);
            self.set_position(new_position, false);
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
