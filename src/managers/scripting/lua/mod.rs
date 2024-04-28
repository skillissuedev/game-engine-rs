pub mod lua_functions;
use crate::{
    assets::model_asset::{self, ModelAsset}, managers::{
        assets, debugger, networking::{Message, MessageContents}, physics::{BodyColliderType, BodyType, CollisionGroups, ObjectBodyParameters, RenderColliderType}, scripting::lua::lua_functions::add_lua_vm_to_list, systems::{self, CallList}
    }, objects::{character_controller::CharacterController, model_object::ModelObject, ray::Ray, sound_emitter::SoundEmitter, trigger::Trigger}, systems::System
};
use crate::objects::Object;
use egui_glium::egui_winit::egui::TextBuffer;
use glam::Vec3;
use mlua::{Function, Lua, LuaOptions, StdLib, UserData};
use once_cell::sync::Lazy;
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
                dbg!(&load_result);

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
    fn client_update(&mut self) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "client_update");
            }
            None => debugger::error("lua system client_update function error\ncan't get lua vm reference"),
        }
    }

    fn client_start(&mut self) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "client_start");
            }
            None => debugger::error("lua system client_start function error\ncan't get lua vm reference"),
        }
    }

    fn call(&self, call_id: &str) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, call_id);
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
                let function_result: Result<Function, mlua::Error> = lua.globals().get("reg_message");

                match function_result {
                    Ok(func) => {
                        let call_result: Result<(), mlua::Error> = Function::call(&func, message);
                        if let Err(err) = call_result {
                            debugger::error(&format!(
                                "lua error when calling reg_message in system {}\nerror: {}", self.system_id(), err
                            ));
                        }
                    }
                    Err(err) => debugger::error(&format!("can't get function reg_message in lua system {}\nerror: {}", self.system_id(), err)),
                }
            }
            None => debugger::error("lua system reg_message function error\ncan't get lua vm reference"),
        }
    }

    fn server_start(&mut self) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "server_start");
            }
            None => debugger::error("lua system server_start function error\ncan't get lua vm reference"),
        }
    }

    fn server_update(&mut self) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "server_update");
            }
            None => debugger::error("lua system server_update function error\ncan't get lua vm reference"),
        }
    }

    fn server_render(&mut self) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "server_render");
            }
            None => debugger::error("lua system server_update function error\ncan't get lua vm reference"),
        }
    }

    fn client_render(&mut self) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "client_render");
            }
            None => debugger::error("lua system client_render function error\ncan't get lua vm reference"),
        }
    }
}

fn lua_vm_ref<'a>(system_id: String) -> Option<&'a Lua> {
    unsafe { SYSTEMS_LUA_VMS.get(&system_id) }
}

fn lua_vm_ref_mut<'a>(system_id: String) -> Option<&'a mut Lua> {
    unsafe { SYSTEMS_LUA_VMS.get_mut(&system_id) }
}

fn call_lua_function(system_id: &str, lua: &Lua, function_name: &str) -> Result<(), mlua::Error> {
    let function_result: Result<Function, mlua::Error> = lua.globals().get(function_name);

    match function_result {
        Ok(func) => {
            let call_result: Result<(), mlua::Error> = Function::call(&func, ());
            if let Err(err) = call_result {
                debugger::error(&format!(
                    "lua error when calling '{}' in system {}\nerror: {}",
                    function_name, system_id, err
                ));
                return Err(err);
            }

            Ok(())
        }
        Err(err) => {
            debugger::error(&format!(
                "can't get function '{}' in lua system {}\nerror: {}",
                function_name, system_id, err
            ));
            Err(err)
        }
    }
}

#[derive(Debug)]
pub enum LuaSystemError {
    ScriptLoadingError,
    LuaExecutingError,
    LuaCreationError,
}

/*#[derive(Debug, Clone)]
pub struct LuaObjectHandle {
    pub object_name: String,
    pub owner_system_name: String
}*/

pub struct ObjectHandle {
    pub system_id: String,
    pub name: String,
}

impl UserData for ObjectHandle {
    fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(fields: &mut F) {}

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
                match systems::get_system_mut_with_id(&this.system_id) {
                    Some(system) => match system.find_object_mut(&this.name) {
                        Some(object) => object.set_position(Vec3::new(x, y, z), set_body_position),
                        None => debugger::error(&format!(
                            "lua error: set_position failed! failed to get object {} in system {}",
                            this.name, this.system_id
                        )),
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
                match systems::get_system_mut_with_id(&this.system_id) {
                    Some(system) => match system.find_object_mut(&this.name) {
                        Some(object) => object.set_rotation(Vec3::new(x, y, z), set_body_rotation),
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

                        object.build_object_rigid_body(body_type, render_collider_type, mass, membership, filter);
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
        // model_path - path to the GLTF model
        // render_collider_type = "None"/"Cuboid"/"Capsule"/"Ball"/"Cylinder"
        // collider_size_x, collider_size_y, collider_size_z - works only for render collider, some of them may be ignored
        // mass
        // membership_bits* - bitmask of object collider membership, filter_bits* - bitmask of stuff collider can interact with
        // * = optional
        methods.add_method("build_object_triangle_mesh_rigid_body", |_, this, 
            (body_type, model_path, render_collider_type, collider_size_x, collider_size_y, collider_size_z, 
            mass, membership_bits, filter_bits): (String, String, String, f32, f32, f32, f32, Option<u32>, Option<u32>)| {
            //body_type: Option<BodyType>,
            //custom_render_collider: Option<RenderColliderType>,
            //mass: f32,
            //membership_groups: Option<CollisionGroups>,
            //filter_groups: Option<CollisionGroups>,

            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        let model_asset = ModelAsset::from_gltf(&model_path);
                        match model_asset {
                            Ok(model_asset) => {
                                let body_collider = Some(BodyColliderType::TriangleMesh(model_asset));

                                let (render_collider_type, body_type, membership, filter) =
                                    lua_body_render_colliders_and_groups_to_rust(this.name.clone(), this.system_id.clone(), 
                                        body_collider, body_type, render_collider_type, collider_size_x, collider_size_y, 
                                        collider_size_z, membership_bits, filter_bits);

                                object.build_object_rigid_body(body_type, render_collider_type, mass, membership, filter);
                            },
                            Err(err) => {
                                debugger::error(&format!(
                                    "lua error: build_object_trimesh_rigid_body failed! failed to load a ModelAsset\nerr: {:?}\nobject: {}; system: {}",
                                    err, this.name, this.system_id
                                ));
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
        // i fucking hate my life

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


        // ah shit, here we go again
        // object-specific methods:
        methods.add_method("play_animation", |_, this, anim_name: String| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        match object.downcast_mut::<ModelObject>() {
                            Some(object) => {
                                if let Err(err) = object.play_animation(&anim_name) {
                                    debugger::error(
                                        &format!("lua error(system {}): play_animation failed! error in ModelObject '{}': {:?}", 
                                           this.system_id, this.name, err));
                                }
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

        methods.add_method("intersection_position", |_, this, _: ()| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        return match object.downcast_mut::<Ray>() {
                            Some(object) => {
                                return match object.intersection_position() {
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
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        return match object.downcast_mut::<Ray>() {
                            Some(object) => {
                                Ok(object.is_intersecting())
                            },
                            None => {
                                match object.downcast_mut::<Trigger>() {
                                    Some(object) => Ok(object.is_intersecting()),
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

        methods.add_method("is_intersecting_with_group", |_, this, group: String| {
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        return match object.downcast_mut::<Trigger>() {
                            Some(object) => {
                                Ok(object.is_intersecting_with_group(group.into()))
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
                                    Some(object) => Ok(object.is_looping()),
                                    None => {
                                        debugger::error(
                                            &format!("lua error(system {}): is_looping failed in object: {}. this object is neiter SoundEmitter nor ModelObject!",
                                            this.system_id, this.name));
                                        Ok(false)
                                    },
                                }
                            },
                        }
                    }
                    None => {
                        debugger::error(
                            &format!("lua error: is_looping failed! failed to get object {} in system {}", this.name, this.system_id));
                    },
                },
                None => debugger::error(&format!(
                        "lua error: is_looping failed! failed to get system {} to find object {}",
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
                                    Ok(distance) => return Ok(Some(distance)),
                                    Err(_) => (),
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
            match systems::get_system_mut_with_id(&this.system_id) {
                Some(system) => match system.find_object_mut(&this.name) {
                    Some(object) => {
                        match object.downcast_mut::<CharacterController>() {
                            Some(object) => object.move_controller(Vec3::new(x, y, z)),
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
            match this.message {
                MessageContents::SyncObject(_) => Ok("SyncObject"),
                MessageContents::Custom(_) => Ok("Custom"),
            }
        });

        methods.add_method("sync_object_name", |_, this, _: ()| {
            match &this.message {
                MessageContents::SyncObject(message) => {
                    Ok(Some(message.object_name.to_string()))
                },
                MessageContents::Custom(_) => {
                    debugger::error(&"lua error: get_sync_object_name in Message failed! the contents_type != 'SyncObject'");
                    Ok(None)
                },
            }
        });

        // returns [[x, y, z], [x, y, z], [x, y, z]]
        // [position_xyz, rotation_xyz, scale_xyz]
        methods.add_method("sync_object_pos_rot_scale", |_, this, _: ()| {
            match &this.message {
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
                    debugger::error(&"lua error: get_sync_object_name in Message failed! the contents_type != 'SyncObject'");
                    Ok(None)
                },
            }
        });

        methods.add_method("custom_contents", |_, this, _: ()| {
            match &this.message {
                MessageContents::SyncObject(_) => {
                    debugger::error(&"lua error: get_custom_contents in Message failed! the contents_type != 'Custom'");
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
    }
}
