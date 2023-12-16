use glam::Vec3;
use glium::glutin::event::VirtualKeyCode;

use crate::{managers::{systems::CallList, networking::{MessageReceiver, MessageReliability, Message, MessageContents, self}, input::{self, InputEventType}, physics::{BodyType, BodyColliderType}}, objects::{Object, model_object::ModelObject}, assets::{model_asset::ModelAsset, shader_asset::ShaderAsset}};
use super::System;

pub struct TestSystem {
    pub is_destroyed: bool,
    pub objects: Vec<Box<dyn Object>>
}

impl TestSystem {
    pub fn new() -> TestSystem {
        TestSystem {
            is_destroyed: false,
            objects: vec![]
        }
    }
}

impl System for TestSystem {
    fn call(&self, _call_id: &str) { }
    fn call_mut(&mut self, _call_id: &str) { }


    fn start(&mut self) {
        //let asset = ModelAsset::from_file("models/test_model.gltf");
        let asset = ModelAsset::from_file("models/knife_test.gltf");
        let mut model_object = Box::new(ModelObject::new("knife_model", asset.unwrap(), None, ShaderAsset::load_default_shader().unwrap()));
        model_object.set_rotation(Vec3::new(0.0, 45.0, 0.0));
        model_object.set_position(Vec3::new(0.0, 0.0, 1.0));

        if networking::is_server() {
            model_object.build_object_rigid_body(Some(BodyType::Dynamic(Some(BodyColliderType::Cuboid(1.0, 1.0, 1.0)))), 1.0);
        }

        self.add_object(model_object);
    }

    fn update(&mut self) {
        /*let obj = self.find_object_mut("knife_model").unwrap();
        let obj_rotation = obj.get_global_transform().rotation;
        obj.set_rotation(Vec3::new(0.0, obj_rotation.y + 0.01, 0.0));
        //println!("{:?}", get_camera_rotation());
        if input::is_bind_pressed("send_test_msg") {
            let _ = self.send_message(MessageReliability::Reliable, Message {
                receiver: MessageReceiver::Everybody,
                system_id: self.system_id().into(),
                message_id: "nice".into(),
                message: MessageContents::Custom("msg".into()),
            });
        }*/
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

    fn reg_message(&mut self, message: Message) {

    }
}
