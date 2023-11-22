pub mod lua_functions;

use std::{fs, collections::HashMap};
use mlua::{Lua, Function};
use once_cell::sync::Lazy;
use crate::{objects::{Object, empty_object::EmptyObject}, managers::{debugger, assets, systems::CallList, scripting::lua::lua_functions::add_lua_vm_to_list}, systems::System};

static mut SYSTEMS_LUA_VMS: Lazy<HashMap<String, Lua>> = Lazy::new(|| return HashMap::new() ); // String is system's id and Lua is it's vm

#[derive(Debug)]
pub struct LuaSystem {
    pub is_destroyed: bool,
    pub id: String,
    pub objects: Vec<Box<dyn Object>>
}

impl LuaSystem {
    fn call_in_object(&mut self, args: Vec<String>) -> Option<&str> { 
        let system_id = self.system_id().to_owned();

        match args.len() >= 2 {
            true => {
                let object_option = self.find_object_mut(&args[0]);
                match object_option {
                    Some(object) => {
                        let mut object_call_args: Vec<&str> = Vec::new();
                        args.iter().enumerate().for_each(|(idx, arg)| if idx > 1 {
                            object_call_args.push(arg);
                        });

                        object.call(&args[1], object_call_args)
                    }
                    None => {
                        debugger::error(&format!("failed to call call_in_object() in system {}, can't find required object {}", system_id, args[0]));
                        return None;
                    },
                }
            }
            false => {
                debugger::error(&format!("failed to call call_in_object() in system {}, can't get first two arguments in vector", self.system_id()));
                None
            },
        }
    }

    pub fn new(id: &str, script_path: &str) -> Result<LuaSystem, LuaSystemError> {
        match fs::read_to_string(assets::get_full_asset_path(&script_path)) {
            Ok(script) => {
                let lua: Lua = Lua::new();
                let load_result = lua.load(script).exec();

                match load_result {
                    Ok(_) => {
                        let system = LuaSystem {
                            is_destroyed: false,
                            id: id.into(),
                            objects: Vec::new()
                        };

                        add_lua_vm_to_list(id.into(), lua);

                        Ok(system)
                    },
                    Err(err) => {
                        debugger::error(&format!("lua system creation error!\nlua execution error\nerror: {}", err));
                        return Err(LuaSystemError::ScriptLoadingError);
                    }
                }
            },
            Err(err) => {
                debugger::error(&format!("lua system creation error!\nerror: {}", err));
                return Err(LuaSystemError::ScriptLoadingError);
            }
        }
    }

    fn new_empty_object(&mut self, name: &str) {
        let object = EmptyObject::new(name);

        self.objects.push(Box::new(object));
    }
}

impl System for LuaSystem {
    fn update(&mut self) {
        let lua_option = get_lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "update");
            },
            None => debugger::error("lua system update function error\ncan't get lua vm reference")
        }
    }

    fn start(&mut self) {
        let lua_option = get_lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, "start");
            },
            None => debugger::error("lua system update function error\ncan't get lua vm reference")
        }
    }


    fn call(&self, call_id: &str) {
        let lua_option = get_lua_vm_ref(self.system_id().into());
        match lua_option {
            Some(lua) => {
                let _ = call_lua_function(self.system_id(), &lua, call_id);
            },
            None => debugger::error(&format!("lua system call function error(call_id: {})\ncan't get lua vm reference", call_id))
        }
    }

    fn call_mut(&mut self, call_id: &str) {
        debugger::warn(&format!("lua system {} warning when calling {}\nyou can just use call, instead of call_mut", self.id, call_id));

        self.call(call_id);
    }

    fn call_with_args(&mut self, call_id: &str, args: Vec<String>) -> Option<&str> {
        let system_id = self.id.clone();

        match call_id {
            "new_empty_object" => {
                match args.get(0) {
                    Some(name) => {
                        self.new_empty_object(&name);
                        None
                    },
                    _ => {
                        debugger::error(&format!("failed to call new_empty_object() in system {}, can't get first argument in vector", system_id));
                        None
                    },
                }
            },

            "call_in_object" => {
                self.call_in_object(args)
            },

            _ => {
                debugger::error(&format!("can't call {} in system {}, check all things that are avaliable to call using get_call_list().", call_id, system_id));
                None
            }
        }
    }

    fn get_objects_list(&self) -> &Vec<Box<dyn Object>> {
        &self.objects
    }

    fn get_objects_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.objects
    }

    fn get_call_list(&self) -> crate::managers::systems::CallList {
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
}

fn get_lua_vm_ref<'a>(system_id: String) -> Option<&'a Lua> {
    unsafe {
        SYSTEMS_LUA_VMS.get(&system_id)
    }
}

fn get_lua_vm_ref_mut<'a>(system_id: String) -> Option<&'a mut Lua> {
    unsafe {
        SYSTEMS_LUA_VMS.get_mut(&system_id)
    }
}

fn call_lua_function(system_id: &str, lua: &Lua, function_name: &str) -> Result<(), mlua::Error> {
    let update_function_result: Result<Function, mlua::Error> = lua.globals().get(function_name);

    match update_function_result {
        Ok(func) => {
            let call_result: Result<(), mlua::Error> = Function::call(&func, ());
            if let Err(err) = call_result {
                debugger::error(&format!("lua error when calling 'update' in system {}\nerror: {}", system_id, err));
                return Err(err);
            }

            return Ok(());
        }
        Err(err) => {
            debugger::error(&format!("can't get an update function in lua system {}\nerror: {}", system_id, err));
            return Err(err)
        }
    }
}

#[derive(Debug)]
pub enum LuaSystemError {
    ScriptLoadingError,
    LuaExecutingError
}

/*#[derive(Debug, Clone)]
pub struct LuaObjectHandle {
    pub object_name: String,
    pub owner_system_name: String
}*/
