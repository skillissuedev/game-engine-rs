pub mod lua_functions;
use crate::{
    framework::{self, DebugMode, Framework}, managers::{
        assets, debugger, networking::{Message, MessageContents}, physics::{BodyColliderType, BodyType, CollisionGroups, RenderColliderType}, render::RenderUniformValue, scripting::lua::lua_functions::add_lua_vm_to_list, systems::{self, CallList, SystemValue}
    }, math_utils::{self, PerlinNoise}, objects::{character_controller::CharacterController, master_instanced_model_object::MasterInstancedModelObject, model_object::ModelObject, particle_system::ParticleSystem, ray::Ray, sound_emitter::SoundEmitter, trigger::Trigger}, systems::System
};
use crate::objects::Object;
use glam::{Vec2, Vec3};
use mlua::{Error, FromLua, FromLuaMulti, Function, IntoLua, Lua, LuaOptions, StdLib, UserData};
use once_cell::sync::Lazy;
use splines::Spline;
use std::{collections::HashMap, fs};

static mut SYSTEMS_LUA_VMS: Lazy<HashMap<String, Lua>> = Lazy::new(|| HashMap::new()); // String is system's id and Lua is it's vm

#[derive(Debug)]
pub struct LuaSystem {
    pub is_destroyed: bool,
    pub id: String,
    pub objects: Vec<Box<dyn Object>>,
}

impl LuaSystem {
    pub fn new(id: &str, script_path: &str) -> Result<LuaSystem, LuaSystemError> {
        match fs::read_to_string(assets::get_full_asset_path(&script_path)) {
            Ok(script) => {
                let lua: Lua = match Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()) {
                    Ok(lua) => lua,
                    Err(err) => {
                        debugger::error(&format!(
                                "lua system creation error!\nlua creation error\nerror: {}",
                                err
                        ));
                        return Err(LuaSystemError::ScriptLoadingError);
                    }
                };

                let load_result = lua.load(script).exec();

                match load_result {
                    Ok(_) => {
                        let system = LuaSystem {
                            is_destroyed: false,
                            id: id.into(),
                            objects: Vec::new(),
                        };

                        add_lua_vm_to_list(id.into(), lua);

                        Ok(system)
                    }
                    Err(err) => {
                        debugger::error(&format!(
                                "lua system creation error!\nlua execution error\nerror: {}",
                                err
                        ));
                        Err(LuaSystemError::ScriptLoadingError)
                    }
                }
            }
            Err(err) => {
                debugger::error(&format!("lua system creation error!\nerror: {}", err));
                Err(LuaSystemError::ScriptLoadingError)
            }
        }
    }
}

impl System for LuaSystem {
    fn client_start(&mut self, framework: &mut Framework) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "client_start", Some(framework));
            }
            None => debugger::error("lua system client_start function error\ncan't get lua vm reference"),
        }
    }

    fn server_start(&mut self, framework: &mut Framework) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "server_start", Some(framework));
            }
            None => debugger::error("lua system server_start function error\ncan't get lua vm reference"),
        }
    }

    fn client_update(&mut self, framework: &mut Framework) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "client_update", Some(framework));
                let _ = lua.gc_collect();
                let _ = lua.gc_collect();
            }
            None => debugger::error("lua system client_update function error\ncan't get lua vm reference"),
        }
    }

    fn server_update(&mut self, framework: &mut Framework) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "server_update", Some(framework));
            }
            None => debugger::error("lua system server_update function error\ncan't get lua vm reference"),
        }
    }

    fn server_render(&mut self) {
        /*
           let lua_option = lua_vm_ref(self.system_id().into());
           match lua_option {
           Some(lua) => {
           let _ = call_lua_function(self.system_id(), &lua, "server_render", None);
           }
           None => debugger::error("lua system server_render function error\ncan't get lua vm reference"),
           }
           do nothing lol
           */
    }

    fn client_render(&mut self, framework: &mut Framework) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "client_render", Some(framework));
            }
            None => debugger::error("lua system client_render function error\ncan't get lua vm reference"),
        }
    }

    fn call(&self, call_id: &str) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "call", None);
            }
            None => debugger::error(&format!(
                    "lua system call function error(call_id: {})\ncan't get lua vm reference",
                    call_id
            )),
        }
    }

    fn call_mut(&mut self, call_id: &str) {
        debugger::warn(&format!(
                "lua system {} warning when calling {}\nyou can just use call, instead of call_mut",
                self.id, call_id
        ));

        self.call(call_id);
    }

    fn objects_list(&self) -> &Vec<Box<dyn Object>> {
        &self.objects
    }

    fn objects_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.objects
    }

    fn call_list(&self) -> crate::managers::systems::CallList {
        // TODO
        CallList {
            immut_call: vec![],
            mut_call: vec![],
        }
    }

    fn system_id(&self) -> &str {
        &self.id
    }

    fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    fn set_destroyed(&mut self, is_destroyed: bool) {
        self.is_destroyed = is_destroyed;
    }

    fn reg_message(&mut self, message: Message) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let scope = lua.scope(|scope| {
                    let framework_ptr: *mut Framework = unsafe { framework::FRAMEWORK_POINTER } as *mut Framework;
                    let framework = unsafe { &mut *framework_ptr };
                    let framework_userdata = scope.create_userdata_ref_mut(framework);

                    let function_result: Result<Function, mlua::Error> = lua.globals().get("reg_message");

                    match function_result {
                        Ok(func) => {
                            let call_result: Result<(), mlua::Error> = Function::call(&func, (message, framework_userdata));
                            if let Err(err) = call_result {
                                debugger::error(
                                    &format!(
                                        "lua error when calling reg_message in system {}\nerror: {}", self.system_id(), err
                                    )
                                );
                            }
                        }
                        Err(err) => debugger::error(&format!("can't get function reg_message in lua system {}\nerror: {}", self.system_id(), err)),
                    }

                    Ok(())
                });
                if let Err(err) = scope {
                    debugger::error(&format!("reg_message function error!\nFailed to create Lua scope! Err: {}", err));
                }
            }
            None => debugger::error("lua system reg_message function error\ncan't get lua vm reference"),
        }
    }

    fn get_value(&mut self, value_name: String) -> Option<SystemValue> {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let func: Result<Function, mlua::Error> = lua.globals().get("get_value");

                match func {
                    Ok(func) => {
                        let call_result: Result<SystemValue, mlua::Error> = Function::call(&func, value_name);
                        return match call_result {
                            Ok(value) => Some(value),
                            Err(err) => {
                                debugger::error(
                                    &format!("lua error when calling get_value in system {}\nerror: {}", self.system_id(), err)
                                );

                                None
                            },
                        }
                    },
                    Err(err) => debugger::error(&format!("can't get function get_value in lua system {}\nerror: {}", self.system_id(), err)),
                }
            },
            None => debugger::error("lua system get_value function error\ncan't get lua vm reference"),
        }
        None
    }
}

fn lua_vm_ref<'a>(system_id: String) -> Option<&'a Lua> {
    unsafe { SYSTEMS_LUA_VMS.get(&system_id) }
}

fn lua_vm_ref_mut<'a>(system_id: String) -> Option<&'a mut Lua> {
    unsafe { SYSTEMS_LUA_VMS.get_mut(&system_id) }
}

fn call_lua_function(system_id: &str, lua: &Lua, function_name: &str, framework: Option<&mut Framework>) {
    let _ = lua.scope(|scope| {
        let framework_userdata = match framework {
            Some(framework) => Some(scope.create_userdata_ref_mut(framework)),
            None => None
        };

        if let Some(framework_userdata) = framework_userdata {
            match framework_userdata {
                Ok(framework_userdata) => {
                    let function_result: Result<Function, mlua::Error> = lua.globals().get(function_name);

                    match function_result {
                        Ok(func) => {
                            let call_result: Result<(), Error> = Function::call(&func, framework_userdata);

                            if let Err(err) = call_result {
                                debugger::error(
                                    &format!(
                                        "lua error when calling '{}' in system {}\nerror: {}",
                                        function_name, system_id, err
                                    )
                                );
                                return Err(err);
                            }
                        }
                        Err(err) => {
                            debugger::error(
                                &format!(
                                    "can't get function '{}' in lua system {}\nerror: {}",
                                    function_name, system_id, err
                                )
                            );
                        }
                    }
                },
                Err(err) => 
                    debugger::error(
                        &format!(
                            "Failed to call server_start!\nFailed to convert Framework to userdata\nErr: {}",
                            err
                        )
                    ),
            };
        } else {
            let function_result: Result<Function, mlua::Error> = lua.globals().get(function_name);

            match function_result {
                Ok(func) => {
                    let call_result: Result<(), Error> = Function::call(&func, ());

                    if let Err(err) = call_result {
                        debugger::error(&format!(
                                "lua error when calling '{}' in system {}\nerror: {}",
                                function_name, system_id, err
                        ));
                        return Err(err);
                    }
                }
                Err(err) => {
                    debugger::error(&format!(
                            "can't get function '{}' in lua system {}\nerror: {}",
                            function_name, system_id, err
                    ));
                }
            }
        }

        Ok(())
    });
}

#[derive(Debug)]
pub enum LuaSystemError {
    ScriptLoadingError,
}

pub struct ObjectHandle {
    pub system_id: String,
    pub name: String,
}

impl UserData for ObjectHandle {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(_: &mut F) {}

    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // methods that work for all objects:
        methods.add_method("children_list", |_, this, (): ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => {
                    match system.find_object_mut(&this.name) {
                        Some(object) => {
                            let mut handles = Vec::new();
                            for child in object.children_list() {
                                let child_handle = ObjectHandle {
                                    system_id: this.system_id.clone(),
                                    name: child.name().into()
                                };
                                handles.push(child_handle);
                            }
                            Ok(Some(handles))
                        },
                        None => {
                            debugger::error(&format!("lua error: children_list failed! failed to get object {} in system {}", this.name, this.system_id));
                            Ok(None)
                        }
                    }
                },
                None => {
                    debugger::error(&format!("lua error: children_list failed! failed to get system {} to find object {}", this.system_id, this.name));
                    Ok(None)
                },
            }
        });

        methods.add_method(
            "name",
            |_, this, _: ()| {
                Ok(this.name.clone())
            }
        );

        methods.add_method(
            "object_type",
            |_, this, _: ()| {
                match systems::get_system_mut_with_id(&this.system_id) {
                    Some(system) => match system.find_object_mut(&this.name) {
                        Some(object) => return Ok(Some(object.object_type().to_string())),
                        None => {
                            debugger::error(
                                &format!("lua error: object_type failed! failed to get object {} in system {}", this.name, this.system_id));
                            return Ok(None)
                        },
                    },
                    None => debugger::error(&format!(
                            "lua error: object_type failed! failed to get system {} to find object {}",
                            this.system_id, this.name
                    )),
                }

                Ok(None)
            },
        );

        methods.add_method_mut("set_name", |_, this, name: String| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        object.set_name(&name);
                        this.name = name;
                    }
                    None => debugger::error(&format!(
                            "lua error: set_name failed! failed to get object {} in system {}",
                            this.name, this.system_id
                    )),
                },
                None => debugger::error(&format!(
                        "lua error: set_name failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(())
        });

        methods.add_method("get_position", |_, this, _: ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        let transform = object.local_transform();
                        return Ok(transform.position.to_array());
                    }
                    None => debugger::error(&format!(
                            "lua error: get_position failed! failed to get object {} in system {}",
                            this.name, this.system_id
                    )),
                },
                None => debugger::error(&format!(
                        "lua error: get_position failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok([0.0, 0.0, 0.0])
        });

        methods.add_method("get_rotation", |_, this, _: ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        let transform = object.local_transform();
                        return Ok(transform.rotation.to_array());
                    }
                    None => debugger::error(&format!(
                            "lua error: get_rotation failed! failed to get object {} in system {}",
                            this.name, this.system_id
                    )),
                },
                None => debugger::error(&format!(
                        "lua error: get_rotation failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok([0.0, 0.0, 0.0])
        });

        methods.add_method("get_scale", |_, this, _: ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        let transform = object.local_transform();
                        return Ok(transform.scale.to_array());
                    }
                    None => debugger::error(&format!(
                            "lua error: get_scale failed! failed to get object {} in system {}",
                            this.name, this.system_id
                    )),
                },
                None => debugger::error(&format!(
                        "lua error: get_scale failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok([0.0, 0.0, 0.0])
        });

        methods.add_method("get_global_position", |_, this, _: ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        let transform = object.global_transform();
                        return Ok(transform.position.to_array());
                    }
                    None => debugger::error(&format!(
                            "lua error: get_global_position failed! failed to get object {} in system {}",
                            this.name, this.system_id
                    )),
                },
                None => debugger::error(&format!(
                        "lua error: get_global_position failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok([0.0, 0.0, 0.0])
        });

        methods.add_method("get_global_rotation", |_, this, _: ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        let transform = object.global_transform();
                        return Ok(transform.rotation.to_array());
                    }
                    None => debugger::error(&format!(
                            "lua error: get_global_rotation failed! failed to get object {} in system {}",
                            this.name, this.system_id
                    )),
                },
                None => debugger::error(&format!(
                        "lua error: get_global_rotation failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok([0.0, 0.0, 0.0])
        });

        methods.add_method("get_global_scale", |_, this, _: ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        let transform = object.global_transform();
                        return Ok(transform.scale.to_array());
                    }
                    None => debugger::error(&format!(
                            "lua error: get_global_scale failed! failed to get object {} in system {}",
                            this.name, this.system_id
                    )),
                },
                None => debugger::error(&format!(
                        "lua error: get_global_scale failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok([0.0, 0.0, 0.0])
        });

        methods.add_method(
            "set_position",
            |_, this, (x, y, z, set_body_position): (f32, f32, f32, bool)| {
                let framework = &mut *get_framework_pointer();
                match systems::get_system_mut_with_id(&this.system_id) {
                    Some(system) => match system.find_object_mut(&this.name) {
                        Some(object) => object.set_position(framework, Vec3::new(x, y, z), set_body_position),
                        None => debugger::error(
                            &format!(
                                "lua error: set_position failed! failed to get object {} in system {}",
                                this.name, this.system_id
                            )
                        ),
                    },
                    None => debugger::error(&format!(
                            "lua error: set_position failed! failed to get system {} to find object {}",
                            this.system_id, this.name
                    )),
                }

                Ok(())
            },
            );

        methods.add_method(
            "set_rotation",
            |_, this, (x, y, z, set_body_rotation): (f32, f32, f32, bool)| {
                let framework = &mut *get_framework_pointer();
                match systems::get_system_mut_with_id(&this.system_id) {
                    Some(system) => match system.find_object_mut(&this.name) {
                        Some(object) => object.set_rotation(framework, Vec3::new(x, y, z), set_body_rotation),
                        None => debugger::error(&format!(
                                "lua error: set_rotation failed! failed to get object {} in system {}",
                                this.name, this.system_id
                        )),
                    },
                    None => debugger::error(&format!(
                            "lua error: set_rotation failed! failed to get system {} to find object {}",
                            this.system_id, this.name
                    )),
                }

                Ok(())
            },
        );

        methods.add_method("set_scale", |_, this, (x, y, z): (f32, f32, f32)| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => object.set_scale(Vec3::new(x, y, z)),
                    None => debugger::error(&format!(
                            "lua error: set_scale failed! failed to get object {} in system {}",
                            this.name, this.system_id
                    )),
                },
                None => debugger::error(&format!(
                        "lua error: set_scale failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(())
        });

        // body_type = "None"/"Fixed"/""/"Ball"/"Cylinder"
        // body_collider_type = "None"/"Cuboid"/"Capsule"/"Ball"/"Cylinder"
        // render_type = "None"/"Cuboid"/"Capsule"/"Ball"/"Cylinder"
        // collider_size_x, collider_size_y, collider_size_z - some of them may be ignored
        // mass
        // membership_bits* - bitmask of object collider membership, filter_bits* - bitmask of stuff collider can interact with
        // * = optional
        methods.add_method("build_object_rigid_body", |_, this, 
            (body_type, body_collider_type, render_collider_type, collider_size_x, collider_size_y, collider_size_z, 
             mass, membership_bits, filter_bits): (String, String, String, f32, f32, f32, f32, Option<u32>, Option<u32>)| {
                //body_type: Option<BodyType>,
                //custom_render_collider: Option<RenderColliderType>,
                //mass: f32,
                //membership_groups: Option<CollisionGroups>,
                //filter_groups: Option<CollisionGroups>,

                let framework = &mut *get_framework_pointer();
                match systems::get_system_mut_with_id(&this.system_id) {
                    Some(system) => match system.find_object_mut(&this.name) {
                        Some(object) => {
                            let body_collider_type = match body_collider_type.as_str() {
                                "None" => None,
                                "Cuboid" => Some(BodyColliderType::Cuboid(collider_size_x, collider_size_y, collider_size_z)),
                                "Capsule" => Some(BodyColliderType::Capsule(collider_size_x, collider_size_y)),
                                "Cylinder" => Some(BodyColliderType::Cylinder(collider_size_x, collider_size_y)),
                                "Ball" => Some(BodyColliderType::Ball(collider_size_x)),
                                _ => {
                                    debugger::error(&format!(
                                            "lua error: build_object_rigid_body failed! the body_collider_type argument is wrong, possible values are 'None', 'Cuboid', 'Capsule', 'Cylinder', 'Ball'; object: {}; system: {}",
                                            this.name, this.system_id
                                    ));
                                    None
                                },
                            };

                            let (render_collider_type, body_type, membership, filter) =
                                lua_body_render_colliders_and_groups_to_rust(this.name.clone(), this.system_id.clone(), 
                                    body_collider_type, body_type, render_collider_type, collider_size_x, collider_size_y, 
                                    collider_size_z, membership_bits, filter_bits);

                            object.build_object_rigid_body(framework, body_type, render_collider_type, mass, membership, filter);
                        },
                        None => debugger::error(&format!(
                                "lua error: build_object_rigid_body failed! failed to get object {} in system {}",
                                this.name, this.system_id
                        )),
                    },
                    None => debugger::error(&format!(
                            "lua error: build_object_rigid_body failed! failed to get system {} to find object {}",
                            this.system_id, this.name
                    )),
                }

                Ok(())
            });


        // body_type = "None"/"Fixed"/""/"Ball"/"Cylinder"
        // model_id - id of the ModelAsset
        // render_collider_type = "None"/"Cuboid"/"Capsule"/"Ball"/"Cylinder"
        // collider_size_x, collider_size_y, collider_size_z - works only for render collider, some of them may be ignored
        // mass
        // membership_bits* - bitmask of object collider membership, filter_bits* - bitmask of stuff collider can interact with
        // * = optional
        methods.add_method("build_object_triangle_mesh_rigid_body", |_, this, 
            (body_type, model_id, render_collider_type, collider_size_x, collider_size_y, collider_size_z, 
             mass, membership_bits, filter_bits): (String, String, String, f32, f32, f32, f32, Option<u32>, Option<u32>)| {
                //body_type: Option<BodyType>,
                //custom_render_collider: Option<RenderColliderType>,
                //mass: f32,
                //membership_groups: Option<CollisionGroups>,
                //filter_groups: Option<CollisionGroups>,

                let framework = &mut *get_framework_pointer();
                match systems::get_system_mut_with_id(&this.system_id) {
                    Some(system) => match system.find_object_mut(&this.name) {
                        Some(object) => {
                            let model_asset_id = framework.get_model_asset(&model_id);
                            let model_asset;
                            match model_asset_id {
                                Some(model_asset_id) => model_asset = framework.assets.get_model_asset(&model_asset_id),
                                None => {
                                    debugger::error(&format!("Lua error: build_object_triangle_mesh_rigid_body failed!\nFailed to get preloaded model asset with id {}", model_id));
                                    return Ok(())
                                },
                            }

                            match model_asset {
                                Some(model_asset) => {
                                    let body_collider = Some(BodyColliderType::TriangleMesh(model_asset.clone()));

                                    let (render_collider_type, body_type, membership, filter) =
                                        lua_body_render_colliders_and_groups_to_rust(this.name.clone(), this.system_id.clone(), 
                                            body_collider, body_type, render_collider_type, collider_size_x, collider_size_y, 
                                            collider_size_z, membership_bits, filter_bits);

                                    object.build_object_rigid_body(framework, body_type, render_collider_type, mass, membership, filter);
                                },
                                None => {
                                    return Ok(());
                                }
                            }
                        },
                        None => debugger::error(&format!(
                                "lua error: build_object_rigid_body failed! failed to get object {} in system {}",
                                this.name, this.system_id
                        )),
                    },
                    None => debugger::error(&format!(
                            "lua error: build_object_rigid_body failed! failed to get system {} to find object {}",
                            this.system_id, this.name
                    )),
                }

                Ok(())
            });

        methods.add_method(
            "object_id",
            |_, this, _: ()| {
                match systems::get_system_mut_with_id(&this.system_id) {
                    Some(system) => match system.find_object_mut(&this.name) {
                        Some(object) => return Ok(Some(object.object_id().clone())),
                        None => {
                            debugger::error(
                                &format!("lua error: object_id failed! failed to get object {} in system {}", this.name, this.system_id));
                            return Ok(None)
                        },
                    },
                    None => debugger::error(&format!(
                            "lua error: object_id failed! failed to get system {} to find object {}",
                            this.system_id, this.name
                    )),
                }

                Ok(None)
            },
        );

        methods.add_method(
            "groups_list",
            |_, this, _: ()| {
                match systems::get_system_mut_with_id(&this.system_id) {
                    Some(system) => match system.find_object_mut(&this.name) {
                        Some(object) => {
                            let mut list = Vec::new();
                            for group in object.groups_list() {
                                list.push(group.as_raw().to_string());
                            }
                            return Ok(Some(list))
                        }
                        None => {
                            debugger::error(
                                &format!("lua error: groups_list failed! failed to get object {} in system {}", this.name, this.system_id));
                            return Ok(None)
                        },
                    },
                    None => debugger::error(&format!(
                            "lua error: groups_list failed! failed to get system {} to find object {}",
                            this.system_id, this.name
                    )),
                }

                Ok(None)
            },
            );


        methods.add_method(
            "find_object",
            |_, this, name: String| {
                match systems::get_system_mut_with_id(&this.system_id) {
                    Some(system) => match system.find_object_mut(&this.name) {
                        Some(object) => {
                            for child in object.children_list() {
                                if child.name() == name {
                                    let handle = ObjectHandle {
                                        system_id: this.system_id.clone(),
                                        name,
                                    };
                                    return Ok(Some(handle))
                                }
                            }
                        }
                        None => {
                            debugger::error(
                                &format!("lua error: find_object failed! failed to get object {} in system {}", this.name, this.system_id));
                            return Ok(None)
                        },
                    },
                    None => debugger::error(&format!(
                            "lua error: find_object failed! failed to get system {} to find object {}",
                            this.system_id, this.name
                    )),
                }

                Ok(None)
            },
            );

        methods.add_method(
            "add_to_group",
            |_, this, group: String| {
                match systems::get_system_mut_with_id(&this.system_id) {
                    Some(system) => match system.find_object_mut(&this.name) {
                        Some(object) => {
                            object.add_to_group(&group);
                        }
                        None => {
                            debugger::error(
                                &format!("lua error: add_to_group failed! failed to get object {} in system {}", this.name, this.system_id));
                        },
                    },
                    None => debugger::error(&format!(
                            "lua error: add_to_group failed! failed to get system {} to find object {}",
                            this.system_id, this.name
                    )),
                }

                Ok(())
            },
            );

        methods.add_method(
            "remove_from_group",
            |_, this, group: String| {
                match systems::get_system_mut_with_id(&this.system_id) {
                    Some(system) => match system.find_object_mut(&this.name) {
                        Some(object) => {
                            object.remove_from_group(&group);
                        }
                        None => {
                            debugger::error(
                                &format!("lua error: remove_from_group failed! failed to get object {} in system {}", this.name, this.system_id));
                        },
                    },
                    None => debugger::error(&format!(
                            "lua error: remove_from_group failed! failed to get system {} to find object {}",
                            this.system_id, this.name
                    )),
                }

                Ok(())
            },
            );


        // object-specific methods:
        methods.add_method("cast_shadows", |_, this, cast: bool| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        match object.downcast_mut::<ModelObject>() {
                            Some(object) => {
                                object.cast_shadows(cast);
                            },
                            None => {
                                match object.downcast_mut::<MasterInstancedModelObject>() {
                                    Some(object) => {
                                        object.cast_shadows(cast);
                                    },
                                    None => {
                                        debugger::error(
                                            &format!("lua error(system {}): cast_shadows failed in object: {}. this object is not ModelObject or a MasterInstancedModelObject!", 
                                                this.system_id, this.name
                                            )
                                        );
                                    },
                                }
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: cast_shadows failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: cast_shadows failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(())
        });

        methods.add_method("start_particles", |_, this, (velocity_x, velocity_y, velocity_z, random_velocity_scale): (f32, f32, f32, f32)| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        match object.downcast_mut::<ParticleSystem>() {
                            Some(object) => {
                                object.start_particles(Vec3::new(velocity_x, velocity_y, velocity_z), random_velocity_scale);
                            },
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): start_particles failed in object: {}. this object is not ParticleSystem!", 
                                        this.system_id, this.name));
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: start_particles failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(
                    &format!(
                        "lua error: start_particles failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                    )
                ),
            }

            Ok(())
        });

        methods.add_method("set_max_particle_distance", |_, this, distance: f32| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        match object.downcast_mut::<ParticleSystem>() {
                            Some(object) => {
                                object.set_max_particle_distance(distance);
                            },
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): set_max_particle_distance failed in object: {}. this object is not ParticleSystem!", 
                                        this.system_id, this.name));
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: set_max_particle_distance failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(
                    &format!(
                        "lua error: set_max_particle_distance failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                    )
                ),
            }

            Ok(())
        });



        methods.add_method("play_animation", |_, this, anim_name: String| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        match object.downcast_mut::<ModelObject>() {
                            Some(object) => {
                                object.play_animation(anim_name);
                            },
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): play_animation failed in object: {}. this object is not ModelObject!", 
                                        this.system_id, this.name));
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: play_animation failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: play_animation failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(())
        });

        methods.add_method("stop_animation", |_, this, _: ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        match object.downcast_mut::<ModelObject>() {
                            Some(object) => object.stop_animation(),
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): play_animation failed in object: {}. this object is not ModelObject!", 
                                        this.system_id, this.name));
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: play_animation failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: play_animation failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(())
        });

        methods.add_method("intersection_position", |_, this, _: ()| {
            let framework = &mut *get_framework_pointer();
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        return match object.downcast_mut::<Ray>() {
                            Some(object) => {
                                return match object.intersection_position(&framework.physics) {
                                    Some(pos) => Ok(Some([pos.x, pos.y, pos.z])),
                                    None => Ok(None),
                                }
                            },
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): intersection_position failed in object: {}. this object is not Ray!", 
                                        this.system_id, this.name));
                                Ok(None)
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: intersection_position failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: intersection_position failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(None)
        });

        methods.add_method("is_intersecting", |_, this, _: ()| {
            let framework = &mut *get_framework_pointer();
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        return match object.downcast_mut::<Ray>() {
                            Some(object) => {
                                Ok(object.is_intersecting(&framework.physics))
                            },
                            None => {
                                match object.downcast_mut::<Trigger>() {
                                    Some(object) => Ok(object.is_intersecting(&framework.physics)),
                                    None => {
                                        debugger::error(
                                            &format!("lua error(system {}): is_intersecting failed in object: {}. this object is neither Ray nor Trigger!",
                                            this.system_id, this.name));
                                        Ok(false)
                                    },
                                }
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: is_intersecting failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: is_intersecting failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(false)
        });

        methods.add_method("intersection_object_name", |_, this, _: ()| {
            let framework = &mut *get_framework_pointer();
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        return match object.downcast_mut::<Ray>() {
                            Some(object) => {
                                Ok(object.intersection_object_name(&framework.physics))
                            },
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): intersection_object_name failed in object: {}. This object is not a Ray!",
                                    this.system_id, this.name));
                                Ok(None)
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: intersection_object_name failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: intersection_object_name failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(None)
        });

        methods.add_method("intersection_object_properties", |_, this, _: ()| {
            let framework = &mut *get_framework_pointer();
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        return match object.downcast_mut::<Ray>() {
                            Some(object) => {
                                Ok(object.intersection_object_properties(&framework.physics))
                            },
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): intersection_object_properties failed in object: {}. This object is not a Ray!",
                                    this.system_id, this.name));
                                Ok(None)
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: intersection_object_properties failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: intersection_object_properties failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(None)
        });

        methods.add_method("intersection_object_groups", |_, this, _: ()| {
            let framework = &mut *get_framework_pointer();
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        return match object.downcast_mut::<Ray>() {
                            Some(object) => {
                                let result = object.intersection_object_groups(&framework.physics);
                                match result {
                                    Some(groups) => {
                                        let groups_string_vec: Vec<String> = groups.into_iter().map(|group| group.as_raw().into()).collect();
                                        Ok(Some(groups_string_vec))
                                    },
                                    None => Ok(None),
                                }
                            },
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): intersection_object_groups failed in object: {}. This object is not a Ray!",
                                    this.system_id, this.name));
                                Ok(None)
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: intersection_object_groups failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: intersection_object_groups failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(None)
        });

        methods.add_method("is_intersecting_with_group", |_, this, group: String| {
            let framework = &mut *get_framework_pointer();
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        return match object.downcast_mut::<Trigger>() {
                            Some(object) => {
                                Ok(object.is_intersecting_with_group(&framework.physics, group.into()))
                            },
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): is_intersecting_with_group failed in object: {}. this object is not Trigger!", 
                                        this.system_id, this.name));
                                Ok(false)
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: is_intersecting_with_group failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: is_intersecting_with_group failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(false)
        });

        methods.add_method("set_looping", |_, this, should_loop: bool| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        return match object.downcast_mut::<SoundEmitter>() {
                            Some(object) => Ok(object.set_looping(should_loop)),
                            None => {
                                match object.downcast_mut::<ModelObject>() {
                                    Some(object) => Ok(object.set_looping(should_loop)),
                                    None => {
                                        debugger::error(
                                            &format!("lua error(system {}): set_looping failed in object: {}. this object is neiter SoundEmitter nor ModelObject!",
                                            this.system_id, this.name));
                                        Ok(())
                                    },
                                }
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: set_looping failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: set_looping failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(())
        });

        methods.add_method("looping", |_, this, _: ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        return match object.downcast_mut::<SoundEmitter>() {
                            Some(object) => Ok(object.is_looping()),
                            None => {
                                match object.downcast_mut::<ModelObject>() {
                                    Some(object) => Ok(object.looping()),
                                    None => {
                                        debugger::error(
                                            &format!("lua error(system {}): looping failed in object: {}. this object is neiter SoundEmitter nor ModelObject!",
                                            this.system_id, this.name));
                                        Ok(false)
                                    },
                                }
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: looping failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: looping failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(false)
        });

        methods.add_method("current_animation", |_, this, _: ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        return match object.downcast_mut::<ModelObject>() {
                            Some(object) => {
                                return match object.current_animation() {
                                    Some(animation) => Ok(Some(animation.to_string())),
                                    None => Ok(None),
                                }
                            },
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): current_animation failed in object: {}. this object is not ModelObject!",
                                    this.system_id, this.name));
                                Ok(None)
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: current_animation failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: current_animation failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(None)
        });

        methods.add_method("add_uniform", |_, this, (name, value): (String, RenderUniformValue)| {
            let framework = &mut *get_framework_pointer();

            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        match object.downcast_mut::<ModelObject>() {
                            Some(object) => {
                                object.add_uniform(framework, name, value);
                            },
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): add_uniform failed in object: {}. this object is not ModelObject!",
                                    this.system_id, this.name));
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: add_uniform failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: add_uniform failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(())
        });

        methods.add_method("play_sound", |_, this, _: ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        return match object.downcast_mut::<SoundEmitter>() {
                            Some(object) => Ok(object.play_sound()),
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): play_sound failed in object: {}. this object is not SoundEmitter!", 
                                        this.system_id, this.name));
                                Ok(())
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: play_sound failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: play_sound failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(())
        });

        methods.add_method("set_max_distance", |_, this, distance: f32| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        match object.downcast_mut::<SoundEmitter>() {
                            Some(object) => {
                                let _ = object.set_max_distance(distance);
                            },
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): set_max_distance failed in object: {}. this object is not SoundEmitter!",
                                    this.system_id, this.name));
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: set_max_distance failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: set_max_distance failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(())
        });

        methods.add_method("get_max_distance", |_, this, _: ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        match object.downcast_mut::<SoundEmitter>() {
                            Some(object) => {
                                match object.get_max_distance() {
                                    Some(distance) => return Ok(Some(distance)),
                                    None => (),
                                }
                            },
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): get_max_distance failed in object: {}. this object is not SoundEmitter!",
                                    this.system_id, this.name));
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: get_max_distance failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: get_max_distance failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(None)
        });

        methods.add_method("move_controller", |_, this, (x, y, z): (f32, f32, f32)| {
            let framework = &mut *get_framework_pointer();
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        match object.downcast_mut::<CharacterController>() {
                            Some(object) => object.move_controller(framework, Vec3::new(x, y, z)),
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): move_controller failed in object: {}. this object is not CharacterController!",
                                    this.system_id, this.name));
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: move_controller failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: move_controller failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(())
        });

        methods.add_method("walk_to", |_, this, (x, y, z, speed): (f32, f32, f32, f32)| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        match object.downcast_mut::<CharacterController>() {
                            Some(object) => object.walk_to(Vec3::new(x, y, z), speed),
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): walk_to failed in object: {}. this object is not CharacterController!",
                                    this.system_id, this.name));
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: walk_to failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: walk_to failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(())
        });

        methods.add_method("stop_walking", |_, this, _: ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        match object.downcast_mut::<CharacterController>() {
                            Some(object) => object.stop_walking(),
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): stop_walking failed in object: {}. this object is not CharacterController!",
                                    this.system_id, this.name));
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: stop_walking failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: stop_walking failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(())
        });

        methods.add_method("next_path_position", |_, this, _: ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        match object.downcast_mut::<CharacterController>() {
                            Some(object) => {
                                return Ok(match object.next_path_position() {
                                    Some(point) => Some(vec![point.x, point.y, point.z]),
                                    None => None,
                                })
                            },
                            None => {
                                debugger::error(
                                    &format!("lua error(system {}): next_path_position failed in object: {}. this object is not CharacterController!",
                                    this.system_id, this.name));
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: next_path_position failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: next_path_position failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(None)
        });

        methods.add_method("set_object_properties", |_, this, properties: HashMap<String, Vec<SystemValue>>| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        object.set_object_properties(properties);
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: set_object_properties failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: set_object_properties failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(())
        });

        methods.add_method("object_properties", |_, this, _: ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => return Ok(object.object_properties().clone()),
                    None => {
                        debugger::error(
                            &format!("lua error: object_properties failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: object_properties failed! failed to get system {} to find object {}",
                        this.system_id, this.name
                )),
            }

            Ok(HashMap::new())
        });
        // i could've used a macro
    }
}

// body_type = "None"/"Fixed"/""/"Ball"/"Cylinder"
// body_collider_type = "None"/"Cuboid"/"Capsule"/"Ball"/"Cylinder"
// render_type = "None"/"Cuboid"/"Capsule"/"Ball"/"Cylinder"
// collider_size_x, collider_size_y, collider_size_z - some of them may be ignored
// mass
// membership_bits* - bitmask of object collider membership, filter_bits* - bitmask of stuff collider can interact with
// * = optional
fn lua_body_render_colliders_and_groups_to_rust(object_name: String, object_system_id: String, body_collider: Option<BodyColliderType>, body_type: String, render_collider_type: String, collider_size_x: f32, collider_size_y: f32, collider_size_z: f32, membership_bits: Option<u32>, filter_bits: Option<u32>) 
    -> (Option<RenderColliderType>, Option<BodyType>, Option<CollisionGroups>, Option<CollisionGroups>) {
        let render_collider_type = match render_collider_type.as_str() {
            "None" => None,
            "Cuboid" => Some(RenderColliderType::Cuboid(None, None, collider_size_x, collider_size_y, collider_size_z, false)),
            "Capsule" => Some(RenderColliderType::Capsule(None, None, collider_size_x, collider_size_y, false)),
            "Cylinder" => Some(RenderColliderType::Cylinder(None, None, collider_size_x, collider_size_y, false)),
            "Ball" => Some(RenderColliderType::Ball(None, None, collider_size_x, false)),
            _ => {
                debugger::error(&format!(
                        "lua error: build_object_rigid_body failed! the render_collider_type argument is wrong, possible values are 'None', 'Cuboid', 'Capsule', 'Cylinder', 'Ball'; object: {}; system: {}",
                        object_name, object_system_id
                ));
                None
            },
        };
        let body_type = match body_type.as_str() {
            "None" => None,
            "Fixed" => Some(BodyType::Fixed(body_collider)),
            "Dynamic" => Some(BodyType::Dynamic(body_collider)),
            "VelocityKinematic" => Some(BodyType::VelocityKinematic(body_collider)),
            "PositionKinematic" => Some(BodyType::PositionKinematic(body_collider)),
            _ => {
                debugger::error(&format!(
                        "lua error: build_object_rigid_body failed! the body_type argument is wrong, possible values are 'None', 'Fixed', 'Dynamic', 'VelocityKinematic', 'PositionKinematic'; object: {}; system: {}",
                        object_name, object_system_id
                ));
                None
            },
        };

        let membership = match membership_bits {
            Some(bits) => Some(CollisionGroups::from(bits)),
            None => None,
        };

        let filter = match filter_bits {
            Some(bits) => Some(CollisionGroups::from(bits)),
            None => None,
        };

        (render_collider_type, body_type, membership, filter)
}

impl UserData for Message {
    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("contents_type", |_, this, _: ()| {
            match this.contents {
                MessageContents::SyncObject(_) => Ok("SyncObject"),
                MessageContents::Custom(_) => Ok("Custom"),
            }
        });

        methods.add_method("sync_object_name", |_, this, _: ()| {
            match &this.contents {
                MessageContents::SyncObject(message) => {
                    Ok(Some(message.object_name.to_string()))
                },
                MessageContents::Custom(_) => {
                    debugger::error(&"lua error: sync_object_name in Message failed! the contents_type != 'SyncObject'");
                    Ok(None)
                },
            }
        });

        // returns [[x, y, z], [x, y, z], [x, y, z]]
        // [position_xyz, rotation_xyz, scale_xyz]
        methods.add_method("sync_object_pos_rot_scale", |_, this, _: ()| {
            match &this.contents {
                MessageContents::SyncObject(message) => {
                    let transform = message.transform;
                    let position = transform.position;
                    let position = [position.x, position.y, position.z];
                    let rotation = transform.rotation;
                    let rotation = [rotation.x, rotation.y, rotation.z];
                    let scale = transform.scale;
                    let scale = [scale.x, scale.y, scale.z];
                    let pos_rot_scale = [position, rotation, scale];
                    Ok(Some(pos_rot_scale))
                },
                MessageContents::Custom(_) => {
                    debugger::error(&"lua error: sync_object_pos_rot_scale in Message failed! the contents_type != 'SyncObject'");
                    Ok(None)
                },
            }
        });

        methods.add_method("custom_contents", |_, this, _: ()| {
            match &this.contents {
                MessageContents::SyncObject(_) => {
                    debugger::error(&"lua error: custom_contents in Message failed! the contents_type != 'Custom'");
                    Ok(None)
                },
                MessageContents::Custom(contents) => {
                    Ok(Some(contents.to_owned()))
                },
            }
        });

        methods.add_method("message_id", |_, this, _: ()| {
            Ok(this.message_id.to_owned())
        });

        methods.add_method("message_sender", |_, this, _: ()| {
            match &this.message_type {
                crate::managers::networking::MessageType::FromServer(_) => {
                    debugger::error(&"lua error: message_sender in Message failed! the message was sent from a server");
                    Ok(None)
                },
                crate::managers::networking::MessageType::FromClient(sender) => {
                    Ok(Some(sender.to_string()))
                },
            }
        });

        methods.add_method("message_type", |_, this, _: ()| {
            match this.message_type {
                crate::managers::networking::MessageType::FromServer(_) => Ok("FromServer"),
                crate::managers::networking::MessageType::FromClient(_) => Ok("FromClient"),
            }
        });
    }
}

impl<'lua> FromLua<'lua> for SystemValue {
    fn from_lua(value: mlua::prelude::LuaValue<'lua>, _: &'lua Lua) -> mlua::prelude::LuaResult<Self> {
        if let Some(value) = value.as_f32() {
            return Ok(SystemValue::Float(value));
        } 
        if let Some(value) = value.as_string() {
            return Ok(SystemValue::String(String::from(value.to_str().unwrap())));
        } 
        if let Some(value) = value.as_u32() {
            return Ok(SystemValue::UInt(value));
        }
        if let Some(value) = value.as_i32() {
            return Ok(SystemValue::Int(value));
        }
        if let Some(value) = value.as_boolean() {
            return Ok(SystemValue::Bool(value));
        }
        if let Some(value) = value.as_userdata() {
            match value.take::<LuaSpline>() {
                Ok(value) => return Ok(SystemValue::Spline(value.0)),
                Err(err) => {
                    return Err(Error::FromLuaConversionError { 
                        from: "UserData",
                        to: "SystemValue::Spline",
                        message: Some(err.to_string()) 
                    })
                },
            }
        }
        if let Some(value) = value.as_table() {
            let mut vec: Vec<SystemValue> = Vec::new();
            if let Err(err) = value.for_each(|_: SystemValue, value: SystemValue| {
                vec.push(value);
                Ok(())
            }) {
                debugger::error(&format!("Failed to convert a Table to a Vec. Err: {}", err))
            };

            return Ok(SystemValue::Vec(vec))
        }
        Err(Error::FromLuaConversionError { from: value.type_name(), to: "SystemValue::Vec", message: None })
    }
}

impl<'lua> FromLua<'lua> for RenderUniformValue {
    fn from_lua(value: mlua::Value<'lua>, lua: &'lua Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::Integer(int) => Ok(RenderUniformValue::Float(int as f32)),
            mlua::Value::Number(float) => Ok(RenderUniformValue::Float(float as f32)),
            mlua::Value::Table(table) => {
                if table.raw_len() == 3 {
                    let mut vector = [0.0, 0.0, 0.0];
                    let x: Result<f32, mlua::Error> = table.get(1);
                    let y: Result<f32, mlua::Error> = table.get(2);
                    let z: Result<f32, mlua::Error> = table.get(3);
                    match x {
                        Ok(x) => vector[0] = x,
                        Err(_) => {
                            debugger::error(&("Failed to convert lua table to a RenderUniformValue Vec3!"
                                    .to_string() + "\nTable's element is NOT a float"));
                        }
                    }
                    match y {
                        Ok(y) => vector[1] = y,
                        Err(_) => {
                            debugger::error(&("Failed to convert lua table to a RenderUniformValue Vec3!"
                                    .to_string() + "\nTable's element is NOT a float"));
                        }
                    }
                    match z {
                        Ok(z) => vector[2] = z,
                        Err(_) => {
                            debugger::error(&("Failed to convert lua table to a RenderUniformValue Vec3!"
                                    .to_string() + "\nTable's element is NOT a float"));
                        }
                    }

                    Ok(RenderUniformValue::Vec3(vector.into()))
                } else {
                    debugger::error(&("Failed to convert lua value to a RenderUniformValue!".to_string()
                            + "\nTable len should be 3 to convert it to a Vec3"));
                    Err(mlua::Error::FromLuaConversionError {
                        from: "Table",
                        to: "RenderUniformValue",
                        message: Some(String::from("Avaliable Lua types: int, number and string (texture id), table (Vec3)"))
                    })
                }
            },
            mlua::Value::String(id) => {
                let framework = &mut *get_framework_pointer();
                if let Some(id) = framework.assets.get_texture_asset_id(id.to_str()?) {
                    Ok(RenderUniformValue::Texture(id))
                } else {
                    debugger::error(&("Failed to convert lua value to a RenderUniformValue!".to_string()
                            + &format!("\nFailed to get a texture with id {}", id.to_str()?)));

                    Err(mlua::Error::FromLuaConversionError {
                        from: "String",
                        to: "RenderUniformValue",
                        message: Some(String::from("Can't find a texture with required ID."))
                    })
                }
            },
            _ => {
                debugger::error(&("Failed to convert lua value to a RenderUniformValue!".to_string()
                        + "\nAvaliable Lua types: int, number and string (texture id), table (Vec3)"));
                Err(mlua::Error::FromLuaConversionError {
                    from: "-",
                    to: "RenderUniformValue",
                    message: Some(String::from("Avaliable Lua types: int, number and string (texture id)"))
                })
            }
        }
        
    }
}
impl<'lua> IntoLua<'lua> for SystemValue {
    fn into_lua(self, lua: &'lua mlua::prelude::Lua) -> mlua::prelude::LuaResult<mlua::prelude::LuaValue<'lua>> {
        match self {
            SystemValue::String(value) => Ok(value.into_lua(lua)?),
            SystemValue::Int(value) => Ok(value.into_lua(lua)?),
            SystemValue::UInt(value) => Ok(value.into_lua(lua)?),
            SystemValue::Float(value) => Ok(value.into_lua(lua)?),
            SystemValue::Bool(value) => Ok(value.into_lua(lua)?),
            SystemValue::Vec(value) => Ok(value.into_lua(lua)?),
            SystemValue::Spline(value) => Ok(LuaSpline(value).into_lua(lua)?),
        }
    }
}

impl<'lua> IntoLua<'lua> for DebugMode {
    fn into_lua(self, lua: &'lua Lua) -> mlua::prelude::LuaResult<mlua::prelude::LuaValue<'lua>> {
        match self {
            DebugMode::None =>
                Ok("None".into_lua(lua)?),
            DebugMode::ShowFps =>
                Ok("ShowFps".into_lua(lua)?),
            DebugMode::Full =>
                Ok("Full".into_lua(lua)?),
        }
    }
}

impl UserData for Framework {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(_: &mut F) {}

    fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_ptr", |_, _, () : _| {
            Ok(unsafe { crate::framework::FRAMEWORK_POINTER.to_string() })
        });
        methods.add_method("debug_mode", |lua, framework, (): _| framework.debug_mode().into_lua(lua));
        methods.add_method_mut("set_debug_mode", 
            |_, framework, debug_mode: String| {
                match debug_mode.as_str() {
                    "None" => framework.set_debug_mode(DebugMode::None),
                    "ShowFps" => framework.set_debug_mode(DebugMode::ShowFps),
                    "Full" => framework.set_debug_mode(DebugMode::Full),
                    _ => debugger::error("Lua framework set_debug_mode error!\nAcceptable values are 'None', 'ShowFps', 'Full'")
                }
                Ok(())
            }
        );

        methods.add_method_mut("delta_time",
            |_, framework, _: ()| {
                Ok(framework.delta_time().as_secs_f32())
            }
        );

        methods.add_method_mut("remove_global_system_value", 
            |_, framework, key: String| {
                framework.remove_global_system_value(&key);
                Ok(())
            }
        );

        methods.add_method_mut("set_global_system_value", 
            |_, framework, (key, value): (String, Vec<SystemValue>)| {
                framework.set_global_system_value(&key, value);
                Ok(())
            }
        );

        methods.add_method("get_global_system_value", 
            |_, framework, key: String| {
                Ok(framework.get_global_system_value(&key))
            }
        );

        methods.add_method_mut("load_save",
            |_, framework, save_name: String| {
                let _ = framework.load_save(&save_name);
                Ok(())
            }
        );

        methods.add_method_mut("register_save_value",
            |_, framework, system_value_name: String| {
                framework.register_save_value(&system_value_name);
                Ok(())
            }
        );

        methods.add_method_mut("unregister_save_value",
            |_, framework, system_value_name: String| {
                framework.unregister_save_value(&system_value_name);
                Ok(())
            }
        );

        methods.add_method_mut("new_save",
            |_, framework, save_name: String| {
                let _ = framework.new_save(&save_name);
                Ok(())
            }
        );

        methods.add_method_mut("save_game",
            |_, framework, _: ()| {
                framework.save_game();
                Ok(())
            }
        );

        methods.add_method_mut("load_lazy_value",
            |_, framework, value_name: String| {
                Ok(framework.load_lazy_value(&value_name))
            }
        );

        methods.add_method_mut("save_lazy_value",
            |_, framework, (value_name, value): (String, Vec<SystemValue>)| {
                Ok(framework.save_lazy_value(&value_name, value))
            }
        );

        methods.add_method_mut("new_bind_keyboard",
            |_, framework, (name, keys): (String, Vec<String>)| {
                let keys: Vec<&str> = keys.iter().map(|key| key.as_str()).collect();
                framework.new_bind_keyboard(&name, keys);
                Ok(())
            }
        );

        methods.add_method_mut("new_bind_mouse",
            |_, framework, (name, buttons): (String, Vec<String>)| {
                let buttons: Vec<&str> = buttons.iter().map(|key| key.as_str()).collect();
                framework.new_bind_mouse(&name, buttons);
                Ok(())
            }
        );

        methods.add_method_mut("is_bind_pressed",
            |_, framework, name: String| {
                Ok(framework.is_bind_pressed(&name))
            }
        );

        methods.add_method_mut("is_bind_down",
            |_, framework, name: String| {
                Ok(framework.is_bind_down(&name))
            }
        );

        methods.add_method_mut("is_bind_released",
            |_, framework, name: String| {
                Ok(framework.is_bind_released(&name))
            }
        );

        methods.add_method_mut("mouse_position",
            |_, framework, _: ()| {
                let position = framework.mouse_position();
                Ok(vec![position.x, position.y])
            }
        );

        methods.add_method_mut("mouse_position_from_center",
            |_, framework, _: ()| {
                let position = framework.mouse_position_from_center();
                Ok(vec![position.x, position.y])
            }
        );

        methods.add_method_mut("mouse_delta",
            |_, framework, _: ()| {
                let delta = framework.mouse_delta();
                Ok(vec![delta.x, delta.y])
            }
        );

        methods.add_method_mut("is_mouse_locked",
            |_, framework, _: ()| {
                Ok(framework.is_mouse_locked())
            }
        );

        methods.add_method_mut("set_mouse_locked",
            |_, framework, lock: bool| {
                Ok(framework.set_mouse_locked(lock))
            }
        );

        methods.add_method_mut("set_mouse_locked",
            |_, framework, lock: bool| {
                Ok(framework.set_mouse_locked(lock))
            }
        );

        methods.add_method_mut("preload_model_asset",
            |_, framework, (asset_id, asset_path): (String, Option<String>)| {
                let asset_path = match asset_path {
                    Some(asset_path) => asset_path,
                    None => asset_id.clone(),
                };

                match framework.preload_model_asset(asset_id.clone(), &asset_path) {
                    Ok(_) => println!("Preloaded ModelAsset with id '{}'!", asset_id),
                    Err(_) => (),
                }
                Ok(())
            }
        );

        methods.add_method_mut("background_preload_model_asset",
            |_, framework, (asset_id, asset_path): (String, Option<String>)| {
                let asset_path = match asset_path {
                    Some(asset_path) => asset_path,
                    None => asset_id.clone(),
                };

                match framework.background_preload_model_asset(asset_id.clone(), &asset_path) {
                    Ok(_) => println!("(Background) Preloaded ModelAsset with id '{}'!", asset_id),
                    Err(_) => println!("(Background) Failed to preload ModelAsset with id '{}'!", asset_id),
                }
                Ok(())
            }
        );

        methods.add_method_mut("preload_sound_asset",
            |_, framework, (asset_id, asset_path): (String, Option<String>)| {
                let asset_path = match asset_path {
                    Some(asset_path) => asset_path,
                    None => asset_id.clone(),
                };

                match framework.preload_sound_asset(asset_id.clone(), &asset_path) {
                    Ok(_) => println!("Preloaded SoundAsset with id '{}'!", asset_id),
                    Err(_) => (),
                }
                Ok(())
            }
        );

        methods.add_method_mut("preload_texture_asset",
            |_, framework, (asset_id, asset_path): (String, Option<String>)| {
                let asset_path = match asset_path {
                    Some(asset_path) => asset_path,
                    None => asset_id.clone(),
                };

                match framework.preload_texture_asset(asset_id.clone(), &asset_path) {
                    Ok(_) => println!("Preloaded TextureAsset with id '{}'!", asset_id),
                    Err(_) => (),
                }
                Ok(())
            }
        );

        // ui
        methods.add_method_mut("is_widget_double_clicked",
            |_, framework, (window_id, widget_id): (String, String)| {
                Ok(framework.is_widget_double_clicked(&window_id, &widget_id))
            }
        );

        methods.add_method_mut("get_widget_numeric_value",
            |_, framework, (window_id, widget_id): (String, String)| {
                Ok(framework.get_widget_numeric_value(&window_id, &widget_id))
            }
        );

        methods.add_method_mut("is_widget_right_clicked",
            |_, framework, (window_id, widget_id): (String, String)| {
                Ok(framework.is_widget_right_clicked(&window_id, &widget_id))
            }
        );

        methods.add_method_mut("add_multiline_text_edit",
            |_, framework, (window_id, widget_id, contents, size, parent): (String, String, String, [f32; 2], Option<String>)| {
                Ok(framework.add_multiline_text_edit(&window_id, &widget_id, &contents, size.into(), parent.as_deref()))
            }
        );

        methods.add_method_mut("add_progress_bar",
            |_, framework, (window_id, widget_id, contents, text, text_size, size, parent): (String, String, f32, String, Option<f32>, [f32; 2], Option<String>)| {
                Ok(framework.add_progress_bar(&window_id, &widget_id, contents, text, text_size, size.into(), parent.as_deref()))
            }
        );

        methods.add_method_mut("add_float_slider",
            |_, framework, (window_id, widget_id, value, min, max, size, parent): (String, String, f32, f32, f32, [f32; 2], Option<String>)| {
                Ok(framework.add_float_slider(&window_id, &widget_id, value, min, max, size.into(), parent.as_deref()))
            }
        );

        methods.add_method_mut("add_int_slider",
            |_, framework, (window_id, widget_id, value, min, max, size, parent): (String, String, i32, i32, i32, [f32; 2], Option<String>)| {
                Ok(framework.add_int_slider(&window_id, &widget_id, value, min, max, size.into(), parent.as_deref()))
            }
        );

        methods.add_method_mut("add_checkbox",
            |_, framework, (window_id, widget_id, value, title, size, parent): (String, String, bool, String, [f32; 2], Option<String>)| {
                Ok(framework.add_checkbox(&window_id, &widget_id, value, &title, size.into(), parent.as_deref()))
            }
        );

        methods.add_method_mut("add_horizontal_scroll",
            |_, framework, (window_id, widget_id, size, parent): (String, String, [f32; 2], Option<String>)| {
                Ok(framework.add_horizontal_scroll(&window_id, &widget_id, size.into(), parent.as_deref()))
            }
        );

        methods.add_method_mut("add_vertical_scroll",
            |_, framework, (window_id, widget_id, size, parent): (String, String, [f32; 2], Option<String>)| {
                Ok(framework.add_vertical_scroll(&window_id, &widget_id, size.into(), parent.as_deref()))
            }
        );

        methods.add_method_mut("add_scroll",
            |_, framework, (window_id, widget_id, size, parent): (String, String, [f32; 2], Option<String>)| {
                Ok(framework.add_scroll(&window_id, &widget_id, size.into(), parent.as_deref()))
            }
        );

        methods.add_method_mut("add_horizontal",
            |_, framework, (window_id, widget_id, size, parent): (String, String, [f32; 2], Option<String>)| {
                Ok(framework.add_horizontal(&window_id, &widget_id, size.into(), parent.as_deref()))
            }
        );

        methods.add_method_mut("add_vertical",
            |_, framework, (window_id, widget_id, size, parent): (String, String, [f32; 2], Option<String>)| {
                Ok(framework.add_vertical(&window_id, &widget_id, size.into(), parent.as_deref()))
            }
        );

        methods.add_method_mut("add_button",
            |_, framework, (window_id, widget_id, contents, size, parent): (String, String, String, [f32; 2], Option<String>)| {
                Ok(framework.add_button(&window_id, &widget_id, &contents, size.into(), parent.as_deref()))
            }
        );

        methods.add_method_mut("add_text_button",
            |_, framework, (window_id, widget_id, contents, font_size, bold, size, parent): (String, String, String, f32, bool, [f32; 2], Option<String>)| {
                Ok(framework.add_text_button(&window_id, &widget_id, &contents, font_size, bold, size.into(), parent.as_deref()))
            }
        );

        methods.add_method_mut("add_image",
            |_, framework, (window_id, widget_id, image_path, size, parent): (String, String, String, [f32; 2], Option<String>)| {
                Ok(framework.add_image(&window_id, &widget_id, &image_path, size.into(), parent.as_deref()))
            }
        );

        methods.add_method_mut("add_label",
            |_, framework, (window_id, widget_id, contents, text_size, size, parent): (String, String, String, Option<f32>, [f32; 2], Option<String>)| {
                Ok(framework.add_label(&window_id, &widget_id, &contents, text_size, size.into(), parent.as_deref()))
            }
        );

        methods.add_method_mut("add_bold_label",
            |_, framework, (window_id, widget_id, contents, text_size, size, parent): (String, String, String, Option<f32>, [f32; 2], Option<String>)| {
                Ok(framework.add_bold_label(&window_id, &widget_id, &contents, text_size, size.into(), parent.as_deref()))
            }
        );

        methods.add_method_mut("set_widget_spacing",
            |_, framework, (window_id, widget_id, spacing): (String, String, f32)| {
                Ok(framework.set_widget_spacing(&window_id, &widget_id, spacing))
            }
        );

        methods.add_method_mut("add_window_theme",
            |_, framework, (theme_id, theme_json): (String, String)| {
                Ok(framework.add_window_theme(theme_id, theme_json))
            }
        );

        methods.add_method_mut("add_theme",
            |_, framework, (theme_id, theme_json): (String, String)| {
                Ok(framework.add_theme(theme_id, theme_json))
            }
        );

        methods.add_method_mut("set_window_theme",
            |_, framework, (window_id, theme_id): (String, Option<String>)| {
                let theme_id_temp;
                let theme_id = match theme_id {
                    Some(theme_id) => {
                        theme_id_temp = theme_id;
                        Some(theme_id_temp.as_str())
                    },
                    None => None,
                };

                Ok(framework.set_window_theme(&window_id, theme_id))
            }
        );

        methods.add_method_mut("set_widget_theme",
            |_, framework, (window_id, widget_id, theme_id): (String, String, Option<String>)| {
                let theme_id_temp;
                let theme_id = match theme_id {
                    Some(theme_id) => {
                        theme_id_temp = theme_id;
                        Some(theme_id_temp.as_str())
                    },
                    None => None,
                };

                Ok(framework.set_widget_theme(&window_id, &widget_id, theme_id))
            }
        );

        methods.add_method_mut("new_window",
            |_, framework, (window_id, transparent): (String, bool)| {
                Ok(framework.new_window(&window_id, transparent))
            }
        );

        methods.add_method_mut("set_window_on_top",
            |_, framework, (window_id, show_on_top): (String, bool)| {
                Ok(framework.set_window_on_top(&window_id, show_on_top))
            }
        );

        methods.add_method_mut("remove_window",
            |_, framework, window_id: String| {
                Ok(framework.remove_window(&window_id))
            }
        );

        methods.add_method_mut("remove_widget",
            |_, framework, (window_id, widget_id): (String, String)| {
                Ok(framework.remove_widget(&window_id, &widget_id))
            }
        );

        methods.add_method_mut("is_widget_hovered",
            |_, framework, (window_id, widget_id): (String, String)| {
                Ok(framework.is_widget_hovered(&window_id, &widget_id))
            }
        );

        methods.add_method_mut("is_widget_left_clicked",
            |_, framework, (window_id, widget_id): (String, String)| {
                Ok(framework.is_widget_left_clicked(&window_id, &widget_id))
            }
        );

        methods.add_method_mut("is_widget_dragged",
            |_, framework, (window_id, widget_id): (String, String)| {
                Ok(framework.is_widget_dragged(&window_id, &widget_id))
            }
        );

        methods.add_method_mut("is_widget_changed",
            |_, framework, (window_id, widget_id): (String, String)| {
                Ok(framework.is_widget_changed(&window_id, &widget_id))
            }
        );

        methods.add_method_mut("get_resolution",
            |_, framework, _: ()| {
                let resolution = framework.get_resolution();
                Ok([resolution.x, resolution.y])
            }
        );

        methods.add_method_mut("set_window_position",
            |_, framework, (window_id, position): (String, Option<[f32; 2]>)| {
                let position = match position {
                    Some(position) => Some(Vec2::new(position[0], position[1])),
                    None => None,
                };

                Ok(framework.set_window_position(&window_id, position))
            }
        );

        methods.add_method_mut("set_window_size",
            |_, framework, (window_id, size): (String, Option<[f32; 2]>)| {
                let size = match size {
                    Some(size) => Some(Vec2::new(size[0], size[1])),
                    None => None,
                };

                Ok(framework.set_window_size(&window_id, size))
            }
        );

        methods.add_method_mut("show_title_bar",
            |_, framework, (window_id, show): (String, bool)| {
                Ok(framework.show_title_bar(&window_id, show))
            }
        );

        methods.add_method_mut("show_close_button",
            |_, framework, (window_id, show): (String, bool)| {
                Ok(framework.show_close_button(&window_id, show))
            }
        );
        // render
        methods.add_method_mut("set_camera_position",
            |_, framework, (x, y, z): (f32, f32, f32)| {
                Ok(framework.set_camera_position(Vec3::new(x, y, z)))
            }
        );

        methods.add_method_mut("set_camera_rotation",
            |_, framework, (x, y, z): (f32, f32, f32)| {
                Ok(framework.set_camera_rotation(Vec3::new(x, y, z)))
            }
        );

        methods.add_method_mut("set_camera_fov",
            |_, framework, fov: f32| {
                Ok(framework.set_camera_fov(fov))
            }
        );

        methods.add_method_mut("set_light_direction",
            |_, framework, (x, y, z): (f32, f32, f32)| {
                Ok(framework.set_light_direction(Vec3::new(x, y, z)))
            }
        );

        methods.add_method_mut("get_light_direction",
            |_, framework, (): ()| {
                let direction = framework.get_light_direction();
                match direction {
                    Some(direction) => Ok(Some([direction.x, direction.y, direction.z])),
                    None => Ok(None),
                }
            }
        );

        methods.add_method_mut("get_camera_position",
            |_, framework, (): ()| {
                let result = framework.get_camera_position();
                match result {
                    Some(vector) => Ok(Some([vector.x, vector.y, vector.z])),
                    None => Ok(None),
                }
            }
        );

        methods.add_method_mut("get_camera_rotation",
            |_, framework, (): ()| {
                let result = framework.get_camera_rotation();
                match result {
                    Some(vector) => Ok(Some([vector.x, vector.y, vector.z])),
                    None => Ok(None),
                }
            }
        );

        methods.add_method_mut("get_camera_front",
            |_, framework, (): ()| {
                let result = framework.get_camera_front();
                match result {
                    Some(vector) => Ok(Some([vector.x, vector.y, vector.z])),
                    None => Ok(None),
                }
            }
        );

        methods.add_method_mut("get_camera_left",
            |_, framework, (): ()| {
                let result = framework.get_camera_left();
                match result {
                    Some(vector) => Ok(Some([vector.x, vector.y, vector.z])),
                    None => Ok(None),
                }
            }
        );

        methods.add_method_mut("rotate_vector",
            |_, _, (dir_x, dir_y, dir_z, rot_x, rot_y, rot_z): (f32, f32, f32, f32, f32, f32)| {
                let vec = math_utils::rotate_vector(Vec3::new(dir_x, dir_y, dir_z), Vec3::new(rot_x, rot_y, rot_z));
                Ok([vec.x, vec.y, vec.z])
            }
        );
    }
}

impl UserData for Box<dyn Object> { }

impl<'lua> FromLuaMulti<'lua> for Framework {
    fn from_lua_multi(values: mlua::prelude::LuaMultiValue<'lua>, _: &'lua Lua) -> mlua::prelude::LuaResult<Self> {
        let value = values.get(0);
        match value {
            Some(value) => {
                let userdata = value.as_userdata();
                match userdata {
                    Some(userdata) => {
                        let framework: Result<Framework, Error> = userdata.take();
                        match framework {
                            Ok(framework) => Ok(framework),
                            Err(err) => {
                                debugger::error(
                                    &format!("Lua error! Failed to get Framework from lua. Can't convert userdata to Framework.\nErr: {}", err));
                                mlua::prelude::LuaResult::Err(Error::FromLuaConversionError { 
                                    from: "-",
                                    to: "Framework",
                                    message: Some("Lua error! Failed to get Framework from lua. Can't convert userdata to Framework.".into())
                                })
                            },
                        }
                    },
                    None => {
                        debugger::error("Lua error! Failed to get Framework from lua. Can't convert value to userdata.");
                        mlua::prelude::LuaResult::Err(Error::FromLuaConversionError { 
                            from: "-",
                            to: "Framework",
                            message: Some("Lua error! Failed to get Framework from lua. Can't convert value to userdata.".into())
                        })
                    },
                }
            },
            None => {
                debugger::error("Lua error! Failed to get Framework from lua. values[0] = None");
                mlua::prelude::LuaResult::Err(Error::FromLuaConversionError { 
                    from: "-",
                    to: "Framework",
                    message: Some("Lua error! Failed to get Framework from lua. values[0] = None".into())
                })
            },
        }
    }
}

pub(crate) struct LuaSpline(pub Spline<f32, f32>);

impl UserData for LuaSpline {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {}

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_value", |_, spline, t: f32| -> Result<Option<f32>, Error> {
            Ok(match spline.0.sample(t) {
                Some(v) => Some(v),
                None => None,
            })
        });
    }
}

impl UserData for PerlinNoise {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {}

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get_x", |_, noise, coordinate: f32| -> Result<f32, Error> {
            Ok(noise.get_x(coordinate))
        });

        methods.add_method("get_y", |_, noise, coordinate: f32| -> Result<f32, Error> {
            Ok(noise.get_y(coordinate))
        });
    }
}

fn get_framework_pointer() -> &'static mut Framework {
    let framework_ptr: *mut Framework = unsafe { framework::FRAMEWORK_POINTER } as *mut Framework;
    let framework = unsafe { &mut *framework_ptr };
    framework
}
