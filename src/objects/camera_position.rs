use egui_glium::egui_winit::egui::{self, TextEdit};

use super::{gen_object_id, Object, ObjectGroup, Transform};
use crate::managers::{physics::ObjectBodyParameters, render};

#[derive(Debug)]
pub struct CameraPosition {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    body: Option<ObjectBodyParameters>,
    id: u128,
    groups: Vec<ObjectGroup>,
}

impl CameraPosition {
    pub fn new(name: &str) -> Self {
        CameraPosition {
            transform: Transform::default(),
            children: vec![],
            name: name.to_string(),
            parent_transform: None,
            body: None,
            id: gen_object_id(),
            groups: vec![],
        }
    }
}

impl Object for CameraPosition {
    fn start(&mut self) {}

    fn update(&mut self) {
        let global_transform = self.global_transform();
        render::set_camera_position(global_transform.position);
        render::set_camera_rotation(global_transform.rotation);
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
        "CameraPosition"
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
        ui.label("Position:");
        ui.horizontal(|ui| {
            ui.label("x: ");
            ui.add_sized(egui::vec2(20.0, 10.0), TextEdit::singleline(&mut "123"));
            ui.label("y: ");
            ui.add_sized(egui::vec2(20.0, 10.0), TextEdit::singleline(&mut "420"));
            ui.label("z: ");
            ui.add_sized(egui::vec2(20.0, 10.0), TextEdit::singleline(&mut "69"));
        });
    }

    fn groups_list(&mut self) -> &mut Vec<super::ObjectGroup> {
        &mut self.groups
    }

    fn call(&mut self, name: &str, args: Vec<&str>) -> Option<String> {
        None
    }
}
