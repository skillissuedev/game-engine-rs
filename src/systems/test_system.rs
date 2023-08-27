use ultraviolet::Vec3;

use crate::{managers::{systems::CallList, render}, objects::{Object, camera_position::CameraPosition, empty_object::EmptyObject, sound_emitter::{SoundEmitter, SoundEmitterType}}, assets::sound_asset::SoundAsset};
use super::System;

pub struct TestSystem {
    pub is_destroyed: bool,
    pub objects: Vec<Box<dyn Object>>
}

impl System for TestSystem {
    fn call(&self, _call_id: &str) { }
    fn call_mut(&mut self, _call_id: &str) { }


    fn start(&mut self) {
        self.add_object(Box::new(EmptyObject::new("Test Object")));
        self.get_objects_list_mut()[0].add_child(Box::new(CameraPosition::new("Position")));
        self.find_object_mut("Test Object").unwrap().set_position(Vec3::new(50.0, 0.0, 6.0));
        let mut sound_emitter = Box::new(SoundEmitter::new("Sound", &SoundAsset::from_wav("sounds/test.wav").expect("failed to create an asset"), SoundEmitterType::Positional).unwrap());
        sound_emitter.play_sound();
        sound_emitter.set_looping(true);
        sound_emitter.set_max_distance(500.0).unwrap();
        self.find_object_mut("Position").unwrap().add_child(sound_emitter);
        //self.add_object(sound_emitter);

        println!("test_object: {:?}", self.find_object_mut("Test Object"));
    }

    fn update(&mut self) {
        let camera_object = self.find_object_mut("Position").unwrap();
        let camera_pos = camera_object.get_local_transform().position;
        camera_object.set_position(Vec3::new(camera_pos.x + 0.25, camera_pos.y, camera_pos.z));
        camera_object.set_rotation(Vec3::new(0.0, 90.0, 0.0));
        self.find_object_mut("Sound").unwrap().set_rotation(Vec3::new(0.0, -90.0, 0.0));
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
