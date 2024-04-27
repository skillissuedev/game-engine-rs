use super::{ObjectHandle, SYSTEMS_LUA_VMS};
use crate::{
    assets::{
        self,
        model_asset::ModelAsset,
        shader_asset::{ShaderAsset, ShaderAssetPath},
        sound_asset::SoundAsset,
        texture_asset::TextureAsset,
    },
    managers::{
        self, debugger,
        physics::{BodyColliderType, CollisionGroups},
        systems,
    },
    objects::{
        camera_position::CameraPosition, character_controller::CharacterController,
        empty_object::EmptyObject, model_object::ModelObject, sound_emitter::SoundEmitter, Object,
    }, systems::System,
};
use ez_al::SoundSourceType;
use glam::Vec3;
use mlua::Lua;

pub fn add_lua_vm_to_list(system_id: String, lua: Lua) {
    unsafe {
        SYSTEMS_LUA_VMS.insert(system_id.clone(), lua);
        let lua = SYSTEMS_LUA_VMS.get_mut(&system_id).unwrap();
        let _ = lua.globals().set("current_parent", None::<String>);

        // creating some functions
        let system_id_for_functions = system_id.clone();
        let set_current_parent = lua.create_function(move |lua, name: String| {
            if let Err(err) = lua.globals().set("current_parent", Some(name)) {
                debugger::error(&format!(
                    "lua error: failed to set current_parent! system: {}\nerr: {:?}",
                    system_id_for_functions, err
                ));
            }
            Ok(())
        });

        match set_current_parent {
            Ok(func) => {
                if let Err(err) = lua.globals().set("set_current_parent", func) {
                    debugger::error(&format!("failed to add a function set_current_parent as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function set_current_parent in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let clear_current_parent = lua.create_function(move |lua, _: ()| {
            if let Err(err) = lua.globals().set("current_parent", None::<String>) {
                debugger::error(&format!(
                    "lua error: failed to set current_parent! system: {}\nerr: {:?}",
                    system_id_for_functions, err
                ));
            }
            Ok(())
        });

        match clear_current_parent {
            Ok(func) => {
                if let Err(err) = lua.globals().set("clear_current_parent", func) {
                    debugger::error(&format!("failed to add a function clear_current_parent as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function clear_current_parent in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let find_object = lua.create_function(move |_, name: String| {
            let system_option = systems::get_system_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => match system.find_object(&name) {
                    Some(object) => Ok(Some(ObjectHandle {
                        system_id: system.system_id().into(),
                        name: object.name().into(),
                    })),
                    None => {
                        debugger::error("failed to call find_object: object not found");
                        Ok(None)
                    }
                },
                None => {
                    debugger::error("failed to call find_object: system not found");
                    Ok(None)
                }
            }
        });

        match find_object {
            Ok(func) => {
                if let Err(err) = lua.globals().set("find_object", func) {
                    debugger::error(&format!("failed to add a function find_object as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function find_object in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let new_empty_object = lua.create_function_mut(move |lua, name: String| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    let object = EmptyObject::new(&name);
                    add_to_system_or_parent(lua, system, Box::new(object));
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
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function new_empty_object in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let new_sound_emitter_object = lua.create_function_mut(move |lua, (name, wav_sound_path, should_loop, is_positional, max_distance): (String, String, bool, bool, f32)| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    let sound_asset = SoundAsset::from_wav(&wav_sound_path);
                    match sound_asset {
                        Ok(asset) => {
                            let emitter_type = match is_positional {
                                true => SoundSourceType::Positional,
                                false => SoundSourceType::Simple,
                            };

                            let object = SoundEmitter::new(&name, &asset, emitter_type);
                            match object {
                                Ok(mut object) => {
                                    if is_positional {
                                        let _ = object.set_max_distance(max_distance);
                                    }
                                    object.set_looping(should_loop);
                                    add_to_system_or_parent(lua, system, Box::new(object));
                                },
                                Err(err) => 
                                    debugger::error(&format!("failed to call new_sound_emitter_object: got an error when creating SoundEmitter! err: {:?}", err)),
                            }
                        },
                        Err(err) => debugger::error(&format!("failed to call new_sound_emitter_object: got an error when creating SoundAsset! err: {:?}", err)),
                    }
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
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function new_sound_emitter_object in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let new_camera_position_object = lua.create_function_mut(move |lua, name: String| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    let object = CameraPosition::new(&name);
                    add_to_system_or_parent(lua, system, Box::new(object));
                }
                None => {
                    debugger::error("failed to call new_camera_position_object, system not found")
                }
            }

            Ok(())
        });

        match new_camera_position_object {
            Ok(func) => {
                if let Err(err) = lua.globals().set("new_camera_position_object", func) {
                    debugger::error(&format!("failed to add a function new_camera_position_object as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function new_camera_position_object in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let new_model_object = lua.create_function_mut(
            move |lua, (name, model_asset_path, texture_asset_path, vertex_shader_asset_path, fragment_shader_asset_path):
            (String, String, Option<String>, Option<String>, Option<String>)| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    let texture_asset;
                    match texture_asset_path {
                        Some(path) => {
                            let asset = TextureAsset::from_file(&path);
                            match asset {
                                Ok(asset) => texture_asset = Some(asset),
                                Err(err) => {
                                    debugger::warn(&format!("lua warning: error when calling new_model_object, failed to load texture asset!\nerr: {:?}", err));
                                    texture_asset = None;
                                },
                            }
                        },
                        None => texture_asset = None,
                    }
                    let mut shader_asset_path = ShaderAssetPath {
                        vertex_shader_path: assets::shader_asset::get_default_vertex_shader_path(),
                        fragment_shader_path: assets::shader_asset::get_default_fragment_shader_path(),
                    };
                    if let Some(vertex_shader_asset_path) = vertex_shader_asset_path {
                        shader_asset_path.vertex_shader_path = vertex_shader_asset_path;
                    }
                    if let Some(fragment_shader_asset_path) = fragment_shader_asset_path {
                        shader_asset_path.fragment_shader_path = fragment_shader_asset_path;
                    }
                    let shader_asset = ShaderAsset::load_from_file(shader_asset_path);
                    match shader_asset {
                        Ok(shader_asset) => {
                            let model_asset = ModelAsset::from_gltf(&model_asset_path);
                            match model_asset {
                                Ok(model_asset) => {
                                    let object = ModelObject::new(&name, model_asset, texture_asset, shader_asset);
                                    add_to_system_or_parent(lua, system, Box::new(object));
                                },
                                Err(err) => 
                                    debugger::error(&format!("lua error: error when calling new_model_object, failed to load a model asset!\nerr: {:?}", err)),
                            }
                        },
                        Err(err) => 
                            debugger::error(&format!("lua error: error when calling new_model_object, failed to load a shader asset!\nerr: {:?}", err)),
                    }
                },
                None => debugger::error("failed to call new_model_object, system not found"),
            }
            
            Ok(())
        });

        match new_model_object {
            Ok(func) => {
                if let Err(err) = lua.globals().set("new_model_object", func) {
                    debugger::error(&format!("failed to add a function new_model_object as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function new_model_object in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let new_character_controller = lua.create_function_mut(
            move |lua,
                  (name, collider_type, size_x, size_y, size_z, membership_bits, mask_bits): (
                String,
                String,
                f32,
                f32,
                f32,
                Option<u32>,
                Option<u32>,
            )| {
                let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
                match system_option {
                    Some(system) => {
                        let collider = match collider_type.as_str() {
                            "Cuboid" => BodyColliderType::Cuboid(size_x, size_y, size_z),
                            "Capsule" => BodyColliderType::Capsule(size_x, size_y),
                            "Cylinder" => BodyColliderType::Cylinder(size_x, size_y),
                            "Ball" => BodyColliderType::Ball(size_x),
                            _ => {
                                // error here
                                BodyColliderType::Capsule(size_x, size_y)
                            }
                        };
                        let membership = match membership_bits {
                            Some(bits) => Some(CollisionGroups::from(bits)),
                            None => None,
                        };
                        let mask = match mask_bits {
                            Some(bits) => Some(CollisionGroups::from(bits)),
                            None => None,
                        };
                        let object = CharacterController::new(&name, collider, membership, mask);
                        add_to_system_or_parent(lua, system, Box::new(object));
                    }
                    None => debugger::error("failed to call new_empty_object, system not found"),
                }

                Ok(())
            },
        );

        match new_character_controller {
            Ok(func) => {
                if let Err(err) = lua.globals().set("new_character_controller_object", func) {
                    debugger::error(&format!("failed to add a function new_character_controller as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function new_character_controller in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let set_object_position = lua.create_function_mut(
            move |_, (name, pos_x, pos_y, pos_z): (String, f32, f32, f32)| {
                match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                    Some(system) => {
                        let object_option = system.find_object_mut(&name);
                        match object_option {
                            Some(object) => {
                                object.set_position(Vec3::new(pos_x, pos_y, pos_z), true)
                            }
                            None => debugger::error(
                                "failed to call set_object_position, object not found",
                            ),
                        }
                    }
                    None => debugger::error("failed to call set_object_position, system not found"),
                }

                Ok(())
            },
        );

        match set_object_position {
            Ok(func) => {
                if let Err(err) = lua.globals().set("set_object_position", func) {
                    debugger::error(&format!("failed to add a function set_object_position as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function set_object_position in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let set_object_rotation = lua.create_function_mut(
            move |_, (name, rot_x, rot_y, rot_z): (String, f32, f32, f32)| {
                match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                    Some(system) => {
                        let object_option = system.find_object_mut(&name);
                        match object_option {
                            Some(object) => {
                                object.set_rotation(Vec3::new(rot_x, rot_y, rot_z), true)
                            }
                            None => debugger::error(
                                "failed to call set_object_rotation, object not found",
                            ),
                        }
                    }
                    None => debugger::error("failed to call set_object_rotation, system not found"),
                }

                Ok(())
            },
        );

        match set_object_rotation {
            Ok(func) => {
                if let Err(err) = lua.globals().set("set_object_rotation", func) {
                    debugger::error(&format!("failed to add a function set_object_rotation as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function set_object_rotation in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let set_object_scale = lua.create_function_mut(
            move |_, (name, sc_x, sc_y, sc_z): (String, f32, f32, f32)| {
                match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                    Some(system) => {
                        let object_option = system.find_object_mut(&name);
                        match object_option {
                            Some(object) => object.set_scale(Vec3::new(sc_x, sc_y, sc_z)),
                            None => {
                                debugger::error("failed to call set_object_scale, object not found")
                            }
                        }
                    }
                    None => debugger::error("failed to call set_object_scale, system not found"),
                }

                Ok(())
            },
        );

        match set_object_scale {
            Ok(func) => {
                if let Err(err) = lua.globals().set("set_object_scale", func) {
                    debugger::error(&format!("failed to add a function set_object_scale as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function set_object_scale in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let get_object_position = lua.create_function_mut(move |_, name: String| {
            match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                Some(system) => {
                    let object_option = system.find_object_mut(&name);
                    match object_option {
                        Some(object) => {
                            return Ok(vec![
                                object.global_transform().position.x,
                                object.global_transform().position.y,
                                object.global_transform().position.z,
                            ])
                        }
                        None => {
                            debugger::error("failed to call get_object_position, object not found")
                        }
                    }
                }
                None => debugger::error("failed to call get_object_position, system not found"),
            }

            Ok(vec![])
        });

        match get_object_position {
            Ok(func) => {
                if let Err(err) = lua.globals().set("get_object_position", func) {
                    debugger::error(&format!("failed to add a function get_object_position as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function get_object_position in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let get_object_rotation = lua.create_function_mut(move |_, name: String| {
            match managers::systems::get_system_with_id(&system_id_for_functions) {
                Some(system) => {
                    let object_option = system.find_object(&name);
                    match object_option {
                        Some(object) => {
                            return Ok(vec![
                                object.global_transform().rotation.x,
                                object.global_transform().rotation.y,
                                object.global_transform().rotation.z,
                            ])
                        }
                        None => {
                            debugger::error("failed to call get_object_rotation, object not found")
                        }
                    }
                }
                None => debugger::error("failed to call get_object_rotation, system not found"),
            }

            Ok(vec![])
        });

        match get_object_rotation {
            Ok(func) => {
                if let Err(err) = lua.globals().set("get_object_rotation", func) {
                    debugger::error(&format!("failed to add a function get_object_rotation as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function get_object_rotation in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let get_object_scale = lua.create_function_mut(move |_, name: String| {
            match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                Some(system) => {
                    let object_option = system.find_object_mut(&name);
                    match object_option {
                        Some(object) => {
                            return Ok(vec![
                                object.global_transform().scale.x,
                                object.global_transform().scale.y,
                                object.global_transform().scale.z,
                            ])
                        }
                        None => {
                            debugger::error("failed to call get_object_scale, object not found")
                        }
                    }
                }
                None => debugger::error("failed to call get_object_scale, system not found"),
            }

            Ok(vec![])
        });

        match get_object_scale {
            Ok(func) => {
                if let Err(err) = lua.globals().set("get_object_scale", func) {
                    debugger::error(&format!("failed to add a function get_object_scale as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function get_object_scale in system {}\nerror: {}",
                system_id, err
            )),
        }

        /*let call_in_object = lua.create_function_mut(move |_, (name, call_id, args): (String, String, Vec<String>)| {
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
        }*/
    }
}

fn add_to_system_or_parent(lua: &Lua, system: &mut Box<dyn System>, object: Box<dyn Object>) {
    if let Ok(current_parent) = lua.globals().get::<&str, Option<String>>("current_parent") {
        if let Some(current_parent) = current_parent {
            match system.find_object_mut(&current_parent) {
                Some(parent_object) => {
                    parent_object.add_child(object);
                    return; 
                },
                None => {
                    debugger::error("failed to call new_empty_object, failed to get the current_parent object!");
                }
            }
        }
    }
    system.add_object(object);
}
