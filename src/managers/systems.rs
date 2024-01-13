use std::collections::HashMap;

use glium::{Frame, Display};
use once_cell::sync::Lazy;
use crate::systems::System;

static mut SYSTEMS: Vec<Box<dyn System>> = vec![];
static mut OBJECTS_ID_NAMES: Lazy<HashMap<u128, String>> = Lazy::new(|| HashMap::new());
static mut OBJECTS_ID_SYSTEMS: Lazy<HashMap<u128, String>> = Lazy::new(|| HashMap::new());

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

pub fn update() {
    unsafe {
        for system in &mut SYSTEMS {
            system.update();
            system.update_objects();
        }
    }
}

pub fn render(display: &mut Display, target: &mut Frame) {
    unsafe {
        for system in &mut SYSTEMS {
            system.render();
            system.render_objects(display, target);
        }
    }
}

pub fn add_system(system: Box<dyn System>) {
    unsafe {
        SYSTEMS.push(system);    
        SYSTEMS.last_mut().expect("Failed to add system").start();
    }
}

pub fn register_object_id_name(id: u128, name: &str) {
    unsafe {
        OBJECTS_ID_NAMES.entry(id).or_insert(name.into());
    }
}

pub fn register_object_id_system(id: u128, system: &str) {
    unsafe {
        OBJECTS_ID_SYSTEMS.entry(id).or_insert(system.into());
    }
}


#[derive(Debug, Clone)]
pub struct CallList {
    pub immut_call: Vec<String>,
    pub mut_call: Vec<String>
}
