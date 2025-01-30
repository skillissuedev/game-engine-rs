use glam::Vec3;

use crate::objects::{model_object::ModelObject, Object};

use super::System;

pub struct MainSystem {
    pub objects: Vec<Box<dyn Object>>
}

impl System for MainSystem {
    fn client_start(&mut self, framework: &mut crate::framework::Framework) {
        self.objects[0].set_position(framework, Vec3::new(0.0, 0.0, -10.0), true);
        self.objects[1].set_position(framework, Vec3::new(0.0, -2.0, -50.0), true);
    }

    fn server_start(&mut self, framework: &mut crate::framework::Framework) {
        todo!()
    }

    fn client_update(&mut self, framework: &mut crate::framework::Framework) {
    }

    fn server_update(&mut self, framework: &mut crate::framework::Framework) {
        todo!()
    }

    fn server_render(&mut self) {
        todo!()
    }

    fn client_render(&mut self, framework: &mut crate::framework::Framework) {
    }

    fn call(&self, call_id: &str) {
        todo!()
    }

    fn call_mut(&mut self, call_id: &str) {
        todo!()
    }

    fn objects_list(&self) -> &Vec<Box<dyn crate::objects::Object>> {
        &self.objects
    }

    fn objects_list_mut(&mut self) -> &mut Vec<Box<dyn crate::objects::Object>> {
        &mut self.objects
    }

    fn call_list(&self) -> crate::managers::systems::CallList {
        todo!()
    }

    fn system_id(&self) -> &str {
        "MainSystem"
    }

    fn is_destroyed(&self) -> bool {
        todo!()
    }

    fn set_destroyed(&mut self, is_destroyed: bool) {
        todo!()
    }

    fn reg_message(&mut self, message: crate::managers::networking::Message) {
        todo!()
    }

    fn get_value(&mut self, value_name: String) -> Option<crate::managers::systems::SystemValue> {
        todo!()
    }
}
