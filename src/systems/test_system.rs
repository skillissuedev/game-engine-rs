use glam::Vec3;
use crate::{managers::{systems::CallList, networking::{Message, self, MessageReliability, SyncObjectMessage, MessageContents}, physics::{BodyType, BodyColliderType}, input::{self, InputEventType}}, objects::{Object, model_object::ModelObject}, assets::{model_asset::ModelAsset, shader_asset::ShaderAsset}};
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
        let mut model_object = 
            Box::new(ModelObject::new("knife_model", asset.unwrap(), None, ShaderAsset::load_default_shader().unwrap()));
        //println!("start");
        model_object.set_position(Vec3::new(0.0, 0.0, 6.0), true);

        if networking::is_server() {
            model_object.build_object_rigid_body(Some(BodyType::Dynamic(Some(BodyColliderType::Cuboid(1.0, 1.0, 1.0)))), 1.0);
        }

        self.add_object(model_object);

        input::new_bind("forward", vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::W)]);
        input::new_bind("left", vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::A)]);
        input::new_bind("backwards", vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::S)]);
        input::new_bind("right", vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::D)]);
    }

    fn update(&mut self) {
        let obj = self.find_object_mut("knife_model").unwrap();
        let obj_position = obj.get_local_transform();
        //dbg!(obj_position);

        if networking::is_server() {
            let send_result = self.send_message(MessageReliability::Reliable, Message {
                receiver: networking::MessageReceiver::Everybody,
                system_id: self.system_id().into(),
                message_id: "sync_knife".into(),
                message: MessageContents::SyncObject(SyncObjectMessage {
                    object_name: "knife_model".into(),
                    transform: obj_position,
                }),
            });
            //dbg!(send_result);
        }

        //obj.set_rotation(Vec3::new(0.0, obj_rotation.y + 0.01, 0.0));
        //println!("{:?}", get_camera_rotation());
        /*if input::is_bind_pressed("forward") {
            obj_position.position.z += 1.0;
        }
        if input::is_bind_pressed("backwards") {
            obj_position.position.z -= 1.0;
        }
        if input::is_bind_pressed("left") {
            obj_position.position.x -= 1.0;
        }
        if input::is_bind_pressed("right") {
            obj_position.position.x += 1.0;
        }
        obj.set_local_transform(obj_position);*/
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
        //dbg!(&message);
        match message.message {
            networking::MessageContents::SyncObject(sync_msg) => {
                if &sync_msg.object_name == "knife_model" {
                    //println!("object");
                    let object = self.find_object_mut("knife_model");
                    object.unwrap().set_local_transform(sync_msg.transform);
                }
            },
            networking::MessageContents::Custom(_) => (),
        }
    }
}
