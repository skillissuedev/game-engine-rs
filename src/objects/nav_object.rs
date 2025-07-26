use std::collections::HashMap;

use egui_glium::egui_winit::egui;
use glam::Vec3;
use super::{gen_object_id, Object, ObjectGroup, Transform};
use crate::{framework::Framework, managers::{assets::ModelAssetId, physics::ObjectBodyParameters}};

#[derive(Debug)]
pub struct NavObject {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    body: Option<ObjectBodyParameters>,
    id: u128,
    groups: Vec<ObjectGroup>,
    object_properties: HashMap<String, Vec<crate::managers::systems::SystemValue>>,
    data: NavObjectData,
    past_transform: Transform,
    is_started: bool,
}

#[derive(Debug, Clone)]
pub enum NavObjectData {
    StaticMesh(ModelAssetId),
    DynamicCapsule(f32),
}

impl NavObject {
    pub fn new(name: &str, data: NavObjectData) -> Self {
        NavObject {
            transform: Transform::default(),
            children: vec![],
            name: name.to_string(),
            parent_transform: None,
            body: None,
            id: gen_object_id(),
            groups: vec![],
            object_properties: HashMap::new(),
            data,
            past_transform: Transform::default(),
            is_started: false,
        }
    }
}

impl Object for NavObject {
    fn start(&mut self) {
    }

    fn update(&mut self, framework: &mut Framework) {
        // if past transform != current transform,
        // update the whole thing in the manager xd.
        // also update it if the object just appeared
        // and when it's deleted
        if self.is_started == false {
            framework.navigation.add_object(&framework.assets, *self.object_id(), self.data.clone(), self.global_transform());
            self.past_transform = self.transform;
            self.is_started = true;
            return;
        }

        if self.past_transform != self.transform {
            match &self.data {
                NavObjectData::StaticMesh(_) => {
                    framework.navigation.set_island_transform(*self.object_id(), self.global_transform());
                },
                NavObjectData::DynamicCapsule(_) => {
                    framework.navigation.set_character_position(*self.object_id(), self.global_transform().position);
                },
            }

            self.past_transform = self.transform;
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
        "NavObject"
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
        ui.heading("Some test UI stuff going on rn!");
    }

    fn groups_list(&mut self) -> &mut Vec<super::ObjectGroup> {
        &mut self.groups
    }

    fn call(&mut self, name: &str, args: Vec<&str>) -> Option<String> {
        if name == "test" {
            println!("test message {}", args[0])
        }
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
