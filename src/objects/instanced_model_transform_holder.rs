use super::{gen_object_id, Object, ObjectGroup, Transform};
use crate::{managers::{
    physics::ObjectBodyParameters,
    render::{self, Cascades, ShadowTextures},
}, math_utils::deg_vec_to_rad};
use glam::{Mat4, Quat};
use glium::{
    framebuffer::SimpleFrameBuffer, Display
};

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
    mats: Vec<Mat4>
}

impl InstancedModelTransformHolder {
    pub fn new(
        name: &str,
        instance: &str,
        transforms: Vec<Transform>
    ) -> Self {

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
            rotation_vector.x, rotation_vector.y, rotation_vector.z
        );

        let transform =
            Mat4::from_scale_rotation_translation(scale_vector, rotation_quat, translation_vector);
        transform
    }

    pub fn set_transforms(&mut self, transforms: Vec<Transform>) {
        self.mats = Self::transforms_to_mats(transforms);
    }

    fn transforms_to_mats(transforms: Vec<Transform>) -> Vec<Mat4> { 
        transforms.iter().map(|tr| Self::setup_mat(tr)).collect::<Vec<Mat4>>()
    }
}

impl Object for InstancedModelTransformHolder {
    fn start(&mut self) {}

    fn update(&mut self) {
        render::add_instance_positions_vec(&self.instance, &self.mats);
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

    fn inspector_ui(&mut self, ui: &mut egui_glium::egui_winit::egui::Ui) {
        ui.heading("InstancedModelObject parameters");
        ui.label(format!("instance: {}", self.instance));
    }

    fn groups_list(&mut self) -> &mut Vec<ObjectGroup> {
        &mut self.groups
    }

    fn render(&mut self, _: &Display, _: &mut glium::Frame, _: &Cascades, _: &ShadowTextures) {}

    fn shadow_render(&mut self, _: &Mat4, _: &Display, _: &mut SimpleFrameBuffer) {}
}

#[derive(Debug)]
pub struct SetupMatrixResult {
    pub mvp: Mat4,
    pub model: Mat4,
}
