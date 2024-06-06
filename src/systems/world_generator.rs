use super::System;
use crate::{
    framework::{get_delta_time, set_global_system_value}, managers::{
        input::{self, is_mouse_locked, set_mouse_locked, InputEventType},
        networking::Message,
        render::{get_camera_front, get_camera_position, get_camera_right, get_camera_rotation, set_camera_position, set_camera_rotation, set_light_direction},
        systems::{CallList, SystemValue},
    }, objects::Object
};
use glam::Vec3;

pub struct WorldGenerator {
    pub is_destroyed: bool,
    pub objects: Vec<Box<dyn Object>>,
}

impl WorldGenerator {
    pub fn new() -> WorldGenerator {
        WorldGenerator {
            is_destroyed: false,
            objects: vec![],
        }
    }
}

impl System for WorldGenerator {
    fn client_start(&mut self) {
    }

    fn server_start(&mut self) {
    }

    fn client_update(&mut self) {
    }

    fn server_update(&mut self) {
    }

    fn server_render(&mut self) {}
    fn client_render(&mut self) {}

    fn call(&self, _call_id: &str) {}
    fn call_mut(&mut self, _call_id: &str) {}

    fn objects_list(&self) -> &Vec<Box<dyn Object>> {
        &self.objects
    }

    fn objects_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.objects
    }

    fn call_list(&self) -> CallList {
        CallList {
            immut_call: vec![],
            mut_call: vec![],
        }
    }

    fn system_id(&self) -> &str {
        "PlayerManager"
    }

    fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
    fn set_destroyed(&mut self, is_destroyed: bool) {
        self.is_destroyed = is_destroyed;
    }

    fn reg_message(&mut self, message: Message) {
    }

    fn get_value(&mut self, value_name: String) -> Option<SystemValue> {
        None
    }
}

