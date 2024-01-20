use glam::Vec3;
use crate::{managers::{systems::CallList, networking::{Message, self, MessageReliability, SyncObjectMessage, MessageContents, MessageReceiver}, physics::{BodyType, BodyColliderType, RenderColliderType, CollisionGroups}, input::{self, InputEventType}}, objects::{Object, model_object::ModelObject, empty_object::EmptyObject, ray::Ray, trigger::Trigger, character_controller::CharacterController}, assets::{model_asset::{ModelAsset, ModelAssetError}, shader_asset::ShaderAsset}};
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
        let mut knife_model = 
            Box::new(ModelObject::new("knife_model", asset.unwrap(), None, ShaderAsset::load_default_shader().unwrap()));
        //println!("start");
        knife_model.set_position(Vec3::new(5.0, 6.0, 6.0), true);

        let mut ground_collider = Box::new(EmptyObject::new("ground_collider"));
        ground_collider.set_position(Vec3::new(0.0, -2.0, 0.0), true);

        let mut ray = Box::new(Ray::new("ray", Vec3::new(0.0, 0.0, 40.0), Some(CollisionGroups::Group3)));
        ray.set_position(Vec3::new(4.0, -1.5, 0.0), false);
        ray.set_rotation(Vec3::new(0.0, 0.0, 0.0), false);

        let trigger = Box::new(Trigger::new("trigger", None, Some(CollisionGroups::Group1 | CollisionGroups::Group3), BodyColliderType::Cuboid(10.0, 0.5, 10.0)));


        if networking::is_server() {
            knife_model.build_object_rigid_body(Some(BodyType::Dynamic(Some(BodyColliderType::Cuboid(0.2, 2.0, 0.2)))), None, 1.0, Some(CollisionGroups::Group3), None);
            ground_collider.build_object_rigid_body(Some(BodyType::Fixed(Some(BodyColliderType::Cuboid(10.0, 0.5, 10.0)))),
                None, 1.0, None, Some(CollisionGroups::Group2 | CollisionGroups::Group1 | CollisionGroups::Group3));
            let mut controller = Box::new(CharacterController::new("controller", BodyColliderType::Capsule(1.0, 2.0),
                None, Some(CollisionGroups::Group1 | CollisionGroups::Group3)).unwrap());
            controller.set_position(Vec3::new(0.0, 2.0, 0.0), false);
            self.add_object(controller);
        } else {
            knife_model.build_object_rigid_body(None, Some(RenderColliderType::Cuboid(None, None, 0.2, 2.0, 0.2, false)), 1.0, None, None);
            ground_collider.build_object_rigid_body(None, Some(RenderColliderType::Cuboid(None, None, 10.0, 0.5, 10.0, false)), 1.0, None, None);
            let capsule_model_asset = ModelAsset::from_file("models/capsule.gltf").unwrap();
            let mut controller = Box::new(ModelObject::new("controller", capsule_model_asset, None, ShaderAsset::load_default_shader().unwrap()));
            controller.set_position(Vec3::new(0.0, 2.0, 0.0), false);
            self.add_object(controller);
        }

        self.add_object(knife_model);
        self.add_object(ground_collider);
        self.add_object(ray);
        self.add_object(trigger);

        input::new_bind("forward", vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::W)]);
        input::new_bind("left", vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::A)]);
        input::new_bind("backwards", vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::S)]);
        input::new_bind("right", vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::D)]);
    }

    fn update(&mut self) {
        if networking::is_server() {
            {
                let obj = self.find_object_mut("knife_model").unwrap();
                let obj_position = obj.get_local_transform();

                let _ = self.send_message(MessageReliability::Reliable, Message {
                    receiver: networking::MessageReceiver::Everybody,
                    system_id: self.system_id().into(),
                    message_id: "sync_knife".into(),
                    message: MessageContents::SyncObject(SyncObjectMessage {
                        object_name: "knife_model".into(),
                        transform: obj_position,
                    }),
                });
            }

            let transform;
            {
                let controller = self.find_object_mut("controller").unwrap().downcast_mut::<CharacterController>().unwrap();
                controller.move_controller(Vec3::new(0.0, -1.0, 0.0));
                transform = controller.get_local_transform();
            }

            let _ = self.send_message(MessageReliability::Unreliable, Message {
                receiver: MessageReceiver::Everybody,
                system_id: "TestSystem".into(),
                message_id: "sync_controller".into(),
                message: MessageContents::SyncObject(SyncObjectMessage {
                    object_name: "controller".into(), transform
                }),
            });
        }


        //let ray = self.find_object_mut("ray").unwrap();
        //dbg!(ray.call("get_intersection_position", vec![]));


        //let trigger = self.find_object_mut("trigger").unwrap();
        //dbg!(trigger.call("is_colliding", vec![]));

        if input::is_bind_down("forward") {
            println!("forward");
            let _ = self.send_message(MessageReliability::Reliable, Message {
                receiver: networking::MessageReceiver::Everybody,
                system_id: self.system_id().into(),
                message_id: "move_controller".into(),
                message: MessageContents::Custom("forward".into()),
            });
        }

        if input::is_bind_down("backwards") {
            let _ = self.send_message(MessageReliability::Reliable, Message {
                receiver: networking::MessageReceiver::Everybody,
                system_id: self.system_id().into(),
                message_id: "move_controller".into(),
                message: MessageContents::Custom("backwards".into()),
            });
        }

        if input::is_bind_down("left") {
            let _ = self.send_message(MessageReliability::Reliable, Message {
                receiver: networking::MessageReceiver::Everybody,
                system_id: self.system_id().into(),
                message_id: "move_controller".into(),
                message: MessageContents::Custom("left".into()),
            });
        }

        if input::is_bind_down("right") {
            let _ = self.send_message(MessageReliability::Reliable, Message {
                receiver: networking::MessageReceiver::Everybody,
                system_id: self.system_id().into(),
                message_id: "move_controller".into(),
                message: MessageContents::Custom("right".into()),
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
        match message.message {
            networking::MessageContents::SyncObject(sync_msg) => {
                if &sync_msg.object_name == "knife_model" {
                    let object = self.find_object_mut("knife_model");
                    object.unwrap().set_local_transform(sync_msg.transform);
                }
                if &sync_msg.object_name == "controller" {
                    let object = self.find_object_mut("controller");
                    object.unwrap().set_local_transform(sync_msg.transform);
                }
            },
            networking::MessageContents::Custom(contents) => {
                if message.message_id == "move_controller" {
                    match contents.as_str() {
                        "forward" => self.move_controller(Vec3 { x: 0.0, y: 0.0, z: 0.01 } ),
                        "backwards" => self.move_controller(Vec3 { x: 0.0, y: 0.0, z: -0.01 } ),
                        "right" => self.move_controller(Vec3 { x: 0.01, y: 0.0, z: 0.0 } ),
                        "left" => self.move_controller(Vec3 { x: -0.01, y: 0.0, z: 0.0 } ),
                        _ => ()
                   }
                }
            },
        }
    }
}

impl TestSystem {
    fn move_controller(&mut self, direction: Vec3) {
        let transform;
        
        {
            let controller = self.find_object_mut("controller").unwrap().downcast_mut::<CharacterController>().unwrap();
            controller.move_controller(direction);
            transform = controller.get_local_transform();
        }


        println!("move");
    }
}
