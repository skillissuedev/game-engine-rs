use glium::{Display, Frame};
use ultraviolet::Vec3;
use std::collections::HashMap;

pub trait Object {
    fn start(&mut self) {}
    fn update(&mut self) {}
    fn opaque_render(&mut self, display: &Display, target: &mut Frame) {}
    fn transparent_render(&mut self, display: &Display, target: &mut Frame) {}

    fn get_component_type(&self) -> &str;

    fn get_data(&self) -> Option<HashMap<&str, String>> {
        None
    }

    fn get_transform() -> Transform;
    fn set_transform(transform: Transform);

    fn get_position() -> Vec3;
    fn get_rotation() -> Vec3;
    fn get_scale() -> Vec3;

    fn set_position(position: Vec3);
    fn set_rotation(rotation: Vec3);
    fn set_scale(scale: Vec3);
}

pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3
}
