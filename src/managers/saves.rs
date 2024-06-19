use std::{collections::HashMap, fs::{self, File}, io::{self, Read, Write}, path::Path};

use crate::{framework::{self, Framework}, managers::debugger};

use super::{assets::get_full_asset_path, systems::SystemValue};

#[derive(Default)]
pub struct SavesManager {
    save_system_values: Vec<String>,
    current_save_file: Option<String>
}

impl SavesManager {
    pub fn load_save(&mut self, framework: &mut Framework, save_name: &str) -> Result<(), ()> {
        let save_file_path = "saves/".to_string() + save_name;
        let save_file_path = get_full_asset_path(&save_file_path);

        let mut json = String::new();
        match File::open(&save_file_path) {
            Ok(mut file) => {
                if let Err(err) = file.read_to_string(&mut json) {
                    debugger::error(
                        &format!("save manager's load_game error!\nfailed to read the save file\nerr: {}, path: {}", err, save_file_path)
                    );
                    return Err(())
                }
            },
            Err(err) => {
                debugger::error(
                    &format!("save manager's load_game error!\nfailed to open the save file\nerr: {}, path: {}", err, save_file_path)
                );
                return Err(())
            }
        }

        let values: Result<HashMap<&str, Vec<SystemValue>>, serde_json::Error> = serde_json::from_str(&json);
        match values {
            Ok(values) => {
                for (key, value) in values {
                    self.save_system_values.push(key.into());
                    framework.set_global_system_value(key, value);
                };
                self.current_save_file = Some(save_name.into());
                Ok(())
            },
            Err(err) => {
                self.current_save_file = Some(save_name.into());
                debugger::error(
                    &format!("save manager's load_game error!\nfailed to deserialize the save file contents!\nerr: {}, path: {}", err, save_file_path)
                );

                Err(())
            }
        }
    }

    pub fn register_save_value(&mut self, system_value_name: &str) {
        if !self.save_system_values.contains(&(system_value_name.into())) {
            self.save_system_values.push(system_value_name.into());
        }
    }

    pub fn unregister_save_value(&mut self, system_value_name: &str) {
        self.save_system_values.retain(|value| value != system_value_name);
    }

    pub fn new_save(&mut self, framework: &mut Framework, save_name: &str) -> Result<(), io::Error> {
        let save_dir_path = "saves/";
        let save_dir_path = get_full_asset_path(save_dir_path);

        if !Path::new(&save_dir_path).exists() {
            if let Err(err) = fs::create_dir(&save_dir_path) {
                debugger::error(&format!("saves manager's new_save error!\nfailed to create 'saves' directory!\nerr: {}", err));
                return Err(err)
            }
        }

        let save_path = save_dir_path.to_string() + save_name;
        if let Err(err) = File::create_new(save_path) {
            debugger::error(&format!("saves manager's new_save error!\nfailed to create '{}' file!\nerr: {}", save_name, err));
            return Err(err)
        }

        println!("saves manager: save file '{}' created and used as current one!", save_name);
        self.current_save_file = Some(save_name.into());
        self.save_game(framework);

        Ok(())
    }

    pub fn save_game(&mut self, framework: &mut Framework) {
        let mut values_list: HashMap<&str, Vec<SystemValue>> = HashMap::new();

        for value_name in &self.save_system_values {
            let value = framework.get_global_system_value(&value_name);
            match value {
                Some(value) => {
                    values_list.insert(value_name, value);
                },
                None =>
                    debugger::warn(&format!("saves manager's save_game warning!\nfailed to get global system value '{}'", value_name)),
            }
        }

        let json = serde_json::to_string_pretty(&values_list);
        match json {
            Ok(json) => {
                if let Some(current_save_file) = &self.current_save_file {
                    let save_file_path = "saves/".to_string() + current_save_file;
                    let save_file_path = get_full_asset_path(&save_file_path);

                    match File::create(&save_file_path) {
                        Ok(mut file) => {
                            if let Err(err) = file.write_all(json.as_bytes()) {
                                debugger::error(
                                    &format!("save manager's save_game error!\nfailed to write the file\nerr: {}, path: {}", err, save_file_path)
                                );
                            }
                        },
                        Err(err) => 
                            debugger::error(
                                &format!("save manager's save_game error!\nfailed to open the save file\nerr: {}, path: {}", err, save_file_path)
                            ),
                    }
                } else {
                    debugger::error("save manager's save_game error!\ncurrent save file is none! load/create one first");
                }
            },
            Err(err) => 
                debugger::error(&format!("save manager's save_game error!\n failed to serialize data!\nerror: {}", err)),
        }
    }
}
