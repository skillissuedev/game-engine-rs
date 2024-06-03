use std::collections::HashMap;

use super::{ObjectHandle, SYSTEMS_LUA_VMS};
use crate::{
    assets::{
        self,
        model_asset::ModelAsset,
        shader_asset::{ShaderAsset, ShaderAssetPath},
        sound_asset::SoundAsset,
        texture_asset::TextureAsset,
    }, framework, managers::{
        self, debugger, networking::{self, Message, MessageContents, MessageReceiver, MessageReliability, SyncObjectMessage}, physics::{BodyColliderType, CollisionGroups}, systems::{self, SystemValue}
    }, objects::{
        camera_position::CameraPosition, character_controller::CharacterController, empty_object::EmptyObject, instanced_model_object::InstancedModelObject, instanced_model_transform_holder::InstancedModelTransformHolder, master_instanced_model_object::MasterInstancedModelObject, model_object::ModelObject, nav_obstacle::NavObstacle, navmesh::NavigationGround, ray::Ray, sound_emitter::SoundEmitter, trigger::Trigger, Object, Transform
    }, systems::System
};
use ez_al::SoundSourceType;
use glam::{Vec2, Vec3};
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
        let delete_object = lua.create_function(move |_, name: String| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) =>
                    Ok(system.delete_object(&name)),
                None => {
                    debugger::error("failed to call delete_object: system not found");
                    Ok(())
                }
            }
        });

        match delete_object {
            Ok(func) => {
                if let Err(err) = lua.globals().set("delete_object", func) {
                    debugger::error(&format!("failed to add a function delete_object as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function delete_object in system {}\nerror: {}",
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
        let new_master_instanced_model_object = lua.create_function_mut(
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
                                    debugger::warn(&format!("lua warning: error when calling new_master_instanced_model_object, failed to load texture asset!\nerr: {:?}", err));
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
                                    let object = MasterInstancedModelObject::new(&name, model_asset, texture_asset, shader_asset);
                                    add_to_system_or_parent(lua, system, Box::new(object));
                                },
                                Err(err) => 
                                    debugger::error(&format!("lua error: error when calling new_master_instanced_model_object, failed to load a model asset!\nerr: {:?}", err)),
                            }
                        },
                        Err(err) => 
                            debugger::error(&format!("lua error: error when calling new_master_instanced_model_object, failed to load a shader asset!\nerr: {:?}", err)),
                    }
                },
                None => debugger::error("failed to call new_master_instanced_model_object, system not found"),
            }
            
            Ok(())
        });

        match new_master_instanced_model_object {
            Ok(func) => {
                if let Err(err) = lua.globals().set("new_master_instanced_model_object", func) {
                    debugger::error(&format!("failed to add a function new_master_instanced_model_object as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function new_master_instanced_model_object in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let new_instanced_model_object = lua.create_function_mut(move |lua, (name, instance): (String, String)| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    let object = InstancedModelObject::new(&name, &instance);
                    add_to_system_or_parent(lua, system, Box::new(object));
                },
                None => debugger::error("failed to call new_instanced_model_object, system not found"),
            }
            
            Ok(())
        });

        match new_instanced_model_object {
            Ok(func) => {
                if let Err(err) = lua.globals().set("new_instanced_model_object", func) {
                    debugger::error(&format!("failed to add a function new_instanced_model_object as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function new_instanced_model_object in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        // transform = {{pos_x, pos_y, pos_z, rot_x, rot_y, rot_z, sc_x, sc_y, sc_z}, ...}
        let new_instanced_model_transform_holder = lua.create_function_mut(move |lua, (name, instance, transforms): (String, String, Vec<[f32; 9]>)| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    let transforms = transforms.iter().map(|transform| {
                        Transform {
                            position: Vec3::new(transform[0], transform[1], transform[2]),
                            rotation: Vec3::new(transform[3], transform[4], transform[5]),
                            scale: Vec3::new(transform[6], transform[7], transform[8]),
                        }
                    }).collect();
                    let object = InstancedModelTransformHolder::new(&name, &instance, transforms);
                    add_to_system_or_parent(lua, system, Box::new(object));
                },
                None => debugger::error("failed to call new_instanced_model_transform_holder, system not found"),
            }
            
            Ok(())
        });

        match new_instanced_model_transform_holder {
            Ok(func) => {
                if let Err(err) = lua.globals().set("new_instanced_model_transform_holder", func) {
                    debugger::error(&format!("failed to add a function new_instanced_model_transform_holder as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function new_instanced_model_transform_holder in system {}\nerror: {}",
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
                    None => debugger::error("failed to call new_character_controller, system not found"),
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
        let new_navigation_ground = lua.create_function_mut(move |lua, (name, size_x, size_z): (String, f32, f32)| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    let object = NavigationGround::new(&name, Vec2::new(size_x, size_z));
                    add_to_system_or_parent(lua, system, Box::new(object));
                },
                None => debugger::error("failed to call new_navigation_ground, system not found"),
            }
            
            Ok(())
        });

        match new_navigation_ground {
            Ok(func) => {
                if let Err(err) = lua.globals().set("new_navigation_ground", func) {
                    debugger::error(&format!("failed to add a function new_navigation_ground as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function new_navigation_ground in system {}\nerror: {}",
                system_id, err
            )),
        }



        let system_id_for_functions = system_id.clone();
        let new_ray = lua.create_function_mut(move |lua, (name, direction_x, direction_y, direction_z, mask_bits): (String, f32, f32, f32, Option<u32>)| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    let mask = match mask_bits {
                        Some(mask_bits) => Some(CollisionGroups::from(mask_bits)),
                        None => None,
                    };

                    let object = Ray::new(&name, Vec3::new(direction_x, direction_y, direction_z), mask);
                    add_to_system_or_parent(lua, system, Box::new(object));
                },
                None => debugger::error("failed to call new_ray, system not found"),
            }
            
            Ok(())
        });

        match new_ray {
            Ok(func) => {
                if let Err(err) = lua.globals().set("new_ray", func) {
                    debugger::error(&format!("failed to add a function new_ray as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function new_ray in system {}\nerror: {}",
                system_id, err
            )),
        }



        let system_id_for_functions = system_id.clone();
        let new_trigger = lua.create_function_mut(move |lua, (name, collider_type, size_x, size_y, size_z, membership_bits, mask_bits): (String, String, f32, f32, f32, Option<u32>, Option<u32>)| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    let membership = match membership_bits {
                        Some(membership_bits) => Some(CollisionGroups::from(membership_bits)),
                        None => None,
                    };
                    let mask = match mask_bits {
                        Some(mask_bits) => Some(CollisionGroups::from(mask_bits)),
                        None => None,
                    };

                    let collider = match collider_type.as_str() {
                        "Cuboid" => BodyColliderType::Cuboid(size_x, size_y, size_z),
                        "Capsule" => BodyColliderType::Capsule(size_x, size_y),
                        "Cylinder" => BodyColliderType::Cylinder(size_x, size_y),
                        "Ball" => BodyColliderType::Ball(size_x),
                        _ => {
                            debugger::error(
                                "lua error: new_trigger failed! the body_collider_type argument is wrong, possible values are 'None', 'Cuboid', 'Capsule', 'Cylinder', 'Ball'"
                            );
                            BodyColliderType::Cuboid(size_x, size_y, size_z)
                        },
                    };

                    let object = Trigger::new(&name, membership, mask, collider);
                    add_to_system_or_parent(lua, system, Box::new(object));
                },
                None => debugger::error("failed to call new_trigger, system not found"),
            }
            
            Ok(())
        });

        match new_trigger {
            Ok(func) => {
                if let Err(err) = lua.globals().set("new_trigger", func) {
                    debugger::error(&format!("failed to add a function new_trigger as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function new_trigger in system {}\nerror: {}",
                system_id, err
            )),
        }



        let system_id_for_functions = system_id.clone();
        let new_nav_obstacle = lua.create_function_mut(move |lua, (name, size_x, size_z): (String, f32, f32)| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    let object = NavObstacle::new(&name, Vec3::new(size_x, 1.0, size_z));
                    add_to_system_or_parent(lua, system, Box::new(object));
                },
                None => debugger::error("failed to call new_nav_obstacle, system not found"),
            }
            
            Ok(())
        });

        match new_nav_obstacle {
            Ok(func) => {
                if let Err(err) = lua.globals().set("new_nav_obstacle", func) {
                    debugger::error(&format!("failed to add a function new_nav_obstacle as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function new_nav_obstacle in system {}\nerror: {}",
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

        let system_id_for_functions = system_id.clone();

        let send_custom_message = lua.create_function_mut(move |_, (is_reliable, message_id, contents, receiver, client_id): (bool, String, String, Option<String>, Option<u64>)| {
            match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                Some(system) => {
                    let reliability = match is_reliable {
                        true => MessageReliability::Reliable,
                        false => MessageReliability::Unreliable,
                    };
                    if let networking::NetworkingMode::Server(_) = networking::get_current_networking_mode() {
                        match receiver {
                            Some(receiver) => {
                                let receiver = match receiver.as_str() {
                                    "Everybody" => MessageReceiver::Everybody,
                                    "OneClient" => {
                                        match client_id {
                                            Some(id) => MessageReceiver::OneClient(id),
                                            None => {
                                                debugger::error("lua send_custom_message error: message receiver set to 'Client', but client id is 0");
                                                MessageReceiver::OneClient(0)
                                            },
                                        }
                                    }, 
                                    "EverybodyExcept" => {
                                        match client_id {
                                            Some(id) => MessageReceiver::EverybodyExcept(id),
                                            None => {
                                                debugger::error("lua send_custom_message error: message receiver set to 'EverybodyExcept', but client id is 0");
                                                MessageReceiver::OneClient(0)
                                            },
                                        }
                                    },
                                    _ => {
                                        debugger::error("lua send_custom_message error: receiver arg is wrong! Possible values: 'Everybody', 'OneClient', 'EverybodyExcept'");
                                        MessageReceiver::OneClient(0)
                                    },
                                };
                                let message = Message::new_from_server(receiver, MessageContents::Custom(contents), system_id_for_functions.clone(), message_id);
                                let _ = system.send_message(reliability, message);
                            },
                            None => {
                                debugger::error("lua send_custom_message error: receiver arg is nil! Possible values: 'Everybody', 'OneClient', 'EverybodyExcept'");
                            },
                        };

                    } else if let networking::NetworkingMode::Client(_) = networking::get_current_networking_mode() {
                        dbg!(networking::get_id());
                        let message = Message::new_from_client(MessageContents::Custom(contents), system_id_for_functions.clone(), message_id);
                        let _ = system.send_message(reliability, message);
                    }
                }
                None => debugger::error("failed to call send_custom_message, system not found"),
            }

            Ok(())
        });

        match send_custom_message {
            Ok(func) => {
                if let Err(err) = lua.globals().set("send_custom_message", func) {
                    debugger::error(&format!("failed to add a function send_custom_message as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function send_custom_message in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let send_sync_object_message = lua.create_function_mut(move 
            |_, (is_reliable, message_id, object_name, pos, rot, scale, receiver, client_id): 
            (bool, String, String, [f32; 3], [f32; 3], [f32; 3], Option<String>, Option<u64>)| {
            let contents = MessageContents::SyncObject(SyncObjectMessage {
                object_name,
                transform: Transform {
                    position: pos.into(),
                    rotation: rot.into(),
                    scale: scale.into()
                },
            });
            match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                Some(system) => {
                    let reliability = match is_reliable {
                        true => MessageReliability::Reliable,
                        false => MessageReliability::Unreliable,
                    };
                    if let networking::NetworkingMode::Server(_) = networking::get_current_networking_mode() {
                        match receiver {
                            Some(receiver) => {
                                let receiver = match receiver.as_str() {
                                    "Everybody" => MessageReceiver::Everybody,
                                    "OneClient" => {
                                        match client_id {
                                            Some(id) => MessageReceiver::OneClient(id),
                                            None => {
                                                debugger::error("lua send_sync_object_message error: message receiver set to 'Client', but client id is 0");
                                                MessageReceiver::OneClient(0)
                                            },
                                        }
                                    }, 
                                    "EverybodyExcept" => {
                                        match client_id {
                                            Some(id) => MessageReceiver::EverybodyExcept(id),
                                            None => {
                                                debugger::error("lua send_sync_object_message error: message receiver set to 'EverybodyExcept', but client id is 0");
                                                MessageReceiver::OneClient(0)
                                            },
                                        }
                                    },
                                    _ => {
                                        debugger::error("lua send_sync_object_message error: receiver arg is wrong! Possible values: 'Everybody', 'OneClient', 'EverybodyExcept'");
                                        MessageReceiver::OneClient(0)
                                    },
                                };
                                let message = Message::new_from_server(receiver, contents, system_id_for_functions.clone(), message_id);
                                let _ = system.send_message(reliability, message);
                            },
                            None => {
                                debugger::error("lua send_sync_object_message error: receiver arg is nil! Possible values: 'Everybody', 'OneClient', 'EverybodyExcept'");
                            },
                        };

                    } else if let networking::NetworkingMode::Client(_) = networking::get_current_networking_mode() {
                        let message = Message::new_from_client(contents, system_id_for_functions.clone(), message_id);
                        let _ = system.send_message(reliability, message);
                    }
                }
                None => debugger::error("failed to call send_sync_object_message, system not found"),
            }

            Ok(())
        });

        match send_sync_object_message {
            Ok(func) => {
                if let Err(err) = lua.globals().set("send_sync_object_message", func) {
                    debugger::error(&format!("failed to add a function send_sync_object_message as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function send_sync_object_message in system {}\nerror: {}",
                system_id, err
            )),
        }


        

/*

        match send_sync_object_message_to_everybody {
            Ok(func) => {
                if let Err(err) = lua.globals().set("send_sync_object_message_to_everybody", func) {
                    debugger::error(&format!("failed to add a function send_sync_object_message_to_everybody as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function send_sync_object_message_to_everybody in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let send_sync_object_message_to_one_client = lua.create_function_mut(
            move |_, (client_id, is_reliable, message_id, object_name, pos, rot, scale): (u64, bool, String, String, [f32; 3], [f32; 3], [f32; 3])| {
            match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                Some(system) => {
                    let reliability = match is_reliable {
                        true => MessageReliability::Reliable,
                        false => MessageReliability::Unreliable,
                    };
                    let message = Message {
                        receiver: MessageReceiver::OneClient(client_id),
                        system_id: system.system_id().into(),
                        message_id,
                        contents: MessageContents::SyncObject(SyncObjectMessage {
                            object_name,
                            transform: Transform {
                                position: pos.into(),
                                rotation: rot.into(),
                                scale: scale.into()
                            },
                        }),
                    };

                    let _ = system.send_message(reliability, message);
                },
                None => debugger::error("failed to call send_sync_object_message_to_one_client, system not found"),
            }

            Ok(())
        });

        match send_sync_object_message_to_one_client {
            Ok(func) => {
                if let Err(err) = lua.globals().set("send_sync_object_message_to_one_client", func) {
                    debugger::error(&format!("failed to add a function send_sync_object_message_to_one_client as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function send_sync_object_message_to_one_client in system {}\nerror: {}",
                system_id, err
            )),
        }

        let system_id_for_functions = system_id.clone();
        let send_sync_object_message_to_everybody_except = lua.create_function_mut(
            move |_, (client_id, is_reliable, message_id, object_name, pos, rot, scale): (u64, bool, String, String, [f32; 3], [f32; 3], [f32; 3])| {
            match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                Some(system) => {
                    let reliability = match is_reliable {
                        true => MessageReliability::Reliable,
                        false => MessageReliability::Unreliable,
                    };
                    let message = Message {
                        receiver: MessageReceiver::EverybodyExcept(client_id),
                        system_id: system.system_id().into(),
                        message_id,
                        contents: MessageContents::SyncObject(SyncObjectMessage {
                            object_name,
                            transform: Transform {
                                position: pos.into(),
                                rotation: rot.into(),
                                scale: scale.into()
                            },
                        }),
                    };

                    let _ = system.send_message(reliability, message);
                }
                None => debugger::error("failed to call send_sync_object_message_to_everybody_except, system not found"),
            }

            Ok(())
        });

        match send_sync_object_message_to_everybody_except {
            Ok(func) => {
                if let Err(err) = lua.globals().set("send_sync_object_message_to_everybody_except", func) {
                    debugger::error(&format!("failed to add a function send_sync_object_message_to_everybody_except as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function send_sync_object_message_to_everybody_except in system {}\nerror: {}",
                system_id, err
            )),
        }*/

        //let system_id_for_functions = system_id.clone();
        let get_network_events = lua.create_function_mut(
            move |_, _: ()| {
                let mut events: Vec<HashMap<&str, String>> = Vec::new();
                for ev in networking::get_network_events() {
                    match ev {
                        networking::NetworkEvent::ClientConnected(id) => {
                            let mut ev = HashMap::new();
                            ev.insert("type", "ClientConnected".into());
                            ev.insert("id", id.to_string());
                            events.push(ev);
                        },
                        networking::NetworkEvent::ClientDisconnected(id, reason) => {
                            let mut ev = HashMap::new();
                            ev.insert("type", "ClientDisconnected".into());
                            ev.insert("id", id.to_string());
                            ev.insert("reason", reason.clone());
                            events.push(ev);
                        },
                        networking::NetworkEvent::ConnectedSuccessfully => {
                            let mut ev = HashMap::new();
                            ev.insert("type", "ConnectedSuccessfully".into());
                            events.push(ev);
                        },
                        networking::NetworkEvent::Disconnected(_) => {
                            let mut ev = HashMap::new();
                            ev.insert("type", "Disconnected".into());
                            events.push(ev);
                        },
                    }
                }
                Ok(events)
            }
        );

        match get_network_events {
            Ok(func) => {
                if let Err(err) = lua.globals().set("get_network_events", func) {
                    debugger::error(&format!("failed to add a function get_network_events as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function get_network_events in system {}\nerror: {}",
                system_id, err
            )),
        }

        let get_value_in_system = lua.create_function_mut(
            move |_, (system_id, value_name): (String, String)| {
                Ok(managers::systems::get_value_in_system(&system_id, value_name))
            }
        );

        match get_value_in_system {
            Ok(func) => {
                if let Err(err) = lua.globals().set("get_value_in_system", func) {
                    debugger::error(&format!("failed to add a function get_value_in_system as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function get_value_in_system in system {}\nerror: {}",
                system_id, err
            )),
        }

        let get_global_system_value = lua.create_function_mut(
            move |_, name: String| {
                Ok(framework::get_global_system_value(&name))
            }
        );

        match get_global_system_value {
            Ok(func) => {
                if let Err(err) = lua.globals().set("get_global_system_value", func) {
                    debugger::error(&format!("failed to add a function get_global_system_value as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function get_global_system_value in system {}\nerror: {}",
                system_id, err
            )),
        }

        let set_global_system_value = lua.create_function_mut(
            move |_, (name, value): (String, Vec<SystemValue>)| {
                Ok(framework::set_global_system_value(&name, value))
            }
        );

        match set_global_system_value {
            Ok(func) => {
                if let Err(err) = lua.globals().set("set_global_system_value", func) {
                    debugger::error(&format!("failed to add a function set_global_system_value as a lua global in system {}\nerror: {}", system_id, err));
                }
            }
            Err(err) => debugger::error(&format!(
                "failed to create a function set_global_system_value in system {}\nerror: {}",
                system_id, err
            )),
        }
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
                    debugger::error("lua error: failed to add object! failed to get the current_parent object!");
                }
            }
        }
    }
    system.add_object(object);
}
