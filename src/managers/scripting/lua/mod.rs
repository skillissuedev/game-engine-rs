pub mod lua_functions;
use crate::{
    managers::{
        assets, debugger,
        networking::Message,
        scripting::lua::lua_functions::add_lua_vm_to_list,
        systems::{self, CallList},
    },
    systems::System,
};
use crate::objects::Object;
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
            None => debugger::error("lua system update function error\ncan't get lua vm reference"),
        }
    }

    fn client_start(&mut self) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "client_start");
            }
            None => debugger::error("lua system update function error\ncan't get lua vm reference"),
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
        todo!()
    }

    fn server_start(&mut self) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "server_start");
            }
            None => debugger::error("lua system update function error\ncan't get lua vm reference"),
        }
    }

    fn server_update(&mut self) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "server_update");
            }
            None => debugger::error("lua system update function error\ncan't get lua vm reference"),
        }
    }

    fn server_render(&mut self) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "server_render");
            }
            None => debugger::error("lua system update function error\ncan't get lua vm reference"),
        }
    }

    fn client_render(&mut self) {
        let lua_option = lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "client_render");
            }
            None => debugger::error("lua system update function error\ncan't get lua vm reference"),
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
    }
}
