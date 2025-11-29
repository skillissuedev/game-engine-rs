use egui_glium::egui_winit::egui::Visuals;
use glium::winit::keyboard::KeyCode;
use crate::{
    framework::{DebugMode, Framework},
    managers::{input::InputEventType, networking, scripting::lua::LuaSystem, systems::add_system},
    systems::main_system::MainSystem, Args
};

pub fn start(args: Args, framework: &mut Framework) {
    println!("{}", serde_json::to_string_pretty(&Visuals::dark()).unwrap());

    if let Some(save_name) = &args.new_save_name {
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
        framework.preload_sound_asset("ui_click_sound".into(), "sounds/ui_click.wav")
            .expect("Failed to load the default UI click sound!");
        framework.preload_sound_asset("ui_hover_start_sound".into(), "sounds/ui_hover_start.wav")
            .expect("Failed to load the default UI hover start sound!");
        framework.preload_sound_asset("ui_hover_stop_sound".into(), "sounds/ui_hover_stop.wav")
            .expect("Failed to load the default UI hover stop sound!");

        let click = framework.get_sound_asset("ui_click_sound").expect("Failed to get the default UI click sound!");
        let hover_start = framework.get_sound_asset("ui_hover_stop_sound").expect("Failed to get the default UI hover start sound!");
        //let hover_stop = framework.get_sound_asset("ui_hover_stop_sound").expect("Failed to get the default UI hover stop sound!");

        if let Some(al) = &mut framework.al {
            if let Some(ui) = &mut framework.ui {
                ui.set_click_sound(al, &framework.assets, Some(click));
                ui.set_hover_start_sound(al, &framework.assets, Some(hover_start));
                //ui.set_hover_stop_sound(al, &framework.assets, Some(hover_stop));
            }
        }

        framework
            .preload_texture_asset("default".into(), "textures/default_texture.png")
            .expect("Failed to load the default texture!");

        framework
            .preload_shader_asset("default".into(), "shaders/default.vert", "shaders/default.frag")
            .expect("Failed to load the default shader!");

        framework
            .preload_shader_asset("default_instanced".into(), "shaders/default_instanced.vert", "shaders/default_instanced.frag")
            .expect("Failed to load the default instanced shader!");
    }

    add_system(Box::new(LuaSystem::new("player_menu", "scripts/lua/player_menu.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("biomes", "scripts/lua/biomes.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("inventory", "scripts/lua/inventory.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("player_manager", "scripts/lua/player_manager.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("experience", "scripts/lua/experience.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("wild_tiles", "scripts/lua/wild_tiles.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("vanilla_biomes", "scripts/lua/vanilla_biomes.lua").unwrap()), framework);
    //add_system(Box::new(LuaSystem::new("props_manager", "scripts/lua/props_manager.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("vanilla_items", "scripts/lua/vanilla_items.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("land_unlock", "scripts/lua/land_unlock.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("land_placement", "scripts/lua/land_placement.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("interactable_objects", "scripts/lua/interactable_objects.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("vanilla_item_unlockers", "scripts/lua/vanilla_item_unlockers.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("vanilla_enemies", "scripts/lua/vanilla_enemies.lua").unwrap()), framework);

    add_system(Box::new(LuaSystem::new("clouds", "scripts/lua/clouds.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("water", "scripts/lua/water.lua").unwrap()), framework);
    //add_system(Box::new(LuaSystem::new("ambiance_and_music", "scripts/lua/ambiance_and_music.lua").unwrap()), framework);

    add_system(Box::new(LuaSystem::new("props", "scripts/lua/props.lua").unwrap()), framework);
    add_system(Box::new(LuaSystem::new("building", "scripts/lua/building.lua").unwrap()), framework);

    add_system(Box::new(MainSystem {objects: vec![]}), framework);
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

