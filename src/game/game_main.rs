use glam::{Vec2, Vec3};
use glium::winit::keyboard::KeyCode;

use crate::{
    assets::{model_asset::ModelAsset, shader_asset::ShaderAsset}, framework::{DebugMode, Framework}, managers::{assets::ModelAssetId, input::InputEventType, networking, render::{RenderLayer, RenderObjectData, RenderShader}, scripting::lua::LuaSystem, systems::{add_system, SystemValue}}, objects::{point_light::PointLight, Transform}, systems::main_system::MainSystem, Args//, systems::player_manager::PlayerManager, Args,
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
        add_system(Box::new(MainSystem {objects: vec![]}), framework);
    }

    //add_system(Box::new(PlayerManager::new()), framework);
    //add_system(Box::new(WorldGenerator::new()));
    add_system(Box::new(LuaSystem::new("player_menu", "scripts/lua/player_menu.lua").unwrap()), framework);
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
    add_system(Box::new(LuaSystem::new("land_placement", "scripts/lua/land_placement.lua").unwrap()), framework);
}

pub fn update(framework: &mut Framework) {
    if framework.input.is_bind_pressed("debug_toggle") {
        match framework.debug_mode() {
            DebugMode::None => framework.set_debug_mode(DebugMode::ShowFps),
            _ => {
                framework.set_debug_mode(DebugMode::None);
            }
        }
    }/*
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
    framework.set_camera_rotation(camera_rot);*/
}

//pub fn render() {}
