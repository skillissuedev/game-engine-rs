use std::collections::HashMap;

use glam::Vec3;
use glium::winit::keyboard::KeyCode;

use crate::{
    assets::{model_asset::ModelAsset, shader_asset::ShaderAsset}, framework::{DebugMode, Framework}, managers::{assets::ModelAssetId, input::InputEventType, networking, render::{RenderLayer, RenderObjectData, RenderShader}, scripting::lua::LuaSystem, systems::{add_system, SystemValue}}, systems::main_system::MainSystem, Args//, systems::player_manager::PlayerManager, Args,
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
        framework.preload_model_asset("test".into(), "models/cube.gltf");
        let model_asset = framework.get_model_asset("test").unwrap();
        let texture_asset = framework.get_texture_asset("default").unwrap();
        let obj = framework.new_model_object("test", model_asset, Some(texture_asset), ShaderAsset::load_default_shader().unwrap(), false, RenderLayer::Layer1);
        add_system(Box::new(MainSystem {objects: vec![Box::new(obj)]}), framework);
        framework.set_camera_position(Vec3::new(0.0, 0.0, -2.0));
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
}

//pub fn render() {}
