use crate::{framework::Framework, managers::{
    navigation::{self, NavMeshObstacleTransform}, physics::ObjectBodyParameters,
}};
use glam::{Vec2, Vec3};

use super::{gen_object_id, Object, ObjectGroup, Transform};

#[derive(Debug)]
pub struct NavObstacle {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    body: Option<ObjectBodyParameters>,
    id: u128,
    groups: Vec<ObjectGroup>,
    size: Vec3,
}

impl NavObstacle {
    pub fn new(name: &str, size: Vec3) -> Self {
        NavObstacle {
            transform: Transform::default(),
            children: vec![],
            name: name.to_string(),
            parent_transform: None,
            body: None,
            id: gen_object_id(),
            groups: vec![],
            size,
        }
    }

    pub fn set_size(&mut self, size: Vec3) {
        self.size = size
    }
}

impl Object for NavObstacle {
    fn start(&mut self) {}

    fn update(&mut self, _: &mut Framework) {
        let position = self.global_transform().position;
        let position_xz = Vec2::new(position.x, position.z);
        let size_xz = Vec2::new(self.size.x, self.size.z);
        navigation::add_obstacle(NavMeshObstacleTransform::new(position_xz, size_xz));
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
        "NavObstacle"
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

    fn set_body_parameters(&mut self, rigid_body: Option<ObjectBodyParameters>) {
        self.body = rigid_body
    }

    fn body_parameters(&self) -> Option<ObjectBodyParameters> {
        self.body
    }

    fn object_id(&self) -> &u128 {
        &self.id
    }

    fn inspector_ui(&mut self, ui: &mut egui_glium::egui_winit::egui::Ui) {
        ui.heading("NavObstacle parameters");
        ui.label("this object type is made specifically for servers so there's noting to change here ._.");
    }

    fn groups_list(&mut self) -> &mut Vec<super::ObjectGroup> {
        &mut self.groups
    }

    fn call(&mut self, name: &str, args: Vec<&str>) -> Option<String> {
        None
    }
}
