use ez_al::sound_source::SoundSourceType;
use glium::glutin::event::VirtualKeyCode;

use crate::{managers::{systems::CallList, networking::{MessageReceiver, MessageReliability, Message}, input::{self, InputEventType}}, objects::{Object, sound_emitter::SoundEmitter}, assets::sound_asset::SoundAsset};
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
        //let asset = ModelAsset::from_file("models/knife_test.gltf");
        //let mut model_object = Box::new(ModelObject::new("cool hot", asset.unwrap(), None, ShaderAsset::load_default_shader().unwrap()));
        //model_object.set_position(Vec3::new(2.0, -3.0, 0.0));
        //model_object.set_rotation(Vec3::new(0.0, 0.0, 0.0));
        //self.add_object(model_object);
        //self.find_object_mut("cool hot").unwrap().call("play_animation", vec!["CubeAction"]);
        //self.find_object_mut("cool hot").unwrap().call("play_animation", vec!["ArmatureAction"]);
        //self.find_object_mut("cool hot").unwrap().call("set_looping", vec!["true"]);
        input::new_bind("send_test_msg", vec![InputEventType::Key(VirtualKeyCode::Return)]);
    }

    fn update(&mut self) {
        //let obj = self.find_object_mut("cool hot").unwrap();
        //let obj_position = obj.get_global_transform().position;
        //let obj_rotation = obj.get_global_transform().rotation;
        //obj.set_position(Vec3::new(2.0, obj_position.y + 0.0001, 0.0));
        //obj.set_scale(Vec3::new(2.0, 2.0, 2.0));
        //obj.set_rotation(Vec3::new(0.0, obj_rotation.y + 0.01, 0.0));
        //println!("{:?}", get_camera_rotation());
        if input::is_bind_pressed("send_test_msg") {
            self.send_message(MessageReliability::Reliable, Message {
                receiver: MessageReceiver::Everybody,
                system_id: self.system_id().into(),
                message_id: "nice".into(),
                message: "msg".into(),
            });
        }
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
        println!("{}", message.message);
    }
}
