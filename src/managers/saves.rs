use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read, Write},
    path::Path,
};
use serde::Serialize;
use crate::managers::debugger; 
use super::{assets::get_full_asset_path, systems::SystemValue}; 

#[derive(Default)] 
pub struct SavesManager { 
    save_system_values: Vec<String>, 
    current_save_name: Option<String>, 
    lazy_values_to_save: HashMap<String, Vec<SystemValue>> 
} 

impl SavesManager { 
    pub fn load_save(&mut self, save_name: &str) -> Result<HashMap<String, Vec<SystemValue>>, ()> {
        let mut global_values: HashMap<String, Vec<SystemValue>> = HashMap::new();
        let save_value_names_file_path = "saves/".to_string() + save_name + "/save_values.json";
        let save_value_names_file_path = get_full_asset_path(&save_value_names_file_path);

        match fs::read_to_string(&save_value_names_file_path) {
            Ok(save_values_names_json) => {
                match serde_json::from_str::<Vec<String>>(&save_values_names_json) {
                    Ok(save_value_names) => {
                        for value_name in save_value_names {
                            match load_value_from_file(save_name, &value_name) {
                                Some(value) => {
                                    global_values.insert(value_name, value);
                                },
                                None => (),
                            }
                        }
                        self.current_save_name = Some(save_name.into());
                        return Ok(global_values)
                    },
                    Err(err) => {
                        debugger::error(
                            &format!(
                                "save manager's error!\nfailed to deserialize save values list!\nfile path: {}\nerr: {}", 
                                save_value_names_file_path, err
                            )
                        );
                    },
                }
            },
            Err(err) => {
                debugger::error(
                    &format!(
                        "save manager's error!\nfailed to read {}\nerr: {}", 
                        save_value_names_file_path, err
                    )
                );
            },
        }

        self.current_save_name = Some(save_name.into());
        Err(())
    }

    pub fn register_save_value(&mut self, system_value_name: &str) {
        if !self
            .save_system_values
            .contains(&(system_value_name.into()))
        {
            self.save_system_values.push(system_value_name.into());
        }
    }

    pub fn unregister_save_value(&mut self, system_value_name: &str) {
        self.save_system_values
            .retain(|value| value != system_value_name);
    }

    pub fn new_save(
        &mut self,
        save_name: &str,
        global_values: &HashMap<String, Vec<SystemValue>>,
    ) -> Result<(), io::Error> {
        let save_dir_path = "saves/";
        let save_dir_path = get_full_asset_path(save_dir_path);
        dbg!(&save_dir_path);

        if !Path::new(&save_dir_path).exists() {
            if let Err(err) = fs::create_dir(&save_dir_path) {
                debugger::error(&format!(
                    "saves manager's new_save error!\nfailed to create 'saves' directory!\nerr: {}",
                    err
                ));
                return Err(err);
            }
        }

        let save_path = save_dir_path.to_string() + save_name + "/";
        dbg!(&save_path);

        if !Path::new(&save_path).exists() {
            let create_dir = fs::create_dir(&save_path);
            dbg!(&create_dir);
            if let Err(err) = create_dir {
                debugger::error(&format!(
                    "saves manager's new_save error!\nfailed to create 'saves/{}/' directory!\nerr: {}",
                     save_name, err
                ));
                return Err(err);
            }
            if let Err(err) = File::create_new(save_path + "save_values.json") {
                debugger::error(
                    &format!(
                        "saves manager's new_save error!\nfailed to create 'saves/{}/save_values.json' file!\nerr: {}",
                        save_name, err
                    )
                );
                return Err(err);
            }
        }

        println!(
            "saves manager: save directory '{}' created and used as current one!",
            save_name
        );
        self.current_save_name = Some(save_name.into());
        self.save_game(global_values);

        Ok(())
    }

    pub fn save_game(&mut self, global_values: &HashMap<String, Vec<SystemValue>>) {
        let mut values_list: HashMap<&str, Vec<SystemValue>> = HashMap::new();
        let mut save_values_keys_list: Vec<&str> = Vec::new();

        for value_name in &self.save_system_values {
            let value = global_values.get(value_name);
            match value {
                Some(value) => {
                    values_list.insert(value_name, value.to_vec());
                    save_values_keys_list.push(&value_name);
                }
                None => debugger::warn(
                    &format!(
                        "saves manager's save_game warning!\nfailed to get global system value '{}'",
                        value_name
                    )
                ),
            }
        }

        if let Some(current_save_name) = &self.current_save_name {
            for (value_name, value) in values_list.iter() {
                save_value_to_file(current_save_name, value, value_name);
            }

            for (value_name, value) in self.lazy_values_to_save.iter() {
                save_value_to_file(current_save_name, value, value_name);
            }
            save_value_to_file(current_save_name, &save_values_keys_list, "save_values");
        } else {
            debugger::error("save manager's save_game error!\ncurrent save file is none! load/create one first");
        }
    }

    pub fn save_lazy_value(&mut self, key: &str, value: Vec<SystemValue>) {
        self.lazy_values_to_save.insert(key.into(), value);
    }

    pub fn load_lazy_value(&self, value_name: &str) -> Option<Vec<SystemValue>> {
        match self.lazy_values_to_save.get(value_name) {
            Some(value) => Some(value.to_vec()),
            None => {
                if let Some(current_save_name) = &self.current_save_name {
                    load_value_from_file(current_save_name, value_name)
                } else {
                    debugger::error(
                        &format!(
                            "save manager's load_lazy_value error!\nfailed to load value {} from file\ncreate/load a save file first",
                            value_name
                        )
                    );

                    None
                }
            },
        }
    }
}

fn save_value_to_file(current_save_name: &str, value: &impl Serialize, value_name: &str) {
    let json = serde_json::to_string_pretty(value);

    match json {
        Ok(json) => {
            let save_file_path = "saves/".to_string() + current_save_name + "/" + value_name + ".json";
            let save_file_path = get_full_asset_path(&save_file_path);
            dbg!(&save_file_path);


            if Path::new(&save_file_path).exists() {
                if let Err(err) = fs::remove_file(&save_file_path) {
                    debugger::error(
                        &format!("save manager's save_game error!\nfailed to remove the old save file!\nerr: {}, path: {}", err, save_file_path)
                    );
                }
            }

            match File::create(&save_file_path) {
                Ok(mut file) => {
                    if let Err(err) = file.write_all(json.as_bytes()) {
                        debugger::error(
                            &format!("save manager's save_game error!\nfailed to write the file\nerr: {}, path: {}", err, save_file_path)
                        );
                    }
                },
                Err(err) => {
                    debugger::error(
                        &format!("save manager's save_game error!\nfailed to open the save file\nerr: {}, path: {}", err, save_file_path)
                    );
                }
            }
        }
        Err(err) => debugger::error(
            &format!(
                "save manager's save_game error!\n failed to serialize data!\nerror: {}",
                err
            )
        ),
    }
}

fn load_value_from_file(current_save_name: &str, value_name: &str) -> Option<Vec<SystemValue>> {
    let save_file_path = "saves/".to_string() + current_save_name + "/" + value_name + ".json";
    let save_file_path = get_full_asset_path(&save_file_path);

    let mut json = String::new();
    match File::open(&save_file_path) {
        Ok(mut file) => {
            if let Err(err) = file.read_to_string(&mut json) {
                debugger::error(
                    &format!("save manager's load_game error!\nfailed to read the save file\nerr: {}, path: {}", err, save_file_path)
                );
                return None
            }
        }
        Err(err) => {
            debugger::error(
                &format!("save manager's load_game error!\nfailed to open the save file\nerr: {}, path: {}", err, save_file_path)
            );
            return None
        }
    }

    let values: Result<Vec<SystemValue>, serde_json::Error> =
        serde_json::from_str(&json);

    match values {
        Ok(values) => Some(values),
        Err(err) => {
            debugger::error(
                &format!("save manager's load_game error!\nfailed to deserialize the save file contents!\nerr: {}, path: {}", err, save_file_path)
            );

            None
        }
    }

}
