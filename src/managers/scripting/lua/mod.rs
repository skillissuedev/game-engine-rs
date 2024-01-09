pub mod lua_functions;

use std::{fs, collections::HashMap};
use ez_al::SoundSourceType;
use mlua::{Lua, Function, StdLib, LuaOptions};
use once_cell::sync::Lazy;
use crate::{objects::{Object, empty_object::EmptyObject, model_object::ModelObject, sound_emitter::SoundEmitter, camera_position::CameraPosition}, managers::{debugger, assets, systems::CallList, scripting::lua::lua_functions::add_lua_vm_to_list}, systems::System, assets::{model_asset::ModelAsset, texture_asset::TextureAsset, shader_asset::{ShaderAsset, ShaderAssetPath}, sound_asset::SoundAsset}};

static mut SYSTEMS_LUA_VMS: Lazy<HashMap<String, Lua>> = Lazy::new(|| return HashMap::new() ); // String is system's id and Lua is it's vm

#[derive(Debug)]
pub struct LuaSystem {
    pub is_destroyed: bool,
    pub id: String,
    pub objects: Vec<Box<dyn Object>>
}

impl LuaSystem {
    fn call_in_object(&mut self, args: Vec<String>) -> Option<String> { 
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
                let lua: Lua = match Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()) {
                    Ok(lua) => lua,
                    Err(err) => {
                        debugger::error(&format!("lua system creation error!\nlua creation error\nerror: {}", err));
                        return Err(LuaSystemError::ScriptLoadingError);
                    },
                };

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

        self.add_object(Box::new(object));
    }

    fn new_model_object(
        &mut self, name: &str, model_asset_path: &str, texture_asset_path: &str, vertex_shader_asset_path: &str, fragment_shader_asset_path: &str) {
        let asset = ModelAsset::from_file(model_asset_path);
        let texture = TextureAsset::from_file(texture_asset_path);
        let shader = ShaderAsset::load_from_file(ShaderAssetPath {
            vertex_shader_path: vertex_shader_asset_path.to_string(),
            fragment_shader_path: fragment_shader_asset_path.to_string()
        });

        let asset = match asset {
            Ok(asset) => asset,
            Err(err) => {
                debugger::error(&format!("new model object call error(system {})\nmodel asset loading error: {:?}", self.id, err));
                return;
            },
        };

        let texture = match texture {
            Ok(texture) => texture,
            Err(err) => {
                debugger::error(&format!("new model object call error(system {})\ntexture asset loading error: {:?}", self.id, err));
                return;
            },
        };

        let shader = match shader {
            Ok(shader) => shader,
            Err(err) => {
                debugger::error(&format!("new model object call error(system {})\nshader asset loading error: {:?}", self.id, err));
                return;
            },
        };

        let object = ModelObject::new(name, asset, Some(texture), shader);

        self.add_object(Box::new(object));
    }

    fn new_sound_emitter_object(&mut self, name: &str, asset_path: &str, is_positional: SoundSourceType) {
        let asset_result = SoundAsset::from_wav(asset_path);
        match asset_result {
            Ok(asset) => {
                let object_result = SoundEmitter::new(name, &asset, is_positional);
                match object_result {
                    Ok(object) => self.add_object(Box::new(object)),
                    Err(err) => debugger::error(&format!("new sound emitter object call error(system {})\nobject creation error: {:?}", self.id, err)),
                }
            },
            Err(err) => debugger::error(&format!("new sound emitter object call error(system {})\nsound asset loading error: {:?}", self.id, err)),
        };
    }

    fn new_camera_position_object(&mut self, name: &str) {
        let object = CameraPosition::new(name);
        self.add_object(Box::new(object));
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

    fn call_with_args(&mut self, call_id: &str, args: Vec<String>) -> Option<String> {
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

            "new_camera_position_object" => {
                match args.get(0) {
                    Some(name) => {
                        self.new_camera_position_object(&name);
                        None
                    },
                    _ => {
                        debugger::error(&format!("failed to call new_camera_position_object() in system {}, can't get first argument in vector", system_id));
                        None
                    },
                }
            },

            "new_sound_emitter_object" => {
                match args.len() {
                    3 => {
                        let name = &args[0];
                        let asset = &args[1];
                        let sound_emitter_type = match args[2].as_ref() {
                            "true" => SoundSourceType::Positional, 
                            "false" => SoundSourceType::Simple,
                            _ => {
                                debugger::error(&format!("failed to call new_sound_emitter_object() in system {}, arg 'is_positional' is wrong, should be 'true' or 'false'", system_id));
                                return None;
                            }
                        };

                        self.new_sound_emitter_object(name, asset, sound_emitter_type);
                        None
                    },
                    _ => {
                        debugger::error(&format!("failed to call new_sound_emitter_object() in system {}, args cound is not 3", system_id));
                        None
                    },
                }
            }

            "new_model_object" => {
                match args.len() {
                    5 => {
                        let name = &args[0];
                        let asset = &args[1];
                        let texture = &args[2];
                        let vert_shader = &args[3];
                        let frag_shader = &args[4];

                        self.new_model_object(name, &asset, texture, &vert_shader, &frag_shader);
                        None
                    },
                    _ => {
                        debugger::error(&format!("failed to call new_model_object() in system {}, args cound is not 5", system_id));
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

    fn reg_message(&mut self, message: crate::managers::networking::Message) {
        todo!()
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
    LuaExecutingError,
    LuaCreationError
}

/*#[derive(Debug, Clone)]
pub struct LuaObjectHandle {
    pub object_name: String,
    pub owner_system_name: String
}*/
