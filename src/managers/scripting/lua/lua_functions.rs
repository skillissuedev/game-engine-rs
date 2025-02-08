use std::collections::HashMap;

use super::{ObjectHandle, SYSTEMS_LUA_VMS};
use crate::{
    assets::{
        self,
        shader_asset::{ShaderAsset, ShaderAssetPath},
    }, managers::{
        self, debugger::{self, error}, networking::{self, Message, MessageContents, MessageReceiver, MessageReliability, SyncObjectMessage}, physics::{BodyColliderType, CollisionGroups}, render::RenderLayer, scripting::lua::{get_framework_pointer, LuaSpline}, systems::{self, SystemValue}
    }, math_utils, objects::{
        Object, Transform
    }, systems::System
};
use egui_glium::egui_winit::egui::TextBuffer;
use glam::Vec3;
use mlua::Lua;
use splines::Spline;

macro_rules! add_function {
    ($name:literal, $function:expr, $lua:expr, $system_id:expr) => {
        match $function {
            Ok(function) => {
                if let Err(err) = $lua.globals().set($name, function) {
                    debugger::error(&format!("failed to add a function {} as a lua global in system {}\nerror: {}", $name, $system_id, err));
                }
            }
            Err(err) => debugger::error(
                &format!(
                    "failed to create a function {} in system {}\nerror: {}",
                    $name, $system_id, err
                )
            ),
        }
    };
}

pub fn add_lua_vm_to_list(system_id: String, lua: Lua) {
    unsafe {
        SYSTEMS_LUA_VMS.insert(system_id.clone(), lua);
        let lua = SYSTEMS_LUA_VMS.get_mut(&system_id).unwrap();
        let _ = lua.globals().set("current_parent", None::<String>);

        // creating some functions
        // splines
        let system_id_for_functions = system_id.clone();
        let new_spline = lua.create_function(move |_,
                (t, v, interpolation_types, interpolation_values): (Vec<f32>, Vec<f32>, Option<Vec<String>>, Option<Vec<f32>>)| {
            let mut keys: Vec<splines::Key<f32, f32>> = Vec::new();
            for (idx, t) in t.iter().enumerate() {
                let v = v.get(idx);
                match v {
                    Some(v) => {
                        let interpolation = match &interpolation_types {
                            Some(interpolation) => match interpolation.get(idx) {
                                Some(interpolation) => match interpolation.to_lowercase().as_str() {
                                    "linear" => splines::Interpolation::Linear,
                                    "cosine" => splines::Interpolation::Cosine,
                                    "catmullrom" => splines::Interpolation::CatmullRom,
                                    "bezier" => {
                                        let interpolation_value = match &interpolation_values {
                                            Some(interpolation_values) => match interpolation_values.get(idx) {
                                                Some(interpolation_value) => *interpolation_value,
                                                None => {
                                                    debugger::warn("new_spline warning! Check the fourth argument (interpolation_values)");
                                                    println!("lua system id of the spline error above = {}", system_id_for_functions);
                                                    1.0
                                                },
                                            }
                                            None => {
                                                debugger::warn("new_spline warning! Check the fourth argument (interpolation_values)");
                                                println!("lua system id of the spline error above = {}", system_id_for_functions);
                                                1.0
                                            },
                                        };
                                        splines::Interpolation::Bezier(interpolation_value)
                                    },
                                    _ => {
                                        debugger::warn("new_spline warning! Check the third argument (interpolation_types)");
                                        println!("lua system id of the spline error above = {}", system_id_for_functions);
                                        splines::Interpolation::Linear
                                    },
                                },
                                None => {
                                    debugger::warn("new_spline warning! Check the third argument (interpolation_types)");
                                    println!("lua system id of the spline error above = {}", system_id_for_functions);
                                    splines::Interpolation::Linear
                                },
                            },
                            None => splines::Interpolation::Linear,
                        };
                        keys.push(splines::Key::new(*t, *v, interpolation))
                    },
                    None => {
                        debugger::error("new_spline failed! Check the second argument (values)");
                        println!("lua system id of the spline error above = {}", system_id_for_functions);
                        return Ok(None)
                    },
                }
            };
            Ok(Some(LuaSpline(Spline::from_vec(keys))))
        });
        add_function!("new_spline", new_spline, lua, &system_id);

        // setting/crearing current parent
        let system_id_for_functions = system_id.clone();
        let set_current_parent = lua.create_function(move |lua, name: String| {
            if let Err(err) = lua.globals().set("current_parent", Some(name)) {
                debugger::error(&format!(
                        "lua error: failed to set current_parent! 
                        system: {}\nerr: {:?}",
                        system_id_for_functions, err
                ));
            }
            Ok(())
        });
        add_function!("set_current_parent", set_current_parent, lua, system_id);

        let system_id_for_functions = system_id.clone();
        let clear_current_parent = lua.create_function(move |lua, _: ()| {
            if let Err(err) = lua.globals().set("current_parent", None::<String>) {
                debugger::error(
                    &format!(
                        "lua error: failed to set current_parent! system: {}\nerr: {:?}",
                        system_id_for_functions, err
                    )
                );
            }
            Ok(())
        });
        add_function!("clear_current_parent", clear_current_parent, lua, &system_id);

        // delete and find objects
        let system_id_for_functions = system_id.clone();
        let delete_object = lua.create_function(move |_, name: String| {
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            let framework_ptr = get_framework_pointer();
            let framework = &mut *framework_ptr;
            match system_option {
                Some(system) =>
                    return Ok(system.delete_object(framework, &name)),
                None => {
                    debugger::error("failed to call delete_object: system not found");
                }
            }
            Ok(())
        });
        add_function!("delete_object", delete_object, lua, system_id);

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
        add_function!("find_object", find_object, lua, system_id);

        let rotate_vector = lua.create_function(move |_, (dir_x, dir_y, dir_z, rot_x, rot_y, rot_z): (f32, f32, f32, f32, f32, f32)| {
            let vec = math_utils::rotate_vector(Vec3::new(dir_x, dir_y, dir_z), Vec3::new(rot_x, rot_y, rot_z));
            Ok([vec.x, vec.y, vec.z])
        });
        add_function!("rotate_vector", rotate_vector, lua, system_id);

        let system_id_for_functions = system_id.clone();
        let does_object_exist = lua.create_function(move |_, name: String| {
                match systems::get_system_mut_with_id(&system_id_for_functions) {
                    Some(system) => match system.find_object(&name) {
                        Some(object) => {
                            for child in object.children_list() {
                                if child.name() == name {
                                    return Ok(true)
                                }
                            }
                            if object.name() == name {
                                return Ok(true)
                            }

                            return Ok(false)
                        },
                        None => return Ok(false),
                    },
                    None => return Ok(false)
                }
            },
        );
        add_function!("does_object_exist", does_object_exist, lua, system_id);


        // creating new objects
        let system_id_for_functions = system_id.clone();
        let new_character_controller = lua.create_function_mut(move |lua, (name, shape, membership_groups, mask, size_x, size_y, size_z):
            (String, String, Option<u32>, Option<u32>, f32, f32, f32)| {
                let collider = match shape.as_str() {
                    "Cuboid" => BodyColliderType::Cuboid(size_x, size_y, size_z),
                    "Capsule" => BodyColliderType::Capsule(size_x, size_y),
                    "Cylinder" => BodyColliderType::Cylinder(size_x, size_y),
                    "Ball" => BodyColliderType::Ball(size_x),
                    _ => {
                        // error here
                        BodyColliderType::Capsule(size_x, size_y)
                    }
                };

                let membership_groups = match membership_groups {
                    Some(groups) => Some(CollisionGroups::from(groups)),
                    None => None,
                };

                let mask = match mask {
                    Some(groups) => Some(CollisionGroups::from(groups)),
                    None => None,
                };
                let framework_ptr = get_framework_pointer();
                let framework = &mut *framework_ptr;
                let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
                match system_option {
                    Some(system) => {
                        let object =
                            framework.new_character_controller_object(&name, collider, membership_groups, mask);
                        add_to_system_or_parent(lua, system, Box::new(object));
                    },
                    None => debugger::error("Lua error!\nFailed to call new_character_controller: system not found"),
            }
                Ok(())
            }
        );
        add_function!("new_character_controller", new_character_controller, lua, &system_id);

        let system_id_for_functions = system_id.clone();
        let new_empty_object = 
            lua.create_function_mut(move |lua, name: String| {
                let framework_ptr = get_framework_pointer();
                let framework = &mut *framework_ptr;
                let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
                match system_option {
                    Some(system) => {
                        let object = framework.new_empty_object(&name);
                        add_to_system_or_parent(lua, system, Box::new(object));
                    },
                    None => debugger::error("Lua error!\nFailed to call new_empty_object: system not found"),
            }
                Ok(())
            }
            );
        add_function!("new_empty_object", new_empty_object, lua, &system_id);

        let system_id_for_functions = system_id.clone();
        let new_sound_emitter_object = lua.create_function_mut(move |lua, (name, asset_id, should_loop, is_positional, max_distance):
            (String, String, bool, bool, f32)| {
                let system_option = systems::get_system_mut_with_id(&system_id_for_functions);

                let framework_ptr = get_framework_pointer();
                let framework = &mut *framework_ptr;
                match system_option {
                    Some(system) => {
                        let id = framework.get_sound_asset(&asset_id);
                        match id {
                            Some(id) => {
                                let mut object = framework.new_sound_emitter(&name, id, is_positional);
                                if is_positional {
                                    let _ = object.set_max_distance(max_distance);
                                }
                                object.set_looping(should_loop);
                                add_to_system_or_parent(lua, system, Box::new(object));
                            },
                            None => todo!(),
                        }
                    },
                    None => debugger::error("failed to call new_sound_emitter_object, system not found"),
                }

                Ok(())
            });
        add_function!("new_sound_emitter_object", new_sound_emitter_object, lua, &system_id);

        let system_id_for_functions = system_id.clone();
        let new_model_object = lua.create_function_mut(
            move |lua, (name, model_asset_id, texture_asset_id, vertex_shader_asset_path, fragment_shader_asset_path, is_transparent, layer):
            (String, String, Option<String>, Option<String>, Option<String>, bool, Option<u8>)| {
                let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
                let framework_ptr = get_framework_pointer();
                let framework = &mut *framework_ptr;
                match system_option {
                    Some(system) => {
                        let texture_asset;
                        match texture_asset_id {
                            Some(id) => {
                                let asset = framework.get_texture_asset(&id);
                                match asset {
                                    Some(asset) => texture_asset = Some(asset),
                                    None => {
                                        debugger::warn("lua warning: error when calling new_model_object, failed to get preloaded texture asset!");
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
                        let shader_asset = ShaderAsset::load_from_file(&shader_asset_path);
                        match shader_asset {
                            Ok(shader_asset) => {
                                let model_asset = framework.get_model_asset(&model_asset_id);
                                match model_asset {
                                    Some(model_asset) => {
                                        let layer = match layer {
                                            Some(layer) => match layer {
                                                0 => RenderLayer::Layer1,
                                                1 => RenderLayer::Layer1,
                                                2 => RenderLayer::Layer2,
                                                _ => {
                                                    debugger::error(
                                                        &format!("{} {}",
                                                            "lua error: new_model_object: Failed to convert integer to a RenderLayer!",
                                                            "The number should be >= 0 and <= 2. Defaulting to Layer 1"
                                                        )
                                                    );
                                                    RenderLayer::Layer1
                                                },
                                            },
                                            None => RenderLayer::Layer1
                                        };

                                        let object = framework.new_model_object(&name, model_asset, texture_asset, shader_asset, is_transparent, layer);
                                        add_to_system_or_parent(lua, system, Box::new(object));
                                    },
                                    None => 
                                        debugger::error("lua error: error when calling new_model_object, failed to get the model asset!"),
                                }
                            },
                            Err(err) => 
                                debugger::error(&format!("lua error: error when calling new_model_object, failed to load the shader asset!\nerr: {:?}", err)),
                        }
                    },
                    None => debugger::error("failed to call new_model_object, system not found"),
                }
                Ok(())
            });
        add_function!("new_model_object", new_model_object, lua, &system_id);

        let system_id_for_functions = system_id.clone();
        let new_master_instanced_model_object = lua.create_function_mut(
            move |lua, (name, model_asset_id, texture_asset_id, vertex_shader_asset_path, fragment_shader_asset_path, is_transparent, layer):
            (String, String, Option<String>, Option<String>, Option<String>, bool, Option<u8>)| {
                let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
                let framework_ptr = get_framework_pointer();
                let framework = &mut *framework_ptr;
                match system_option {
                    Some(system) => {
                        let texture_asset;
                        match texture_asset_id {
                            Some(id) => {
                                let asset = framework.get_texture_asset(&id);
                                match asset {
                                    Some(asset) => texture_asset = Some(asset),
                                    None => {
                                        debugger::warn("lua warning: error when calling new_master_instanced_model_object, failed to get preloaded texture asset!");
                                        texture_asset = None;
                                    },
                                }
                            },
                            None => texture_asset = None,
                        }
                        let mut shader_asset_path = ShaderAssetPath {
                            vertex_shader_path: assets::shader_asset::get_default_instanced_vertex_shader_path(),
                            fragment_shader_path: assets::shader_asset::get_default_instanced_fragment_shader_path(),
                        };
                        if let Some(vertex_shader_asset_path) = vertex_shader_asset_path {
                            shader_asset_path.vertex_shader_path = vertex_shader_asset_path;
                        }
                        if let Some(fragment_shader_asset_path) = fragment_shader_asset_path {
                            shader_asset_path.fragment_shader_path = fragment_shader_asset_path;
                        }
                        let shader_asset = ShaderAsset::load_from_file(&shader_asset_path);
                        match shader_asset {
                            Ok(shader_asset) => {
                                let model_asset = framework.get_model_asset(&model_asset_id);
                                match model_asset {
                                    Some(model_asset) => {
                                        let layer = match layer {
                                            Some(layer) => match layer {
                                                0 => RenderLayer::Layer1,
                                                1 => RenderLayer::Layer1,
                                                2 => RenderLayer::Layer2,
                                                _ => {
                                                    debugger::error(
                                                        &format!("{} {}",
                                                            "lua error: new_model_object: Failed to convert integer to a RenderLayer!",
                                                            "The number should be >= 0 and <= 2. Defaulting to Layer 1"
                                                        )
                                                    );
                                                    RenderLayer::Layer1
                                                },
                                            },
                                            None => RenderLayer::Layer1
                                        };

                                        let object = 
                                            framework.new_master_instanced_model_object(&name, model_asset, texture_asset, shader_asset, is_transparent, layer);
                                        add_to_system_or_parent(lua, system, Box::new(object));
                                    },
                                    None => 
                                        debugger::error("lua error: error when calling new_master_instanced_model_object, failed to get the model asset!"),
                                }
                            },
                            Err(err) => {
                                debugger::error(
                                    &format!("lua error: error when calling new_master_instanced_model_object, failed to load the shader asset!\nerr: {:?}", err)
                                )
                            },
                        }
                    },
                    None => debugger::error("failed to call new_master_instanced_model_object, system not found"),
                }
                Ok(())
            });
        add_function!("new_master_instanced_model_object", new_master_instanced_model_object, lua, system_id);

        let system_id_for_functions = system_id.clone();
        let new_instanced_model_object = lua.create_function_mut(move |lua, (name, instance): (String, String)| {
            let framework_ptr = get_framework_pointer();
            let framework = &mut *framework_ptr;
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    let object = framework.new_instanced_model_object(&name, &instance);
                    add_to_system_or_parent(lua, system, Box::new(object));
                },
                None => debugger::error("failed to call new_instanced_model_object, system not found"),
            }

            Ok(())
        });
        add_function!("new_instanced_model_object", new_instanced_model_object, lua, system_id);

        let system_id_for_functions = system_id.clone();
        let new_instanced_model_transform_holder = lua.create_function_mut(move 
            |lua, (name, instance, transforms): (String, String, Vec<[f32; 9]>)| {
                let framework_ptr = get_framework_pointer();
                let framework = &mut *framework_ptr;
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
                        let object = framework.new_instanced_model_transform_holder(&name, &instance, transforms);
                        add_to_system_or_parent(lua, system, Box::new(object));
                    },
                    None => debugger::error("failed to call new_instanced_model_transform_holder, system not found"),
                }
                Ok(())
            });
        add_function!("new_instanced_model_transform_holder", new_instanced_model_transform_holder, lua, system_id);

        let system_id_for_functions = system_id.clone();
        let new_navigation_ground = lua.create_function_mut(move |lua, (name, size_x, size_z): (String, f32, f32)| {
            let framework_ptr = get_framework_pointer();
            let framework = &mut *framework_ptr;
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    let object = framework.new_navigation_ground(&name, Vec3::new(size_x, 1.0, size_z));
                    add_to_system_or_parent(lua, system, Box::new(object));
                },
                None => debugger::error("failed to call new_navigation_ground, system not found"),
            }

            Ok(())
        });
        add_function!("new_navigation_ground", new_navigation_ground, lua, system_id);

        let system_id_for_functions = system_id.clone();
        let new_ray = lua.create_function_mut(
            move |lua, (name, direction_x, direction_y, direction_z, mask_bits): (String, f32, f32, f32, Option<u32>)| {
                let framework_ptr = get_framework_pointer();
                let framework = &mut *framework_ptr;
                let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
                match system_option {
                    Some(system) => {
                        let mask = match mask_bits {
                            Some(mask_bits) => Some(CollisionGroups::from(mask_bits)),
                            None => None,
                        };

                        let object = framework.new_ray(&name, Vec3::new(direction_x, direction_y, direction_z), mask);
                        add_to_system_or_parent(lua, system, Box::new(object));
                    },
                    None => debugger::error("failed to call new_ray, system not found"),
                }

                Ok(())
            });
        add_function!("new_ray", new_ray, lua, system_id);

        let system_id_for_functions = system_id.clone();
        let new_trigger = lua.create_function_mut(move |lua, (name, collider_type, size_x, size_y, size_z, membership_bits, mask_bits): 
            (String, String, f32, f32, f32, Option<u32>, Option<u32>)| {
                let possible_collider_val_err =
                    "lua error: new_trigger failed! the body_collider_type argument is wrong, possible values are 'None', 'Cuboid', 'Capsule', 'Cylinder', 'Ball'";
                let framework_ptr = get_framework_pointer();
                let framework = &mut *framework_ptr;
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
                                debugger::error(possible_collider_val_err);
                                BodyColliderType::Cuboid(size_x, size_y, size_z)
                            },
                        };

                        let object = framework.new_trigger(&name, membership, mask, collider);
                        add_to_system_or_parent(lua, system, Box::new(object));
                    },
                    None => debugger::error("failed to call new_trigger, system not found"),
                }

                Ok(())
            });

        add_function!("new_trigger", new_trigger, lua, system_id);

        let system_id_for_functions = system_id.clone();
        let new_nav_obstacle = lua.create_function_mut(move |lua, (name, size_x, size_z): (String, f32, f32)| {
            let framework_ptr = get_framework_pointer();
            let framework = &mut *framework_ptr;
            let system_option = systems::get_system_mut_with_id(&system_id_for_functions);
            match system_option {
                Some(system) => {
                    let object = framework.new_nav_obstacle(&name, Vec3::new(size_x, 1.0, size_z));
                    add_to_system_or_parent(lua, system, Box::new(object));
                },
                None => debugger::error("failed to call new_nav_obstacle, system not found"),
            }

            Ok(())
        });

        add_function!("new_nav_obstacle", new_nav_obstacle, lua, system_id);

        let system_id_for_functions = system_id.clone();
        let set_object_position = lua.create_function_mut(
            move |_, (name, pos_x, pos_y, pos_z): (String, f32, f32, f32)| {
                match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                    Some(system) => {
                        let framework_ptr = get_framework_pointer();
                        let framework = &mut *framework_ptr;
                        let object_option = system.find_object_mut(&name);
                        match object_option {
                            Some(object) => {
                                object.set_position(framework, Vec3::new(pos_x, pos_y, pos_z), true)
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
        add_function!("set_object_position", set_object_position, lua, system_id);

        let system_id_for_functions = system_id.clone();
        let set_object_rotation = lua.create_function_mut(
            move |_, (name, rot_x, rot_y, rot_z): (String, f32, f32, f32)| {
                match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                    Some(system) => {
                        let framework_ptr = get_framework_pointer();
                        let framework = &mut *framework_ptr;
                        let object_option = system.find_object_mut(&name);
                        match object_option {
                            Some(object) => {
                                object.set_rotation(framework, Vec3::new(rot_x, rot_y, rot_z), true)
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
        add_function!("set_object_rotation", set_object_rotation, lua, system_id);

        let system_id_for_functions = system_id.clone();
        let set_object_scale = lua.create_function_mut(
            move |_, (name, sc_x, sc_y, sc_z): (String, f32, f32, f32)| {
                match managers::systems::get_system_mut_with_id(&system_id_for_functions) {
                    Some(system) => {
                        let object_option = system.find_object_mut(&name);
                        match object_option {
                            Some(object) => {
                                object.set_scale(Vec3::new(sc_x, sc_y, sc_z))
                            }
                            None => debugger::error(
                                "failed to call set_object_rotation, object not found",
                            ),
                        }
                    },
                    None => debugger::error("failed to call set_object_rotation, system not found"),
                }
                Ok(())
            }
        );
        add_function!("set_object_scale", set_object_scale, lua, system_id);

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
        add_function!("get_object_position", get_object_position, lua, system_id);


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
        add_function!("get_object_rotation", get_object_rotation, lua, system_id);

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
        add_function!("get_object_scale", get_object_scale, lua, system_id);

        let system_id_for_functions = system_id.clone();
        let send_custom_message = lua.create_function_mut(move |_, (is_reliable, message_id, contents, receiver, client_id): (bool, String, Vec<SystemValue>, Option<String>, Option<u64>)| {
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
                        let message = Message::new_from_client(MessageContents::Custom(contents), system_id_for_functions.clone(), message_id);
                        let _ = system.send_message(reliability, message);
                    }
                }
                None => debugger::error("failed to call send_custom_message, system not found"),
            }

            Ok(())
        });
        add_function!("send_custom_message", send_custom_message, lua, system_id);

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
        add_function!("send_sync_object_message", send_sync_object_message, lua, system_id);


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
        add_function!("get_network_events", get_network_events, lua, system_id);
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
