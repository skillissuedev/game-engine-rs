/*// TODO


use std::{fs, path::Path, rc::Rc};

use ruwren::{VMConfig, VMWrapper, FunctionHandle, create_module, Class, get_slot_checked, ClassObject, send_foreign, ModuleLibrary};

use crate::{objects::{Object, model_object::{ModelObject, ModelObjectError}, empty_object::EmptyObject}, managers::{assets::get_full_asset_path, debugger}, systems::System, assets::{model_asset::{ModelAsset, ModelAssetError}, texture_asset::TextureAsset, shader_asset::{ShaderAsset, ShaderAssetPath}}};

pub struct WrenSystem {
    pub objects: Vec<Box<dyn Object>>,
    pub is_destroyed: bool,
    pub wren_vm: VMWrapper,
    pub system_config: WrenSystemConfig,
    pub id: String,
}

impl WrenSystem {
    pub fn new(config: WrenSystemConfig) -> Result<WrenSystem, WrenSystemError> {
        let full_script_path_str = &get_full_asset_path(&config.script_path);
        let full_script_path = Path::new(full_script_path_str);
        let script_result = fs::read_to_string(full_script_path);
        match script_result {
            Ok(script) => {
                let id = full_script_path.file_stem().unwrap();
                let mut module_lib = ModuleLibrary::new();
                engine::publish_module(&mut module_lib);

                let vm = VMConfig::new().library(&module_lib).build();
                let engine_script = r#"
                foreign class EngineSystem {
                    foreign static new_empty_object(name)
                }

                foreign class Obj {
                }
                "#;

                vm.interpret("engine", engine_script).unwrap();

                let interpret_result = vm.interpret("main", script);

                match interpret_result {
                    Ok(()) => (),
                    Err(err) => debugger::error(&format!("wren system creation error({})\nerror when trying to interpret wren script\nerror: {}", config.script_path, err))
                }

                return
                    Ok(WrenSystem {
                        objects: Vec::new(),
                        is_destroyed: false,
                        wren_vm: vm,
                        system_config: config,
                        id: id.to_str().unwrap().into(),
                    });
            },
            Err(err) => {
                debugger::error(&format!("wren system creation error({})!\nfile loading error: {}", config.script_path, err.to_string()));
                return Err(WrenSystemError::LoadError(err.to_string()));
            }
        }
    }

    fn call_method_in_class(&self, class: &str, method: &str) {
        self.wren_vm.execute(|vm| {
            vm.ensure_slots(1);
            vm.get_variable("main", class, 0);
        });

        let result = self.wren_vm.call(ruwren::FunctionSignature::Function { name: method.into(), arity: 0 });
        match result {
            Ok(()) => (),
            Err(err) => debugger::warn(&format!("wren system({}) warning\ngot an error when trying to call a wren method {} in class {}\nerror: {}", self.id, method, class, err)),
        }
    }

    fn call_method_in_class_with_handle(&self, class: &str) {
        self.wren_vm.execute(|vm| {
            vm.ensure_slots(1);
            vm.get_variable("main", class, 0);
        });
    }

    fn get_method_handle(&self, class: &str, method: &str) -> Rc<FunctionHandle> {
        self.wren_vm.execute(|vm| {
            vm.ensure_slots(1);
            vm.get_variable("main", class, 0);
        });

        self.wren_vm.make_call_handle(ruwren::FunctionSignature::Function { name: method.into(), arity: 0 })
    }
}

impl System for WrenSystem {
    fn start(&mut self) {
        self.call_method_in_class("SystemBasics", "start");
    }

    fn update(&mut self) {
        self.call_method_in_class("SystemBasics", "update")
    }

    fn render(&mut self) {
        self.call_method_in_class("SystemBasics", "render")
    }

    fn call(&self, call_id: &str) {
        self.wren_vm.execute(|vm| {
            vm.ensure_slots(1);
            vm.get_variable("main", "SystemCalls", 0);
        });

        let result = self.wren_vm.call(ruwren::FunctionSignature::Function { name: call_id.into(), arity: 0 });
        match result {
            Ok(()) => (),
            Err(err) => debugger::warn(&format!("wren system({}) warning\ngot an error when trying to call a wren method {}\nerror: {}", self.id, call_id, err)),
        }
    }

    fn call_mut(&mut self, call_id: &str) {
        debugger::warn("call_mut() does same as the call(), so you can just use call() function");

        let result = self.wren_vm.call(ruwren::FunctionSignature::Function { name: call_id.into(), arity: 0 });
        match result {
            Ok(()) => (),
            Err(err) => debugger::warn(&format!("wren system({}) warning\ngot an error when trying to call a wren function {}\nerror: {}", self.id, call_id, err)),
        }
    }

    fn get_objects_list(&self) -> &Vec<Box<dyn Object>> {
        todo!()
    }

    fn get_objects_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        todo!()
    }

    fn get_call_list(&self) -> crate::managers::systems::CallList {
        todo!()
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


struct EngineSystem {
    pub id: String,
    pub objects: Vec<Box<dyn Object>>
}

impl EngineSystem {
    /*fn new_model_object(&mut self, name: String, asset_path: String, shader_asset_path: Option<ShaderAssetPath>, texture_asset_path: Option<String>)
        -> Result<(), WrenSystemError> {

        let asset_result = ModelAsset::from_file(&asset_path);
        match asset_result {
            Ok(asset) => {
                let shader_result: Result<ShaderAsset, crate::assets::shader_asset::ShaderError>;
                match shader_asset_path {
                    None => shader_result = ShaderAsset::load_default_shader(),
                    Some(shader_path) => shader_result = ShaderAsset::load_from_file(shader_path),
                }

                match shader_result {
                    Ok(shader) => {
                        match texture_asset_path {
                            Some(texture_path) => {
                                let texture_asset_result = TextureAsset::from_file(&texture_path);

                                match texture_asset_result {
                                    Ok(texture_asset) => {
                                        let object = ModelObject::new(&name, asset, Some(texture_asset), shader);
                                        self.objects.push(Box::new(object));
                                        self.objects.last_mut().unwrap().start();
                                    },
                                    Err(texture_error) => {
                                        debugger::error(&format!("got an error in wren system {} when tried to call new_model_object()\nTextureAsset error: {}", self.id, texture_error));
                                        return Err(WrenSystemError::NewModelObjectError("ModelAsset error".into()));
                                    }
                                }

                            },
                            None => {
                                let object = ModelObject::new(&name, asset, None, shader);
                                self.objects.push(Box::new(object));
                                self.objects.last_mut().unwrap().start();
                            }
                        }
                    },
                    Err(shader_err) => {
                        debugger::error(&format!("got an error in wren system {} when tried to call new_model_object()\nShaderAsset error: {}", self.id, shader_err));
                        return Err(WrenSystemError::NewModelObjectError("ShaderAsset error".into()));
                    }
                }



            },
            Err(asset_err) => {
                debugger::error(&format!("got an error in wren system {} when tried to call new_model_object()\nModelAsset error: {}", self.id, asset_err));
                return Err(WrenSystemError::NewModelObjectError("ModelAsset error".into()));
            },
        }

        return Ok(());
    }*/

    fn new_empty_object(vm: &ruwren::VM) {
        let name = get_slot_checked!(vm => string 1);

        vm.ensure_slots(1);
        send_foreign!(vm, "engine", "Obj", Box::new(EmptyObject::new(&name)) as Box<dyn Object> => 0);
    }

    /*fn get_objects_list(&self, vm: &ruwren::VM) {
        let list_len = self.objects.len();
        vm.ensure_slots(list_len + 2);
        vm.set_slot_new_list(0);

        for i in 0..list_len {
            send_foreign!(vm, "engine", "Obj", self.objects[i] => i + 1);
        }
    }*/

    fn find_object(&self, vm: &ruwren::VM) {
    }
}

impl Class for EngineSystem {
    fn initialize(vm: &ruwren::VM) -> Self where Self: Sized {
        let id = get_slot_checked!(vm => string 1);

        return EngineSystem {
            id,
            objects: Vec::new()
        };
    }
}

impl Class for Box<dyn Object> {
    fn initialize(_: &ruwren::VM) -> Self where Self: Sized {
        debugger::warn("Wren Scripting warning \nDO NOT initialize a Box<dyn Object>! Use create_empty_object, create_model_object etc.\nInitialization created an empty object with name 'object'");

        Box::new(EmptyObject::new("object"))
    }
}

create_module! {
    class("EngineSystem") crate::managers::scripting::wren::EngineSystem => system {
        static(fn "new_empty_object", 1) new_empty_object
    }

    class("Obj") Box<dyn crate::objects::Object> => object { }

    module => engine
}


pub struct WrenSystemConfig {
    pub script_path: String
}

#[derive(Debug)]
pub enum WrenSystemError {
    LoadError(String),
    NewModelObjectError(String)
}
*/
