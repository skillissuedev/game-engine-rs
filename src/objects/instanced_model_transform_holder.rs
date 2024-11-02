use std::collections::HashMap;

use super::{gen_object_id, Object, ObjectGroup, Transform};
use crate::{
    framework::Framework,
    managers::{debugger, physics::ObjectBodyParameters},
    math_utils::deg_vec_to_rad,
};
use glam::{Mat4, Quat};

#[derive(Debug)]
pub struct InstancedModelTransformHolder {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    body: Option<ObjectBodyParameters>,
    id: u128,
    groups: Vec<ObjectGroup>,
    instance: String,
    mats: Vec<Mat4>,
    object_properties: HashMap<String, Vec<crate::managers::systems::SystemValue>>
}

impl InstancedModelTransformHolder {
    pub fn new(name: &str, instance: &str, transforms: Vec<Transform>) -> Self {
        InstancedModelTransformHolder {
            transform: Transform::default(),
            children: vec![],
            name: name.to_string(),
            parent_transform: None,
            groups: vec![],
            body: None,
            id: gen_object_id(),
            instance: instance.into(),
            mats: Self::transforms_to_mats(transforms),
            object_properties: HashMap::new()
        }
    }
}

impl InstancedModelTransformHolder {
    fn setup_mat(transform: &Transform) -> Mat4 {
        let rotation_vector = deg_vec_to_rad(transform.rotation);
        let mut translation_vector = transform.position;
        translation_vector.z = -translation_vector.z;
        let scale_vector = transform.scale;

        let rotation_quat = Quat::from_euler(
            glam::EulerRot::XYZ,
            rotation_vector.x,
            rotation_vector.y,
            rotation_vector.z,
        );

        let transform =
            Mat4::from_scale_rotation_translation(scale_vector, rotation_quat, translation_vector);
        transform
    }

    pub fn set_transforms(&mut self, transforms: Vec<Transform>) {
        self.mats = Self::transforms_to_mats(transforms);
    }

    fn transforms_to_mats(transforms: Vec<Transform>) -> Vec<Mat4> {
        transforms
            .iter()
            .map(|tr| Self::setup_mat(tr))
            .collect::<Vec<Mat4>>()
    }
}

impl Object for InstancedModelTransformHolder {
    fn start(&mut self) {}

    fn update(&mut self, framework: &mut Framework) {
        match &mut framework.render {
            Some(render) => render.add_instance_positions_vec(&self.instance, &self.mats),
            None => debugger::warn("InstancedModelTransformHolder is useless without render!"),
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
        "InstancedModelTransformHolder"
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

    fn inspector_ui(&mut self, _: &mut Framework, ui: &mut egui_glium::egui_winit::egui::Ui) {
        ui.heading("InstancedModelObject parameters");
        ui.label(format!("instance: {}", self.instance));
    }

    fn groups_list(&mut self) -> &mut Vec<ObjectGroup> {
        &mut self.groups
    }

    fn set_object_properties(&mut self, properties: HashMap<String, Vec<crate::managers::systems::SystemValue>>) {
        self.object_properties = properties.clone();
        crate::managers::systems::register_object_id_properties(self.object_id().to_owned(), properties);
    }

    fn object_properties(&self) -> &HashMap<String, Vec<crate::managers::systems::SystemValue>> {
        &self.object_properties
    }
}

#[derive(Debug)]
pub struct SetupMatrixResult {
    pub mvp: Mat4,
    pub model: Mat4,
}
