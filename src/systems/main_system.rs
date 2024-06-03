use super::System;
use crate::{
    assets::{model_asset::ModelAsset, shader_asset::{ShaderAsset, ShaderAssetPath}, texture_asset::TextureAsset}, framework::{get_delta_time, get_resolution}, managers::{
        input::{self, is_mouse_locked, set_mouse_locked, InputEventType}, networking::Message, physics::{BodyColliderType, BodyType}, render::{get_camera_front, get_camera_position, get_camera_right, get_camera_rotation, set_camera_position, set_camera_rotation, set_light_direction}, systems::{CallList, SystemValue}
    }, objects::{instanced_model_transform_holder::InstancedModelTransformHolder, master_instanced_model_object::MasterInstancedModelObject, model_object::ModelObject, ray::Ray, Object, Transform}
};
use egui_glium::egui_winit::egui::{Color32, ComboBox, Pos2, ScrollArea, TextEdit, Vec2, Window};
use glam::Vec3;
use rand::{thread_rng, Rng};

#[derive(Debug)]
struct Prop {
    name: String,
    model_path: String,
    texture_path: String,
    instances: Vec<Transform>
}

pub struct MainSystem {
    pub is_destroyed: bool,
    pub objects: Vec<Box<dyn Object>>,
    tile_path: String,
    texture_path: String,
    new_prop_path: String,
    new_prop_texture: String,
    new_prop_name: String,
    current_prop: String,
    props_list: Vec<Prop>,
    last_actions: Vec<Action>,
    prop_count: usize,
    randomly_place_quantity: String,
    area_center: Vec2,
    area_size: Vec2
}

#[derive(Debug, Clone)]
enum Action {
    NewModelObject {
        object_name: String,
        prop_name: String
    },
    NewInstancedObjects(String),
}

impl MainSystem {
    pub fn new() -> Self {
        MainSystem {
            is_destroyed: false,
            objects: vec![],
            tile_path: String::new(),
            texture_path: String::new(),
            new_prop_path: String::new(),
            new_prop_texture: String::new(),
            new_prop_name: String::new(),
            current_prop: String::new(),
            props_list: Vec::new(),
            last_actions: Vec::new(),
            prop_count: 0,
            randomly_place_quantity: String::new(),
            area_center: Vec2::ZERO,
            area_size: Vec2::new(200.0, 200.0),
        }
    }
}

impl System for MainSystem {
    fn ui_render(&mut self, ctx: &egui_glium::egui_winit::egui::Context) {
        let screen_center = get_resolution() / 2.0;
        ctx.debug_painter().circle(Pos2::new(screen_center.x, screen_center.y), 3.0, Color32::WHITE, (1.0, Color32::WHITE));

        let transforms_part_list: Vec<String> = self.props_list.iter().map(|prop| {
            let mut result = format!("\nconst PROP_{}_TRANSFORMS = vec![", prop.name);
            for instance in &prop.instances {
                let pos = instance.position;
                let rot = instance.rotation;
                let sc = instance.scale;

                result.push_str(
                    &format!(
                        "\n    Transform {{ position: Vec3::new({}, {}, {}), rotation: Vec3::new({}, {}, {}), scale: Vec3::new({}, {}, {}) }}",
                        pos.x, pos.y, pos.z, rot.x, rot.y, rot.z, sc.x, sc.y, sc.z,
                    )
                );
            }
            result.push_str("\n];");
            result
        }).collect();
        dbg!(&transforms_part_list);

        let spawn_prop_server_part_list: Vec<String> = self.props_list.iter().map(|prop| {
            format!(
                "\n\nfn new_prop_{}_server(tile: &mut Box<Object>, transform: Transform) {{\n    // so spawn the object here, i guess?\n    tile.add_child(Box::new(prop));\n}}",
                prop.name
            )
        }).collect();
        dbg!(&spawn_prop_server_part_list);

        let spawn_prop_client_part_list: Vec<String> = self.props_list.iter().map(|prop| {
            format!(
                "\n\nfn new_prop_{}_client(tile: &mut Box<Object>, transform: Transform, model: ModelAsset, texture: TextureAsset) {{\n    let prop = ModelObject::new(\"{}\", model, Some(texture), ShaderAsset::load_default_shader().unwrap());\n    prop.set_transform(transform);\n    // so spawn the object here, i guess?\n    tile.add_child(Box::new(prop));\n}}",
                prop.name, prop.name
            )
        }).collect();
        dbg!(&spawn_prop_client_part_list);

        let mut spawn_tile_client = format!(
            "\n\nfn spawn_tile_client(&mut self, position: Vec3) {{\n   let tile_model_asset = ModelAsset::from_gltf(\"{}\");\n    let tile_texture_asset = TextureAsset::from_file(\"{}\");\n    let tile = Box::new(ModelObject::new(\"tile\", tile_model_asset, Some(tile_texture_asset), ShaderAsset::load_default_shader().unwrap()));\n    tile.set_position(position);\n    // maybe build a body here?\n\n\n    //and do the similar thing for all of the props\n",
            self.tile_path, self.texture_path
        );

        for prop in &self.props_list {
            spawn_tile_client.push_str(&format!(
                "\n    let prop_{}_model = ModelAsset::from_gltf(\"{}\");\n    let prop_{}_texture = TextureAsset::from_file(\"{}\");\n    // USING TILE AS A PARENT!\n    for prop_transform in PROP_{}_TRANSFORMS {{\n        new_prop_{}_client(tile, prop_transform,\n            prop_{}_model.clone(), prop_{}_texture.clone()\n        );\n    }}", 
                prop.name, prop.model_path, prop.name, prop.texture_path, prop.name, prop.name, prop.name, prop.name));
        }
        spawn_tile_client.push_str("\n}");
        dbg!(&spawn_tile_client);

        let mut spawn_tile_server = format!(
            "\n\nfn spawn_tile_server(&mut self, position: Vec3) {{\n    let tile_model_asset = ModelAsset::new(\"{}\");\n    let tile = EmptyObject::new(\"tile\");\n   tile.set_position(position);\n    tile.build_object_body(/*build the trimesh static thing here*/);\n\n",
            self.tile_path
        );

        for prop in &self.props_list {
            spawn_tile_server.push_str(&format!(
                "\n    // USING TILE AS A PARENT!\n    for prop_transform in PROP_{}_TRANSFORMS {{\n        new_prop_{}_server(tile, prop_transform);\n    }}", 
                prop.name, prop.name)
            );
        }
        spawn_tile_server.push_str("\n}");
        dbg!(&spawn_tile_server);


        Window::new("generated code").show(ctx, |ui| {
            ui.heading("generated code:");
            ScrollArea::vertical().show(ui, |ui| {
                let mut generated_code = String::new();
                for i in &transforms_part_list {
                    generated_code.push_str(&i);
                }
                for i in &spawn_prop_client_part_list {
                    generated_code.push_str(&i);
                }
                for i in &spawn_prop_server_part_list {
                    generated_code.push_str(&i);
                }
                generated_code.push_str(&spawn_tile_client);
                ui.add(TextEdit::multiline(&mut generated_code).code_editor().desired_rows(20).desired_width(f32::INFINITY));
                /*ui.add(TextEdit::multiline(&mut "const PROP_NAME_TRANSFORMS = vec![
    Transform { position: .., rotation: .., scale: .. },
    Transform { position: .., rotation: .., scale: .. },
    ...
];

fn spawn_tile_server(&mut self, position: Vec3) {
    let tile_model_asset = ModelAsset::new(\"path_here\");
    let tile = EmptyObject::new(/*using our assets here*/);
    tile.set_position(position);
    tile.build_object_body(/*build the trimesh static thing here*/);


    // USING TILE AS A PARENT!
    for prop_transform in PROP_NAME_TRANSFORMS {
        new_prop_name_server(tile, prop_transform);
    }
    // and do the similar thing for all of the props
    self.add_object(tile);
}

fn spawn_tile_client(&mut self, position: Vec3) {
    let tile_model_asset = ModelAsset::new(\"path_here\");
    let tile_texture_asset = TextureAsset::new(\"path_here\");
    let tile = Box::new(ModelObject::new(/*using our assets here*/));
    tile.set_position(position);
    // maybe build a body here?


    let prop_name_model = ModelAsset::new(\"path\");
    let prop_name_texture = ModelAsset::new(\"path\");
    // USING TILE AS A PARENT!
    for prop_transform in PROP_NAME_TRANSFORMS {
        new_prop_name_client(tile, prop_transform,
            prop_name_model.clone, prop_name_texture
        );
    }
    // and do the similar thing for all of the props
}

fn new_prop_name_server(tile: &mut Box<Object>, transform: Transform) {
    // so spawn the object here, i guess?
    tile.add_child(Box::new(prop));
}

fn new_prop_name_client(tile: &mut Box<Object>, transform: Transform, model: ModelAsset, texture: TextureAsset) {
    let prop = ModelObject::new(model, texture);
    prop.set_transform(transform);
    // so spawn the object here, i guess?
    tile.add_child(Box::new(prop));
}").code_editor().desired_rows(20).desired_width(f32::INFINITY));*/
            });
        });

        Window::new("editor").show(ctx, |ui| {
            ui.label("use the RMB/Return to place the selected prop, Z to undo");
            ui.separator();
            ui.label("tile path:");
            ui.text_edit_singleline(&mut self.tile_path);
            ui.label("tile texture path:");
            ui.text_edit_singleline(&mut self.texture_path);

            if ui.button("load & create").clicked() {
                self.delete_object("tile");
                let asset = ModelAsset::from_gltf(&self.tile_path);
                match asset {
                    Ok(asset) => {
                        let texture = TextureAsset::from_file(&self.texture_path);
                        let shader_asset = ShaderAsset::load_default_shader().unwrap();
                        let mut tile;
                        match texture {
                            Ok(texture) => {
                                tile = ModelObject::new("tile", asset.clone(), Some(texture), shader_asset);
                            },
                            Err(_) => {
                                tile = ModelObject::new("tile", asset.clone(), None, shader_asset);
                            },
                        }
                        tile.set_position(Vec3::new(0.0, -100.0, 0.0), true);
                        tile.build_object_rigid_body(
                            Some(BodyType::Fixed(Some(BodyColliderType::TriangleMesh(asset)))),
                            None, 1.0, None, None
                        );
                        self.add_object(Box::new(tile));
                    },
                    Err(_) => (),
                }
            }
            //self.props_list
            ui.separator();
            ui.heading("new prop:");
            ui.label("name:");
            ui.text_edit_singleline(&mut self.new_prop_name);
            ui.label("model path:");
            ui.text_edit_singleline(&mut self.new_prop_path);
            ui.label("texture path:");
            ui.text_edit_singleline(&mut self.new_prop_texture);
            if ui.button("add to the props list").clicked() {
                self.props_list.push(Prop {
                    name: self.new_prop_name.clone(),
                    model_path: self.new_prop_path.clone(),
                    texture_path: self.new_prop_texture.clone(),
                    instances: Vec::new()
                });
            }

            if ui.button("remove from the props list").clicked() {
                let mut i: Option<usize> = None;
                for (idx, prop) in self.props_list.iter().enumerate() {
                    if prop.name == self.new_prop_name {
                        i = Some(idx);
                    }
                }

                if let Some(i) = i {
                    self.props_list.remove(i);
                }
            }

            ui.horizontal(|ui| {
                ui.label("quantity:");
                ui.text_edit_singleline(&mut self.randomly_place_quantity);
            });

            /*
             *  to do someday :)
            ui.horizontal(|ui| {
                ui.label("placement area center:");
                ui.text_edit_singleline(&mut self.area_center);
            });
            ui.horizontal(|ui| {
                ui.label("placement area size:");
                ui.text_edit_singleline(&mut self.area_size);
            });*/
            if ui.button("randomly place").clicked() {
                if let Ok(quantity) = self.randomly_place_quantity.parse::<usize>() {
                    let mut current_prop = None;
                    for prop in &self.props_list {
                        if prop.name == self.current_prop {
                            current_prop = Some(prop)
                        }
                    }

                    if let Some(current_prop) = current_prop {
                        let model_asset = ModelAsset::from_gltf(&current_prop.model_path);
                        if let Ok(model_asset) = model_asset {
                            let texture_asset = TextureAsset::from_file(&current_prop.texture_path);
                            /*let shader_asset = ShaderAsset::load_default_instanced_shader().unwrap();*/
                            let shader_asset = ShaderAsset::load_from_file(ShaderAssetPath { 
                                vertex_shader_path: "shaders/default_instanced.vert".into(), fragment_shader_path: "shaders/grass.frag".into() })
                                .unwrap();
                            let master_instance_name = format!("{}_master", current_prop.name);
                            let master_instance;
                            match texture_asset {
                                Ok(texture_asset) =>
                                    master_instance = MasterInstancedModelObject::new(&master_instance_name, model_asset, Some(texture_asset), shader_asset),
                                Err(_) => 
                                    master_instance = MasterInstancedModelObject::new(&master_instance_name, model_asset, None, shader_asset),
                            }

                            let mut instances = Vec::new();
                            let mut ray = Ray::new("instance_placer_ray", Vec3::new(0.0, -900.0, 0.0), None);
                            for _ in 0..=quantity {
                                let min_x = self.area_center.x - self.area_size.x / 2.0;
                                let max_x = self.area_center.x + self.area_size.x / 2.0;
                                let min_z = self.area_center.y - self.area_size.y / 2.0;
                                let max_z = self.area_center.y + self.area_size.y / 2.0;
                                let x = thread_rng().gen_range(min_x..max_x);
                                let z = thread_rng().gen_range(min_z..max_z);
                                ray.set_position(Vec3::new(x, 500.0, z), false);
                                if let Some(position) = ray.intersection_position() {
                                    instances.push(Transform {
                                        position,
                                        rotation: Default::default(),
                                        scale: Vec3::ONE,
                                    });
                                }
                            }
                            let prop_name = current_prop.name.clone();
                            self.last_actions.push(Action::NewInstancedObjects(prop_name.clone()));

                            let tile = self.find_object_mut("tile").unwrap();
                            tile.add_child(Box::new(master_instance));
                            let positions_holder = InstancedModelTransformHolder::new(&format!("{}_holder", prop_name), &master_instance_name, instances);
                            self.add_object(Box::new(positions_holder))
                        }
                    }
                }
            }

            ComboBox::from_label("current prop").selected_text(&self.current_prop).show_ui(ui, |ui| {
                for prop in &self.props_list {
                    if ui.selectable_label(false, &prop.name).clicked() {
                        self.current_prop = prop.name.clone().into();
                    }
                }
            });
        });
    }

    fn client_start(&mut self) {
        let ray = Ray::new("ray", Vec3::new(0.0, 0.0, 900.0), None);
        self.add_object(Box::new(ray));

        let cube_model_asset = ModelAsset::from_gltf("models/cube.gltf").unwrap();
        let cube = ModelObject::new("cube", cube_model_asset, None, ShaderAsset::load_default_shader().unwrap());
        self.add_object(Box::new(cube));

        set_camera_position(Vec3::new(0.0, 0.0, 0.0));
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
        input::new_bind(
            "lock_mouse",
            vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::L)],
        );
        input::new_bind(
            "undo",
            vec![InputEventType::Key(glium::glutin::event::VirtualKeyCode::Z)],
        );
        input::new_bind(
            "place_prop",
            vec![
                InputEventType::Key(glium::glutin::event::VirtualKeyCode::Return),
                InputEventType::Mouse(glium::glutin::event::MouseButton::Right)
            ],
        );
    }

    fn server_start(&mut self) {}
    fn server_render(&mut self) {}

    fn client_update(&mut self) {
        set_light_direction(Vec3::new(-0.2, 0.0, 0.0));

        //locking mouse
        if input::is_bind_pressed("lock_mouse") {
            set_mouse_locked(!is_mouse_locked());
        }

        // movement
        let delta_time = get_delta_time().as_secs_f32();
        let delta = input::mouse_delta();
        let camera_rotation = get_camera_rotation();

        set_camera_rotation(Vec3::new(camera_rotation.x - delta.y * 50.0 * delta_time, camera_rotation.y + delta.x * 50.0 * delta_time, camera_rotation.z));

        let speed = 420.0 * delta_time;

        let camera_front = get_camera_front();
        let camera_right = get_camera_right();
        let mut camera_position = get_camera_position();

        if input::is_bind_down("cam_up") {
            set_camera_position(Vec3::new(
                camera_position.x,
                camera_position.y + speed,
                camera_position.z,
            ));
            camera_position = get_camera_position();
        }

        if input::is_bind_down("cam_down") {
            set_camera_position(Vec3::new(
                camera_position.x,
                camera_position.y - speed,
                camera_position.z,
            ));
            camera_position = get_camera_position();
        }

        if input::is_bind_down("forward") {
            set_camera_position(camera_position + camera_front * speed);
            camera_position = get_camera_position();
        }

        if input::is_bind_down("backwards") {
            set_camera_position(camera_position - camera_front * speed);
            camera_position = get_camera_position();
        }

        if input::is_bind_down("left") {
            set_camera_position(camera_position - camera_right * speed);
            camera_position = get_camera_position();
        }

        if input::is_bind_down("right") {
            set_camera_position(camera_position + camera_right * speed);
            camera_position = get_camera_position();
        }
        if get_camera_rotation().x > 89.0 {
            let rot = get_camera_rotation();
            set_camera_rotation(Vec3::new(89.0, rot.y, rot.z));
        } else if get_camera_rotation().x < -89.0 {
            let rot = get_camera_rotation();
            set_camera_rotation(Vec3::new(-89.0, rot.y, rot.z));
        }

        // moving the ray to the camera
        let ray: &mut Ray = self.find_object_mut("ray").unwrap().downcast_mut().unwrap();
        ray.set_position(camera_position, true);
        ray.set_direction(camera_front * 900.0);
        if let Some(pos) = ray.intersection_position() {
            self.find_object_mut("cube").unwrap().set_position(pos, true);
            // placing props
            if input::is_bind_pressed("place_prop") {
                let mut current_prop = None;
                for prop in &mut self.props_list {
                    if prop.name == self.current_prop {
                        current_prop = Some(prop)
                    }
                }

                if let Some(current_prop) = current_prop {
                    let model_asset = ModelAsset::from_gltf(&current_prop.model_path);
                    if let Ok(model_asset) = model_asset {
                        let texture_asset = TextureAsset::from_file(&current_prop.texture_path);
                        self.prop_count += 1;
                        let mut prop;
                        match texture_asset {
                            Ok(texture_asset) =>
                                prop = ModelObject::new(&format!("prop{}", self.prop_count), model_asset.clone(), Some(texture_asset), ShaderAsset::load_default_shader().unwrap()),
                            Err(_) => prop = ModelObject::new(&format!("prop{}", self.prop_count), model_asset.clone(), None, ShaderAsset::load_default_shader().unwrap()),
                        }
                        self.last_actions.push(Action::NewModelObject {
                            object_name: prop.name().into(),
                            prop_name: current_prop.name.clone()
                        });
                        prop.build_object_rigid_body(Some(BodyType::Fixed(Some(BodyColliderType::TriangleMesh(model_asset)))), None, 1.0, None, None);
                        prop.set_position(pos, true);
                        current_prop.instances.push(prop.local_transform());
                        self.add_object(Box::new(prop));
                    }
                }
            }
        }
        if input::is_bind_pressed("undo") {
            if self.last_actions.len() > 0 {
                let idx = self.last_actions.len() - 1;
                match &self.last_actions[idx].clone() {
                    Action::NewModelObject { object_name, prop_name } => {
                        let object = self.find_object(&object_name);
                        if let Some(object) = object {
                            let transform = object.local_transform();

                            for prop in &mut self.props_list {
                                if prop.name == *prop_name {
                                    prop.instances.retain(|x| *x != transform);
                                    self.delete_object(&object_name);
                                    self.last_actions.remove(idx);
                                    return
                                }
                            }
                        }
                    },
                    Action::NewInstancedObjects(name) => {
                        self.delete_object(&format!("{}_master", name));
                        self.delete_object(&format!("{}_holder", name));
                        self.last_actions.remove(idx);
                    },
                }
            }
        }
    }

    fn server_update(&mut self) {}

    fn client_render(&mut self) {}

    fn call(&self, _: &str) {}

    fn call_mut(&mut self, _: &str) {}

    fn objects_list(&self) -> &Vec<Box<dyn Object>> {
        &self.objects
    }

    fn objects_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.objects
    }

    fn call_list(&self) -> CallList {
        CallList { immut_call: Vec::new(), mut_call: Vec::new() }
    }

    fn system_id(&self) -> &str {
        "MainSystem"
    }

    fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    fn set_destroyed(&mut self, is_destroyed: bool) {
        self.is_destroyed = is_destroyed
    }

    fn reg_message(&mut self, _message: Message) {
    }

    fn get_value(&mut self, _value_name: String) -> Option<SystemValue> {
        None
    }
}

