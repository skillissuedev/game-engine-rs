// TODO

/*

use std::sync::Arc;
use dyon::{Module, Call, Runtime, dyon_fn, dyon_macro_items, dyon_fn_pop, Dfn, dyon_obj};
use crate::{systems::System, objects::{Object, empty_object::EmptyObject}, managers::{debugger, systems::CallList, assets::get_full_asset_path}};

pub struct DyonSystem {
    pub objects: Vec<Box<dyn Object>>,
    pub is_destroyed: bool,
    pub id: String,
    pub script_path: String,
    pub module: Arc<Module>
}

impl System for DyonSystem {
    fn call(&self, call_id: &str) {
        let call = Call::new(call_id);
        let call_result = call.run(&mut Runtime::new(), &self.module);

        match call_result {
            Ok(_) => (),
            Err(err) => debugger::error(
                &format!("got an error when trying to call function '{}' in a dyon system '{}'!\ncall erorr: {:?}", call_id, self.id, err)),
        }
    }

    fn start(&mut self) {
        let call = Call::new("start");
        let call_result = call.run(&mut Runtime::new(), &self.module);

        match call_result {
            Ok(_) => (),
            Err(err) => debugger::error(&format!("got an error when trying to call function 'start' in a dyon system '{}'!\ncall erorr: {:?}", self.id, err)),
        }
    }

    fn update(&mut self) {
        let call = Call::new("update");
        let call_result = call.run(&mut Runtime::new(), &self.module);

        match call_result {
            Ok(_) => (),
            Err(err) => debugger::error(&format!("got an error when trying to call function 'update' in a dyon system '{}'!\ncall erorr: {:?}", self.id, err)),
        }
    }

    fn render(&mut self) {
        let call = Call::new("render");
        let call_result = call.run(&mut Runtime::new(), &self.module);

        match call_result {
            Ok(_) => (),
            Err(err) => debugger::error(&format!("got an error when trying to call function 'update' in a dyon system '{}'!\ncall erorr: {:?}", self.id, err)),
        }
    }

    fn call_mut(&mut self, _call_id: &str) {
        todo!()
    }

    fn system_id(&self) -> &str {
        &self.id
    }

    fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }

    fn set_destroyed(&mut self, is_destroyed: bool) {
        self.is_destroyed = is_destroyed
    }

    fn get_call_list(&self) -> CallList {
        let call = Call::new("call_list");
        let call_result: Result<CallList, String> = call.run_ret(&mut Runtime::new(), &self.module);

        match call_result {
            Ok(call_list) => call_list,
            Err(err) => {
                debugger::warn(
                    &format!("couldn't get a call_list from dyon system '{}':\ngot an error when trying to call function 'call_list' in a dyon system\ncall erorr: {:?}\nreturning an empty call_list", self.id, err));

                return CallList {
                    immut_call: vec![],
                    mut_call: vec![]
                }
            }
        }
    }

    fn get_objects_list(&self) -> &Vec<Box<dyn crate::objects::Object>> {
        &self.objects
    }

    fn get_objects_list_mut(&mut self) -> &mut Vec<Box<dyn crate::objects::Object>> {
        &mut self.objects
    }
}

impl DyonSystem {
    pub fn new(system_id: &str, path: &str) -> Result<DyonSystem, DyonSystemErr> {
        let source_result = std::fs::read_to_string(get_full_asset_path(path));
        match source_result {
            Ok(source) => {
                let mut module = Module::empty();
                module.add_str("log", log, Dfn::nl(vec![dyon::Type::Str; 1], dyon::Type::Void));

                /*let call_list = dyon::Type::AdHoc(Arc::new("CallList".into()), Box::new(dyon::Type::Any));
                module.add_str(
                    "empty_fn",
                    empty_funtion, // check this function defenition to understand why it's so strange
                    Dfn::nl(vec![], call_list.clone())
                );*/

                let script = dyon::load_str(system_id, Arc::new(source.into()), &mut module);
                match script {
                    Ok(()) => {
                        let system = DyonSystem {
                            objects: Vec::new(),
                            is_destroyed: false,
                            id: system_id.into(),
                            script_path: path.into(),
                            module: module.into()
                        };

                        return Ok(system);
                    },
                    Err(error) => {
                        debugger::error(&format!("got an error when trying to create a dyon system '{}'!\ndyon loading erorr: {}", system_id, error));
                        return Err(DyonSystemErr::DyonErr);
                    },
                }
            },
            Err(error) => {
                debugger::error(&format!("got an error when trying to create a dyon system '{}'!\nfile loading error: {:?}", system_id, error));
                return Err(DyonSystemErr::FileLoadingError);
            }
        }
    }
}

dyon_fn! {
    fn log(text: String) {
        println!("{}", text);
    }
}

dyon_fn! {
    fn new_empty_object(name: String) {
        let object = Box::new(EmptyObject::new(name.as_str()));


    }
}

dyon_obj! {
    CallList {
        immut_call,
        mut_call
    }
}

#[derive(Debug)]
pub enum DyonSystemErr {
    FileLoadingError,
    DyonErr,
    DyonCallErr(String)
}*/
