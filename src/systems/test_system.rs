use crate::{managers::systems::CallList, objects::{Object, camera_position::CameraPosition, empty_object::EmptyObject, sound_emitter::{SoundEmitter, SoundEmitterType}}, assets::{sound_asset::SoundAsset, model_asset::{self, ModelAsset}}};
use super::System;

pub struct TestSystem {
    pub is_destroyed: bool,
    pub objects: Vec<Box<dyn Object>>
}

impl System for TestSystem {
    fn call(&self, _call_id: &str) { }
    fn call_mut(&mut self, _call_id: &str) { }


    fn start(&mut self) {
        let asset = ModelAsset::from_file("models/test_model.gltf");
        let no_anim_asset = ModelAsset::from_file("models/no_anim_test.gltf");
        println!("{:?}", asset);
        println!("\n\n\n");
        println!("{:?}", no_anim_asset);
    }

    fn update(&mut self) {
    }

    fn render(&mut self) { }



    fn system_id(&self) -> &str {
        "TestSystem"
    }

    fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    fn set_destroyed(&mut self, is_destroyed: bool) {
        self.is_destroyed = is_destroyed;
    }

    fn get_call_list(&self) -> CallList {
        CallList {
            immut_call: vec![],
            mut_call: vec![]
        }
    }

    fn get_objects_list(&self) -> &Vec<Box<dyn Object>> {
        &self.objects
    }
    fn get_objects_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.objects
    }
}
