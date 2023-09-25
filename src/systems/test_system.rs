use ultraviolet::Vec3;
use crate::{managers::{systems::CallList, render::get_camera_rotation}, objects::{Object, model_object::ModelObject}, assets::{model_asset::ModelAsset, shader_asset::ShaderAsset}};
use super::System;

pub struct TestSystem {
    pub is_destroyed: bool,
    pub objects: Vec<Box<dyn Object>>
}

impl System for TestSystem {
    fn call(&self, _call_id: &str) { }
    fn call_mut(&mut self, _call_id: &str) { }


    fn start(&mut self) {
        //let asset = ModelAsset::from_file("models/test_model.gltf");
        let asset = ModelAsset::from_file("models/skeleton_test.gltf");
        let mut model_object = Box::new(ModelObject::new("cool hot", asset.unwrap(), None, ShaderAsset::load_default_shader().unwrap()));
        model_object.set_position(Vec3::new(3.0, 0.0, 0.0));
        model_object.set_rotation(Vec3::new(0.0, 0.0, 0.0));
        self.add_object(model_object);
        //self.find_object_mut("cool hot").unwrap().call("play_animation", vec!["CubeAction"]);
        //self.find_object_mut("cool hot").unwrap().call("set_looping", vec!["true"]);
    }

    fn update(&mut self) {
        let obj = self.find_object_mut("cool hot").unwrap();
        obj.set_rotation(Vec3::new(0.0, 0.0, 0.0));
        //println!("{:?}", get_camera_rotation());
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
