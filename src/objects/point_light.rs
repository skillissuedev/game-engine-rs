use std::collections::HashMap;
use egui_glium::egui_winit::egui;
use glam::{Vec2, Vec3};
use super::{gen_object_id, Object, ObjectGroup, Transform};
use crate::{framework::Framework, managers::{physics::ObjectBodyParameters, render::RenderPointLight}};

#[derive(Debug)]
pub struct PointLight {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    body: Option<ObjectBodyParameters>,
    id: u128,
    groups: Vec<ObjectGroup>,
    object_properties: HashMap<String, Vec<crate::managers::systems::SystemValue>>,
    color: Vec3,
    attenuation: Vec2,
}

impl PointLight {
    pub fn new(name: &str, color: Vec3, attenuation: Vec2) -> Self {
        PointLight {
            transform: Transform::default(),
            children: vec![],
            name: name.to_string(),
            parent_transform: None,
            body: None,
            id: gen_object_id(),
            groups: vec![],
            object_properties: HashMap::new(),
            color,
            attenuation,
        }
    }
}

impl Object for PointLight {
    fn start(&mut self) {}

    fn update(&mut self, _: &mut Framework) {}

    fn render(&mut self, framework: &mut Framework) {
        if let Some(render) = framework.render.as_mut() {
            println!("{}", self.name());
            let light = RenderPointLight(
                self.global_transform().position, self.color, self.attenuation
            );
            render.lights.push(light);
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
        "PointLight"
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

    fn inspector_ui(&mut self, _: &mut Framework, ui: &mut egui::Ui) {
        ui.heading("PointLight");
    }

    fn groups_list(&mut self) -> &mut Vec<super::ObjectGroup> {
        &mut self.groups
    }

    fn call(&mut self, _: &str, _: Vec<&str>) -> Option<String> {
        None
    }

    fn set_object_properties(&mut self, properties: HashMap<String, Vec<crate::managers::systems::SystemValue>>) {
        self.object_properties = properties.clone();
        crate::managers::systems::register_object_id_properties(self.object_id().to_owned(), properties);
    }

    fn object_properties(&self) -> &HashMap<String, Vec<crate::managers::systems::SystemValue>> {
        &self.object_properties
    }
}

impl PointLight {
    pub fn set_color(&mut self, color: Vec3) {
        self.color = color
    }

    pub fn color(&mut self) -> Vec3 {
        self.color
    }

    pub fn set_attenuation(&mut self, attenuation: Vec2) {
        self.attenuation = attenuation
    }

    pub fn attenuation(&mut self) -> Vec2 {
        self.attenuation
    }
}
