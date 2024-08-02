use glam::Vec2;
use winit::keyboard::KeyCode;

use crate::{
    framework::{DebugMode, Framework},
    managers::{input::InputEventType, networking, scripting::lua::LuaSystem, systems::add_system}, systems::player_manager::PlayerManager,
    //systems::player_manager::PlayerManager,
};

pub fn start(framework: &mut Framework) {
    framework.input.new_bind(
        "debug_toggle",
        vec![InputEventType::Key(KeyCode::Backquote)],
    );
    if networking::is_server() == false {
        framework
            .preload_texture_asset("default".into(), "textures/default_texture.png")
            .expect("Failed to load the default texture!");
        framework.ui.as_mut().unwrap().new_window("Test", false);
        framework.ui.as_mut().unwrap().new_window("Test2", true);
        framework.ui.as_mut().unwrap().add_button("Test2", "TestButton1", "123213123", Vec2::new(700.0, 10.0), None);
        framework.ui.as_mut().unwrap().add_horizontal("Test", "TestHorizontal", Vec2::new(200.0, 200.0), None);
        framework.ui.as_mut().unwrap().add_button("Test", "TestButton2", "123213123", Vec2::new(69.0, 200.0), Some("TestHorizontal"));
        framework.ui.as_mut().unwrap().add_button("Test", "TestButton3", "456456456", Vec2::new(420.0, 200.0), Some("TestHorizontal"));
        framework.ui.as_mut().unwrap().add_button("Test", "TestButton1", "123213123", Vec2::new(200.0, 200.0), None);
        framework.ui.as_mut().unwrap().add_checkbox("Test", "TestCheckbox", false, "test checkbox lol", Vec2::new(200.0, 200.0), None);
        framework.ui.as_mut().unwrap().add_label("Test", "TestCheckbox", "test label", Vec2::new(200.0, 200.0), None);
        framework.ui.as_mut().unwrap().add_float_slider("Test", "TestSlider", 5.0, 0.0, 21.0, Vec2::new(200.0, 200.0), None);
        framework.ui.as_mut().unwrap().add_progress_bar("Test", "TestProgressBar", 0.25, Vec2::new(200.0, 200.0), None);
        framework.ui.as_mut().unwrap().add_singleline_text_edit("Test", "TestSingleTextEdit", "abc hehe", Vec2::new(200.0, 200.0), None);
        framework.ui.as_mut().unwrap().add_multiline_text_edit("Test", "TestMultiTextEdit", "abc hehe\nabc", Vec2::new(200.0, 200.0), None);
        framework.ui.as_mut().unwrap().set_window_position("Test2", Some(Vec2::new(100.0, 100.0)));
    }

    //add_system(Box::new(TestSystem::new()));
    add_system(Box::new(PlayerManager::new()), framework);
    //add_system(Box::new(WorldGenerator::new()));
    //add_system(Box::new(LuaSystem::new("player_manager", "scripts/lua/player_manager.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("vanila_props", "scripts/lua/vanilla_props.lua").unwrap()), framework);

    add_system(Box::new(LuaSystem::new("player_manager", "scripts/lua/player_manager.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("world_generator", "scripts/lua/world_generation.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("tile1", "scripts/lua/tile1.lua").unwrap()), framework);
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
    if networking::is_server() == false {
        if framework.ui.as_mut().unwrap().get_widget_state("Test2", "TestButton1").unwrap().left_clicked == true {
            framework.ui.as_mut().unwrap().remove_widget("Test", "TestButton2");
            framework.ui.as_mut().unwrap().remove_widget("Test", "TestButton1");
        }
        dbg!(framework.ui.as_mut().unwrap().get_widget_text("Test", "TestMultiTextEdit"));
    }
}

//pub fn render() {}
