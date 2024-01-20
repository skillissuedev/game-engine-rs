use glam::Vec3;
use mlua::Lua;
use crate::{
    managers::{systems, debugger, self}, 
    assets::{texture_asset::get_default_texture_path, shader_asset::{get_default_vertex_shader_path, get_default_fragment_shader_path}}
};
use super::SYSTEMS_LUA_VMS;

pub fn add_lua_vm_to_list(system_id: String, lua: Lua) {
    unsafe {
        SYSTEMS_LUA_VMS.insert(system_id.clone(), lua);
        let lua = SYSTEMS_LUA_VMS.get_mut(&system_id).unwrap();


        // creating some functions 
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

        match new_empty_object {
            Ok(func) => {
                if let Err(err) = lua.globals().set("new_empty_object", func) {
                    debugger::error(&format!("failed to add a function new_empty_object as a lua global in system {}\nerror: {}", system_id, err));
                }
            }, 
            Err(err) => debugger::error(&format!("failed to create a function new_empty_object in system {}\nerror: {}", system_id, err)),
        }

        let system_id_for_functions = system_id.clone();
        let new_sound_emitter_object = lua.create_function_mut(move |_, (name, wav_sound_path, is_positional): (String, String, bool)| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    system.call_with_args("new_sound_emitter_object", vec![name, wav_sound_path, is_positional.to_string()]);
                },
                None => debugger::error("failed to call new_sound_emitter_object, system not found"),
            }
            
            Ok(())
        });

        match new_sound_emitter_object {
            Ok(func) => {
                if let Err(err) = lua.globals().set("new_sound_emitter_object", func) {
                    debugger::error(&format!("failed to add a function new_sound_emitter_object as a lua global in system {}\nerror: {}", system_id, err));
                }
            }, 
            Err(err) => debugger::error(&format!("failed to create a function new_sound_emitter_object in system {}\nerror: {}", system_id, err)),
        }

        let system_id_for_functions = system_id.clone();
        let new_camera_position_object = lua.create_function_mut(move |_, name: String| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    system.call_with_args("new_camera_position_object", vec![name]);
                },
                None => debugger::error("failed to call new_camera_position_object, system not found"),
            }
            
            Ok(())
        });

        match new_camera_position_object {
            Ok(func) => {
                if let Err(err) = lua.globals().set("new_camera_position_object", func) {
                    debugger::error(&format!("failed to add a function new_camera_position_object as a lua global in system {}\nerror: {}", system_id, err));
                }
            }, 
            Err(err) => debugger::error(&format!("failed to create a function new_camera_position_object in system {}\nerror: {}", system_id, err)),
        }

        
        let system_id_for_functions = system_id.clone();
        let new_model_object = lua.create_function_mut(
            move |_, (name, model_asset_path, texture_asset_path, vertex_shader_asset_path, fragment_shader_asset_path):
            (String, String, Option<String>, Option<String>, Option<String>)| {
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

        match new_model_object {
            Ok(func) => {
                if let Err(err) = lua.globals().set("new_model_object", func) {
                    debugger::error(&format!("failed to add a function new_model_object as a lua global in system {}\nerror: {}", system_id, err));
                }
            }, 
            Err(err) => debugger::error(&format!("failed to create a function new_model_object in system {}\nerror: {}", system_id, err)),
        }


        let system_id_for_functions = system_id.clone();
        let set_object_position = lua.create_function_mut(move |_, 
            (name, pos_x, pos_y, pos_z): (String, f32, f32, f32)| {
            match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                Some(system) => {
                    let object_option = system.find_object_mut(&name);
                    match object_option {
                        Some(object) => object.set_position(Vec3::new(pos_x, pos_y, pos_z), true),
                        None => debugger::error("failed to call set_object_position, object not found"),
                    }
                },
                None =>  debugger::error("failed to call set_object_position, system not found"),
            }

            Ok(())
        });

        match set_object_position {
            Ok(func) => {
                if let Err(err) = lua.globals().set("set_object_position", func) {
                    debugger::error(&format!("failed to add a function set_object_position as a lua global in system {}\nerror: {}", system_id, err));
                }
            }, 
            Err(err) => debugger::error(&format!("failed to create a function set_object_position in system {}\nerror: {}", system_id, err)),
        }


        let system_id_for_functions = system_id.clone();
        let set_object_rotation = lua.create_function_mut(move |_, 
            (name, rot_x, rot_y, rot_z): (String, f32, f32, f32)| {
            match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                Some(system) => {
                    let object_option = system.find_object_mut(&name);
                    match object_option {
                        Some(object) => object.set_rotation(Vec3::new(rot_x, rot_y, rot_z), true),
                        None => debugger::error("failed to call set_object_rotation, object not found"),
                    }
                },
                None =>  debugger::error("failed to call set_object_rotation, system not found"),
            }

            Ok(())
        });

        match set_object_rotation {
            Ok(func) => {
                if let Err(err) = lua.globals().set("set_object_rotation", func) {
                    debugger::error(&format!("failed to add a function set_object_rotation as a lua global in system {}\nerror: {}", system_id, err));
                }
            }, 
            Err(err) => debugger::error(&format!("failed to create a function set_object_rotation in system {}\nerror: {}", system_id, err)),
        }


        let system_id_for_functions = system_id.clone();
        let set_object_scale = lua.create_function_mut(move |_, 
            (name, sc_x, sc_y, sc_z): (String, f32, f32, f32)| {
            match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                Some(system) => {
                    let object_option = system.find_object_mut(&name);
                    match object_option {
                        Some(object) => object.set_scale(Vec3::new(sc_x, sc_y, sc_z)),
                        None => debugger::error("failed to call set_object_scale, object not found"),
                    }
                },
                None =>  debugger::error("failed to call set_object_scale, system not found"),
            }

            Ok(())
        });

        match set_object_scale {
            Ok(func) => {
                if let Err(err) = lua.globals().set("set_object_scale", func) {
                    debugger::error(&format!("failed to add a function set_object_scale as a lua global in system {}\nerror: {}", system_id, err));
                }
            }, 
            Err(err) => debugger::error(&format!("failed to create a function set_object_scale in system {}\nerror: {}", system_id, err)),
        }

        let system_id_for_functions = system_id.clone();
        let get_object_position = lua.create_function_mut(move |_, name: String| {
            match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                Some(system) => {
                    let object_option = system.find_object_mut(&name);
                    match object_option {
                        Some(object) => return 
                            Ok(vec![object.global_transform().position.x, object.global_transform().position.y, object.global_transform().position.z]),
                        None => debugger::error("failed to call get_object_position, object not found"),
                    }
                },
                None =>  debugger::error("failed to call get_object_position, system not found"),
            }

            Ok(vec![])
        });

        match get_object_position {
            Ok(func) => {
                if let Err(err) = lua.globals().set("get_object_position", func) {
                    debugger::error(&format!("failed to add a function get_object_position as a lua global in system {}\nerror: {}", system_id, err));
                }
            }, 
            Err(err) => debugger::error(&format!("failed to create a function get_object_position in system {}\nerror: {}", system_id, err)),
        }

        let system_id_for_functions = system_id.clone();
        let get_object_rotation = lua.create_function_mut(move |_, name: String| {
            match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                Some(system) => {
                    let object_option = system.find_object_mut(&name);
                    match object_option {
                        Some(object) => return 
                            Ok(vec![object.global_transform().rotation.x, object.global_transform().rotation.y, object.global_transform().rotation.z]),
                        None => debugger::error("failed to call get_object_rotation, object not found"),
                    }
                },
                None =>  debugger::error("failed to call get_object_rotation, system not found"),
            }

            Ok(vec![])
        });

        match get_object_rotation {
            Ok(func) => {
                if let Err(err) = lua.globals().set("get_object_rotation", func) {
                    debugger::error(&format!("failed to add a function get_object_rotation as a lua global in system {}\nerror: {}", system_id, err));
                }
            }, 
            Err(err) => debugger::error(&format!("failed to create a function get_object_rotation in system {}\nerror: {}", system_id, err)),
        } 

        let system_id_for_functions = system_id.clone();
        let get_object_scale = lua.create_function_mut(move |_, name: String| {
            match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                Some(system) => {
                    let object_option = system.find_object_mut(&name);
                    match object_option {
                        Some(object) => return 
                            Ok(vec![object.global_transform().scale.x, object.global_transform().scale.y, object.global_transform().scale.z]),
                        None => debugger::error("failed to call get_object_scale, object not found"),
                    }
                },
                None =>  debugger::error("failed to call get_object_scale, system not found"),
            }

            Ok(vec![])
        });

        match get_object_scale {
            Ok(func) => {
                if let Err(err) = lua.globals().set("get_object_scale", func) {
                    debugger::error(&format!("failed to add a function get_object_scale as a lua global in system {}\nerror: {}", system_id, err));
                }
            }, 
            Err(err) => debugger::error(&format!("failed to create a function get_object_scale in system {}\nerror: {}", system_id, err)),
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
