use ultraviolet::Vec3;

use crate::{managers::{systems::CallList, render}, objects::{Object, camera_position::CameraPosition, empty_object::EmptyObject}};
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

        println!("test_object: {:?}", self.find_object_mut("Test Object"));
    }

    fn update(&mut self) {
        println!("camera position: {:?}", render::get_camera_position());
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
