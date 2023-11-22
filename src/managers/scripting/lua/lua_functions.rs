use mlua::Lua;

use crate::{managers::{systems, debugger}, assets::{texture_asset::get_default_texture_path, shader_asset::{ShaderAssetPath, get_default_vertex_shader_path, get_default_fragment_shader_path}}};

use super::SYSTEMS_LUA_VMS;

pub fn add_lua_vm_to_list(system_id: String, lua: Lua) {
    unsafe {
        // creating some functions 
        SYSTEMS_LUA_VMS.insert(system_id.clone(), lua);
        let lua = SYSTEMS_LUA_VMS.get_mut(&system_id).unwrap();

        let system_id_for_functions = system_id.clone();
        let new_empty_object = lua.create_function_mut(move |_, name: String| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    system.call_with_args("new_empty_object", vec![name]);
                },
                None => debugger::error("failed to call new_empty_object, system not found"),
            }
            
            Ok(())
        });

        
        let system_id_for_functions = system_id.clone();
        let new_model_object = lua.create_function_mut(
            move |_, (name, model_asset_path, texture_asset_path, vertex_shader_asset_path, fragment_shader_asset_path): (String, String, Option<String>, Option<String>, Option<String>)| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    let texture_asset_path_str: String;
                    match texture_asset_path {
                        Some(path) => texture_asset_path_str = path,
                        None => texture_asset_path_str = get_default_texture_path()
                    }
                    let vert_shader_path_str: String;
                    let frag_shader_path_str: String;
                    match vertex_shader_asset_path {
                        Some(path) => vert_shader_path_str = path,
                        None => vert_shader_path_str = get_default_vertex_shader_path()
                    }
                    match fragment_shader_asset_path {
                        Some(path) => frag_shader_path_str = path,
                        None => frag_shader_path_str = get_default_fragment_shader_path()
                    }
                    system.call_with_args("new_model_object", vec![name, model_asset_path, texture_asset_path_str, vert_shader_path_str, frag_shader_path_str]);
                },
                None => debugger::error("failed to call new_empty_object, system not found"),
            }
            
            Ok(())
        });

        match new_empty_object {
            Ok(func) => {
                if let Err(err) = lua.globals().set("new_empty_object", func) {
                    debugger::error(&format!("failed to add a function new empty_object as a lua global in system {}\nerror: {}", system_id, err));
                }
            }, 
            Err(err) => debugger::error(&format!("failed to create a function new_empty_object in system {}\nerror: {}", system_id, err)),
        }

        let system_id_for_functions = system_id.clone();
        let call_in_object = lua.create_function_mut(move |_, (name, call_id, args): (String, String, Vec<String>)| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    let mut full_args_vector: Vec<String> = Vec::new();
                    full_args_vector.push(name);
                    full_args_vector.push(call_id);
                    args.iter().for_each(|arg| full_args_vector.push(arg.into()));
                    system.call_with_args("call_in_object", full_args_vector);
                },
                None => debugger::error("failed to call call_in_object, system not found"),
            }
            
            Ok(())
        });


        match call_in_object {
            Ok(func) => {
                if let Err(err) = lua.globals().set("call_in_object", func) {
                    debugger::error(&format!("failed to add a function new call_in_object as a lua global in system {}\nerror: {}", system_id, err));
                }
            }, 
            Err(err) => debugger::error(&format!("failed to create a function call_in_object in system {}\nerror: {}", system_id, err)),
        }
    }
}
