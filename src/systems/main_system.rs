use glam::{Vec2, Vec3};

use crate::{assets::shader_asset::ShaderAsset, managers::assets::{AssetManager, ModelAssetId}, objects::{particle_system::ParticleSystem, point_light::PointLight, Object, Transform}};

use super::System;

pub struct MainSystem {
    pub objects: Vec<Box<dyn Object>>
}

impl System for MainSystem {
    fn client_start(&mut self, framework: &mut crate::framework::Framework) {
    }

    fn server_start(&mut self, framework: &mut crate::framework::Framework) {
    }

    fn client_update(&mut self, framework: &mut crate::framework::Framework) {
    }

    fn server_update(&mut self, framework: &mut crate::framework::Framework) {
    }

    fn server_render(&mut self) {
    }

    fn client_render(&mut self, framework: &mut crate::framework::Framework) {
    }

    fn call(&self, call_id: &str) {
    }

    fn call_mut(&mut self, call_id: &str) {
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
