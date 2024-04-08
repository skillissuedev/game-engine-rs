use std::collections::HashMap;

use crate::{objects::ObjectGroup, systems::System};
use glam::Mat4;
use glium::{framebuffer::SimpleFrameBuffer, Display, Frame};
use once_cell::sync::Lazy;

use super::{networking, render::{Cascades, ShadowTextures}};

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

pub fn update() {
    unsafe {
        if networking::is_server() {
            for system in &mut SYSTEMS {
                system.server_update();
                system.update_objects();
            }
        } else {
            for system in &mut SYSTEMS {
                system.client_update();
                system.update_objects();
            }
        }
    }
}

pub fn render(display: &Display, target: &mut Frame, cascades: &Cascades, shadow_textures: &ShadowTextures) {
    unsafe {
        if networking::is_server() {
            for system in &mut SYSTEMS {
                system.server_render();
                //system.render_objects();
            }
        } else {
            for system in &mut SYSTEMS {
                system.client_render();
                system.render_objects(display, target, cascades, shadow_textures);
            }
        }
    }
}

pub fn shadow_render(view_proj: &Mat4, display: &Display, target: &mut SimpleFrameBuffer) {
    unsafe {
        if !networking::is_server() {
            for system in &mut SYSTEMS {
                system.shadow_render_objects(view_proj, display, target);
            }
        }
    }
}

pub fn add_system(system: Box<dyn System>) {
    unsafe {
        SYSTEMS.push(system);
        if networking::is_server() {
            SYSTEMS
                .last_mut()
                .expect("Failed to add system")
                .server_start();
        } else {
            SYSTEMS
                .last_mut()
                .expect("Failed to add system")
                .client_start();
        }
    }
}

pub fn register_object_id_name(id: u128, name: &str) {
    unsafe {
        dbg!(id);
        dbg!(name);
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

#[derive(Debug, Clone)]
pub struct CallList {
    pub immut_call: Vec<String>,
    pub mut_call: Vec<String>,
}
