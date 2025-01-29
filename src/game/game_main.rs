use glam::Vec3;
use glium::winit::keyboard::KeyCode;

use crate::{
    assets::{model_asset::ModelAsset, shader_asset::ShaderAsset}, framework::{DebugMode, Framework}, managers::{assets::ModelAssetId, input::InputEventType, networking, render::{RenderLayer, RenderObjectData, RenderShader}, scripting::lua::LuaSystem, systems::{add_system, SystemValue}}, objects::Transform, systems::main_system::MainSystem, Args//, systems::player_manager::PlayerManager, Args,
    //systems::player_manager::PlayerManager,
};

pub fn start(args: Args, framework: &mut Framework) {
    if let Some(save_name) = &args.new_save_name {
        framework.set_global_system_value("WorldGeneratorSeed", vec![SystemValue::UInt(args.new_save_seed.unwrap())]);
        framework.register_save_value("WorldGeneratorSeed");

        match framework.new_save(&save_name) {
            Ok(_) => println!("Successfully created a new save file!"),
            Err(err) => println!("Failed to create a new save file!\nErr: {}", err),
        }
        std::process::exit(0);
    }

    if let Some(save_name) = &args.load_save {
        if let Err(_) = framework.load_save(&save_name) {
            println!("Failed to load the save file and start the server!");
        }
    }


    framework.input.new_bind(
        "debug_toggle",
        vec![InputEventType::Key(KeyCode::Backquote)],
    );
    if networking::is_server() == false {
        framework
            .preload_texture_asset("default".into(), "textures/default_texture.png")
            .expect("Failed to load the default texture!");
        framework.preload_model_asset("test".into(), "models/123123.gltf");
        framework.preload_model_asset("test1".into(), "models/tiles/vanillaplains/1/land1.gltf");
        let model_asset = framework.get_model_asset("test").unwrap();
        let model_asset1 = framework.get_model_asset("test1").unwrap();
        let texture_asset = framework.get_texture_asset("default").unwrap();
        let obj = framework.new_master_instanced_model_object("test", model_asset, Some(texture_asset.clone()), ShaderAsset::load_default_instanced_shader().unwrap(), false, RenderLayer::Layer1);
        let obj0 = framework.new_instanced_model_object("test0", "test");
        let obj00 = framework.new_instanced_model_object("test00", "test");
        let obj1 = framework.new_model_object("test1", model_asset1, Some(texture_asset), ShaderAsset::load_default_shader().unwrap(), false, RenderLayer::Layer1);
        let obj2 = framework.new_instanced_model_transform_holder("test00", "test",
            vec![
                Transform {position: Vec3::new(1.0, 0.0, 0.0), ..Default::default()},
                Transform {position: Vec3::new(20.0, 0.0, 0.0), ..Default::default()},
                Transform {position: Vec3::new(40.0, 0.0, 0.0), ..Default::default()},
            ]
        );
        add_system(Box::new(MainSystem {objects: vec![Box::new(obj0), Box::new(obj1), Box::new(obj00), Box::new(obj), Box::new(obj2)]}), framework);
        framework.set_camera_position(Vec3::new(0.0, 0.0, 5.0));
        framework.input.new_bind("forward", vec![InputEventType::Key(KeyCode::KeyW)]);
        framework.input.new_bind("backward", vec![InputEventType::Key(KeyCode::KeyS)]);
        framework.input.new_bind("left", vec![InputEventType::Key(KeyCode::KeyA)]);
        framework.input.new_bind("right", vec![InputEventType::Key(KeyCode::KeyD)]);
        framework.input.new_bind("cam_left", vec![InputEventType::Key(KeyCode::ArrowLeft)]);
        framework.input.new_bind("cam_right", vec![InputEventType::Key(KeyCode::ArrowRight)]);

        framework.render.as_mut().unwrap().set_camera_position(Vec3::new(0.0, 2.0, 0.0));
    }

    //add_system(Box::new(PlayerManager::new()), framework);
    //add_system(Box::new(WorldGenerator::new()));
    /*add_system(Box::new(LuaSystem::new("player_menu", "scripts/lua/player_menu.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("biomes", "scripts/lua/biomes.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("inventory", "scripts/lua/inventory.lua").unwrap()), framework);
    //add_system(Box::new(LuaSystem::new("world_generator", "scripts/lua/world_generation.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("tile1", "scripts/lua/tile1.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("player_manager", "scripts/lua/player_manager.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("experience", "scripts/lua/experience.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("vanilla_biomes", "scripts/lua/vanilla_biomes.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("vanilla_props", "scripts/lua/vanilla_props.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("vanilla_items", "scripts/lua/vanilla_items.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("land_unlock", "scripts/lua/land_unlock.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("land_placement", "scripts/lua/land_placement.lua").unwrap()), framework);*/
}

pub fn update(framework: &mut Framework) {
    if framework.input.is_bind_pressed("debug_toggle") {
        match framework.debug_mode() {
            DebugMode::None => framework.set_debug_mode(DebugMode::ShowFps),
            _ => {
                framework.set_debug_mode(DebugMode::None);
            }
        }
    }
    let mut camera_pos = framework.get_camera_position().unwrap();
    let mut camera_rot = framework.get_camera_rotation().unwrap();

    if framework.input.is_bind_pressed("forward") {
        camera_pos.z -= 1.0;
    }

    if framework.input.is_bind_pressed("backward") {
        camera_pos.z += 1.0;
    }

    if framework.input.is_bind_pressed("left") {
        camera_pos.x -= 1.0;
    }

    if framework.input.is_bind_pressed("right") {
        camera_pos.x += 1.0;
    }

    if framework.input.is_bind_pressed("cam_left") {
        camera_rot.y -= 15.0;
    }

    if framework.input.is_bind_pressed("cam_right") {
        camera_rot.y += 15.0;
    }

    framework.set_camera_position(camera_pos);
    framework.set_camera_rotation(camera_rot);
}

//pub fn render() {}
