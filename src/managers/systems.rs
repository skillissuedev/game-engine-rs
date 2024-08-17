use std::collections::HashMap;

use crate::{framework::Framework, objects::ObjectGroup, systems::System};
use egui_glium::egui_winit::egui::Context;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use super::{
    assets::AssetManager,
    debugger, networking,
    render::{CurrentCascade, RenderManager},
};

static mut SYSTEMS: Vec<Box<dyn System>> = vec![];
static mut OBJECTS_ID_NAMES: Lazy<HashMap<u128, String>> = Lazy::new(|| HashMap::new());
static mut OBJECTS_ID_SYSTEMS: Lazy<HashMap<u128, String>> = Lazy::new(|| HashMap::new());
static mut OBJECTS_ID_GROUPS: Lazy<HashMap<u128, Vec<ObjectGroup>>> = Lazy::new(|| HashMap::new());

pub fn get_system_with_id(id: &str) -> Option<&Box<dyn System>> {
    unsafe {
        for system in &SYSTEMS {
            if system.system_id() == id {
                return Some(&system);
            }
        }
        return None;
    }
}

pub fn get_system_mut_with_id(id: &str) -> Option<&mut Box<dyn System>> {
    unsafe {
        for system in &mut SYSTEMS {
            if system.system_id() == id {
                return Some(system);
            }
        }
        return None;
    }
}

pub fn update(framework: &mut Framework) {
    unsafe {
        if networking::is_server() {
            for system in &mut SYSTEMS {
                system.server_update(framework);
                system.update_objects(framework);
            }
        } else {
            for system in &mut SYSTEMS {
                system.client_update(framework);
                system.update_objects(framework);
            }
        }
    }
}

pub fn get_systems_iter<'a>() -> std::slice::Iter<'a, Box<dyn System>> {
    unsafe { SYSTEMS.iter() }
}

pub fn render(framework: &mut Framework) {
    unsafe {
        if networking::is_server() {
            for system in &mut SYSTEMS {
                system.server_render();
                //system.render_objects();
            }
        } else {
            for system in &mut SYSTEMS {
                system.client_render();
                system.render_objects(framework);
            }
        }
    }
}

pub fn ui_render(ctx: &Context) {
    unsafe {
        for system in &mut SYSTEMS {
            system.ui_render(ctx);
        }
    }
}

pub fn shadow_render(render: &mut RenderManager, assets: &AssetManager, cascade: &CurrentCascade) {
    unsafe {
        if !networking::is_server() {
            for system in &mut SYSTEMS {
                system.shadow_render_objects(render, assets, cascade);
            }
        }
    }
}

pub fn add_system(system: Box<dyn System>, framework: &mut Framework) {
    unsafe {
        SYSTEMS.push(system);
        if networking::is_server() {
            SYSTEMS
                .last_mut()
                .expect("Failed to add system")
                .server_start(framework);
        } else {
            SYSTEMS
                .last_mut()
                .expect("Failed to add system")
                .client_start(framework);
        }
    }
}

pub fn register_object_id_name(id: u128, name: &str) {
    unsafe {
        match OBJECTS_ID_NAMES.get_mut(&id) {
            Some(name_in_map) => {
                *name_in_map = name.into();
            }
            None => {
                OBJECTS_ID_NAMES.insert(id, name.into());
            }
        };
    }
}

pub fn register_object_id_system(id: u128, system: &str) {
    unsafe {
        match OBJECTS_ID_SYSTEMS.get_mut(&id) {
            Some(system_in_map) => {
                *system_in_map = system.into();
            }
            None => {
                OBJECTS_ID_SYSTEMS.insert(id, system.into());
            }
        };
    }
}

pub fn register_object_id_groups(id: u128, groups: &Vec<ObjectGroup>) {
    unsafe {
        match OBJECTS_ID_GROUPS.get_mut(&id) {
            Some(group_in_map) => {
                *group_in_map = groups.to_vec();
            }
            None => {
                OBJECTS_ID_GROUPS.insert(id, groups.to_vec());
            }
        };
    }
}

pub fn get_object_groups_with_id(id: u128) -> Option<Vec<ObjectGroup>> {
    unsafe { OBJECTS_ID_GROUPS.get(&id).cloned() }
}

pub fn get_object_name_with_id(id: u128) -> Option<String> {
    unsafe { OBJECTS_ID_NAMES.get(&id).cloned() }
}

pub fn get_value_in_system(system_id: &str, value_name: String) -> Option<SystemValue> {
    match get_system_mut_with_id(system_id) {
        Some(system) => system.get_value(value_name),
        None => {
            debugger::error(&format!(
                "failed to get the system '{}' to get the value '{}'",
                system_id, value_name
            ));
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct CallList {
    pub immut_call: Vec<String>,
    pub mut_call: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SystemValue {
    String(String),
    Int(i32),
    UInt(u32),
    Float(f32),
    Bool(bool),
    Vec(Vec<SystemValue>)
}
