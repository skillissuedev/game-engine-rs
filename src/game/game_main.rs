use winit::keyboard::KeyCode;

use crate::{
    framework::{DebugMode, Framework},
    managers::{
        input::InputEventType, /*scripting::lua::LuaSystem,*/ systems::add_system
    },
    systems::player_manager::PlayerManager
};

pub fn start(framework: &mut Framework) {
    framework.input.new_bind(
        "debug_toggle",
        vec![InputEventType::Key(KeyCode::Backquote)],
    );
    framework.preload_texture_asset("default".into(), "textures/default_texture.png")
        .expect("Failed to load the default texture!");

    //add_system(Box::new(TestSystem::new()));
    add_system(Box::new(PlayerManager::new()), framework);
    //add_system(Box::new(WorldGenerator::new()));
    /*add_system(Box::new(LuaSystem::new("player_manager", "scripts/lua/player_manager.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("world_generator", "scripts/lua/world_generation.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("tile1", "scripts/lua/tile1.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("vanila_props", "scripts/lua/vanilla_props.lua").unwrap()), framework);*/
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
