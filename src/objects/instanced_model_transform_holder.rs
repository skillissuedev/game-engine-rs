use std::collections::HashMap;
use egui_glium::egui_winit::egui;
use glam::{Mat4, Quat};
use super::{gen_object_id, Object, ObjectGroup, Transform};
use crate::{framework::Framework, managers::physics::ObjectBodyParameters, math_utils::deg_vec_to_rad};

#[derive(Debug)]
pub struct InstancedModelTransformHolder {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    body: Option<ObjectBodyParameters>,
    id: u128,
    groups: Vec<ObjectGroup>,
    object_properties: HashMap<String, Vec<crate::managers::systems::SystemValue>>,
    master_object: String,
    transforms: Vec<Mat4>,
}

impl InstancedModelTransformHolder {
    pub fn new(name: &str, master_object: String, transforms: Vec<Transform>) -> Self {
        let object_id = gen_object_id();

        let transforms: Vec<Mat4> = transforms.iter().map(|transform| {
            let rotation = deg_vec_to_rad(transform.rotation);
            let rotation =
                Quat::from_euler(glam::EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
            let transform = Mat4::from_scale_rotation_translation(
                transform.scale, rotation, transform.position,
            );

            transform
        }).collect();

        InstancedModelTransformHolder {
            transform: Transform::default(),
            children: vec![],
            name: name.to_string(),
            parent_transform: None,
            body: None,
            id: object_id,
            groups: vec![],
            object_properties: HashMap::new(),
            master_object,
            transforms,
        }
    }
}

impl Object for InstancedModelTransformHolder {
    fn start(&mut self) {}

    fn update(&mut self, _: &mut Framework) {}

    fn render(&mut self, framework: &mut Framework) {
        if let Some(render) = framework.render.as_mut() {
            match render.instanced_positions.get_mut(&self.master_object) {
                Some(instanced_positions) => {
                    for transform in &self.transforms {
                        instanced_positions.push(*transform);
                    }
                },
                None => {
                    render.instanced_positions
                        .insert(self.master_object.clone(), self.transforms.clone());
                },
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
        "InstancedModelObject"
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
        ui.heading("InstancedModelObject");
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
