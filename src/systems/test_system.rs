use super::System;
use crate::{
    assets::{
        model_asset::ModelAsset, shader_asset::ShaderAsset, sound_asset::SoundAsset,
        texture_asset::TextureAsset,
    },
    managers::{
        input::{self, InputEventType},
        networking::{
            self, Message, MessageContents, MessageReceiver, MessageReliability, SyncObjectMessage,
        },
        physics::{BodyColliderType, BodyType, RenderColliderType},
        render::{get_camera_position, set_camera_position, set_light_direction},
        systems::CallList,
    },
    objects::{
        character_controller::CharacterController, empty_object::EmptyObject,
        model_object::ModelObject, nav_obstacle::NavObstacle, navmesh::NavigationGround, ray::Ray,
        sound_emitter::SoundEmitter, Object,
    },
};
use ez_al::SoundSourceType;
use glam::{Vec2, Vec3};

pub struct TestSystem {
    pub is_destroyed: bool,
    pub objects: Vec<Box<dyn Object>>,
}

impl TestSystem {
    pub fn new() -> TestSystem {
        TestSystem {
            is_destroyed: false,
            objects: vec![],
        }
    }
}

impl System for TestSystem {
    fn call(&self, _call_id: &str) {}
    fn call_mut(&mut self, _call_id: &str) {}

    fn server_start(&mut self) {
        let ground_asset = ModelAsset::from_gltf("models/ground_1.gltf").unwrap();
        //let ground_nav_asset = ModelAsset::from_file("models/ground_navmesh.gltf").unwrap();
        let mut knife_model = Box::new(EmptyObject::new("knife_model"));

        let mut ground_collider = Box::new(ModelObject::new(
            "ground_collider",
            ground_asset.clone(),
            None,
            ShaderAsset::load_default_shader().unwrap(),
        ));

        ground_collider.set_position(Vec3::new(0.0, -100.0, 0.0), true);
        //ground_collider.set_rotation(Vec3::new(0.0, 180.0, 0.0), true);
        //ground_collider.set_scale(Vec3::new(1.0, 1.0, 1.0));

        knife_model.set_position(Vec3::new(0.0, 105.0, 15.0), true);
        knife_model.build_object_rigid_body(
            Some(BodyType::Dynamic(Some(BodyColliderType::Cuboid(
                0.4, 2.81, 0.2,
            )))),
            None,
            0.5,
            None,
            None,
        );
        ground_collider.build_object_rigid_body(
            Some(BodyType::Fixed(Some(BodyColliderType::TriangleMesh(
                ground_asset,
            )))),
            None,
            1.0,
            None,
            None,
        );
        /*ground_collider.build_object_rigid_body(
        Some(BodyType::Fixed(Some(BodyColliderType::Cuboid(103.0, 1.0, 102.0)))),
        None, 1.0, None, None);*/

        let mut navmesh = NavigationGround::new("ground_navmesh", Vec2::new(100.0, 100.0));
        navmesh.set_position(Vec3::new(0.0, 0.0, 0.0), false);
        ground_collider.add_child(Box::new(navmesh));
        let obstacle = NavObstacle::new("obstacle", Vec3::new(4.0, 4.0, 4.0));
        //obstacle.set_position(Vec3::new(2.0, 0.0, 4.0), false);
        knife_model.add_child(Box::new(obstacle));
        //dbg!(unsafe { navigation::MAP.clone() });

        //dbg!(ground_collider.object_id());
        let mut controller = Box::new(CharacterController::new(
            "controller",
            BodyColliderType::Capsule(1.0, 4.0),
            None,
            None,
        ));
        controller.set_position(Vec3::new(45.0, 10.0, 30.0), false);
        controller.set_scale(Vec3::new(0.25, 1.0, 0.25));
        controller.add_to_group("player");

        self.add_object(controller);
        self.add_object(knife_model);
        self.add_object(ground_collider);
    }

    fn client_start(&mut self) {
        set_camera_position(Vec3::new(0.0, 0.0, 0.0));
        let asset = ModelAsset::from_gltf("models/knife_test.gltf");
        let test_anim_asset = ModelAsset::from_gltf("models/test_anim.gltf");
        let ground_asset = ModelAsset::from_gltf("models/tile1_trees_rock.gltf").unwrap();
        let ground_texture_asset = TextureAsset::from_file("textures/biome1_rock1.png");
        let shadow_model_asset =
            ModelAsset::from_gltf("models/test_model_for_shadows.gltf").unwrap();
        let mut test_shadow_model = Box::new(ModelObject::new(
            "test_shadow_model",
            shadow_model_asset,
            None,
            ShaderAsset::load_default_shader().unwrap(),
        ));
        test_shadow_model.set_position(Vec3::new(0.0, 2.0, 25.0), false);

        let sound_asset = SoundAsset::from_wav("sounds/tone.wav").unwrap();
        let emitter =
            SoundEmitter::new("sound_emitter", &sound_asset, SoundSourceType::Positional).unwrap();
        let emitter2 =
            SoundEmitter::new("sound_emitter2", &sound_asset, SoundSourceType::Simple).unwrap();
        self.add_object(Box::new(emitter));
        self.add_object(Box::new(emitter2));
        //let ground_nav_asset = ModelAsset::from_file("models/ground_navmesh.gltf").unwrap();

        let test_anim = Box::new(ModelObject::new(
            "test_anim",
            test_anim_asset.unwrap(),
            None,
            ShaderAsset::load_default_shader().unwrap(),
        ));
        let ray = Box::new(Ray::new("ray", Vec3::Z, None));
        self.add_object(ray);

        let mut knife_model = Box::new(ModelObject::new(
            "knife_model",
            asset.unwrap(),
            None,
            ShaderAsset::load_default_shader().unwrap(),
        ));
        knife_model.set_position(Vec3::new(0.0, 6.0, 10.0), true);

        let mut ground_collider = Box::new(ModelObject::new(
            "ground_collider",
            ground_asset.clone(),
            Some(ground_texture_asset.unwrap()),
            ShaderAsset::load_default_shader().unwrap(),
        ));
        ground_collider.set_position(Vec3::new(0.0, -100.0, 0.0), true);
        //ground_collider.set_rotation(Vec3::new(0.0, 180.0, 0.0), true);
        //ground_collider.set_scale(Vec3::new(1.0, 1.0, 1.0));

        knife_model.build_object_rigid_body(
            None,
            Some(RenderColliderType::Cuboid(
                None, None, 0.4, 2.81, 0.2, false,
            )),
            1.0,
            None,
            None,
        );

        ground_collider.build_object_rigid_body(
            None,
            Some(RenderColliderType::Cuboid(
                None, None, 20.0, 1.0, 20.0, false,
            )),
            1.0,
            None,
            None,
        );

        let capsule_model_asset = ModelAsset::from_gltf("models/capsule.gltf").unwrap();
        let mut controller = Box::new(ModelObject::new(
            "controller",
            capsule_model_asset,
            None,
            ShaderAsset::load_default_shader().unwrap(),
        ));
        controller.set_position(Vec3::new(0.0, -70.0, 0.0), false);
        controller.set_scale(Vec3::new(0.25, 1.0, 0.25));
        controller.add_to_group("player");
        self.add_object(controller);
        self.add_object(test_anim);
        self.add_object(knife_model);
        self.add_object(ground_collider);
        self.add_object(test_shadow_model);

        input::new_bind(
            "forward",
            vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::W)],
        );
        input::new_bind(
            "left",
            vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::A)],
        );
        input::new_bind(
            "backwards",
            vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::S)],
        );
        input::new_bind(
            "right",
            vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::D)],
        );
        input::new_bind(
            "cam_up",
            vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::Q)],
        );
        input::new_bind(
            "cam_down",
            vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::E)],
        );
    }

    fn server_update(&mut self) {
        {
            let obj = self.find_object_mut("knife_model").unwrap();
            let obj_tr = obj.local_transform();

            //dbg!(obj_tr.position);
            let _ = self.send_message(
                MessageReliability::Reliable,
                Message {
                    receiver: networking::MessageReceiver::Everybody,
                    system_id: self.system_id().into(),
                    message_id: "sync_knife".into(),

                    message: MessageContents::SyncObject(SyncObjectMessage {
                        object_name: "knife_model".into(),
                        transform: obj_tr,
                    }),
                },
            );
        }

        {
            let obj = self.find_object("obstacle").unwrap();
            let obj_tr = obj.global_transform();

            //dbg!(obj_tr.position);
        }
        let controller = self.find_object_mut("controller");
        let controller: &mut CharacterController = controller.unwrap().downcast_mut().unwrap();
        controller.walk_to(Vec3::new(20.0, 0.0, 20.0), 0.1);

        //println!("tick");
        /*{
          let trigger: &Trigger = self.find_object("trigger").unwrap().downcast_ref().unwrap();
          let group = ObjectGroup("player".into());
        //dbg!(trigger.is_intersecting_with_group(group));
        }*/

        let transform;
        {
            let controller = self
                .find_object_mut("controller")
                .unwrap()
                .downcast_mut::<CharacterController>()
                .unwrap();
            controller.move_controller(Vec3::new(0.0, -1.0, 0.0));
            transform = controller.local_transform();
        }
        let _ = self.send_message(
            MessageReliability::Unreliable,
            Message {
                receiver: MessageReceiver::Everybody,
                system_id: "TestSystem".into(),
                message_id: "sync_controller".into(),
                message: MessageContents::SyncObject(SyncObjectMessage {
                    object_name: "controller".into(),
                    transform,
                }),
            },
        );
    }
    fn client_update(&mut self) {
        set_light_direction(Vec3::new(-0.2, 0.0, 0.0));

        if input::is_bind_down("cam_up") {
            let camera_position = get_camera_position();
            set_camera_position(Vec3::new(
                camera_position.x,
                camera_position.y + 0.05,
                camera_position.z,
            ));
        }
        if input::is_bind_down("cam_down") {
            let camera_position = get_camera_position();
            set_camera_position(Vec3::new(
                camera_position.x,
                camera_position.y - 0.05,
                camera_position.z,
            ));
        }

        if input::is_bind_down("forward") {
            let camera_position = get_camera_position();
            set_camera_position(Vec3::new(
                camera_position.x,
                camera_position.y,
                camera_position.z + 0.05,
            ));
        }

        if input::is_bind_down("backwards") {
            let camera_position = get_camera_position();
            set_camera_position(Vec3::new(
                camera_position.x,
                camera_position.y,
                camera_position.z - 0.05,
            ));
        }

        if input::is_bind_down("left") {
            let camera_position = get_camera_position();
            set_camera_position(Vec3::new(
                camera_position.x - 0.05,
                camera_position.y,
                camera_position.z,
            ));
        }

        if input::is_bind_down("right") {
            let camera_position = get_camera_position();
            set_camera_position(Vec3::new(
                camera_position.x + 0.05,
                camera_position.y,
                camera_position.z,
            ));
        }
    }

    fn client_render(&mut self) {}
    fn server_render(&mut self) {}

    fn system_id(&self) -> &str {
        "TestSystem"
    }

    fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    fn set_destroyed(&mut self, is_destroyed: bool) {
        self.is_destroyed = is_destroyed;
    }

    fn call_list(&self) -> CallList {
        CallList {
            immut_call: vec![],
            mut_call: vec![],
        }
    }

    fn objects_list(&self) -> &Vec<Box<dyn Object>> {
        &self.objects
    }
    fn objects_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.objects
    }

    fn reg_message(&mut self, message: Message) {
        match message.message {
            networking::MessageContents::SyncObject(sync_msg) => {
                dbg!(&sync_msg);
                if &sync_msg.object_name == "knife_model" {
                    let object = self.find_object_mut("knife_model");
                    object.unwrap().set_local_transform(sync_msg.transform);
                }
                if &sync_msg.object_name == "controller" {
                    let object = self.find_object_mut("controller");
                    object.unwrap().set_local_transform(sync_msg.transform);
                }
            }
            networking::MessageContents::Custom(contents) => {
                if message.message_id == "move_controller" {
                    match contents.as_str() {
                        "forward" => self.move_controller(Vec3 {
                            x: 0.0,
                            y: 0.0,
                            z: 0.01,
                        }),
                        "backwards" => self.move_controller(Vec3 {
                            x: 0.0,
                            y: 0.0,
                            z: -0.01,
                        }),
                        "right" => self.move_controller(Vec3 {
                            x: 0.01,
                            y: 0.0,
                            z: 0.0,
                        }),
                        "left" => self.move_controller(Vec3 {
                            x: -0.01,
                            y: 0.0,
                            z: 0.0,
                        }),
                        _ => (),
                    }
                }
            }
        }
    }
}

impl TestSystem {
    fn move_controller(&mut self, direction: Vec3) {
        let controller = self
            .find_object_mut("controller")
            .unwrap()
            .downcast_mut::<CharacterController>()
            .unwrap();
        controller.move_controller(direction);
    }
}
