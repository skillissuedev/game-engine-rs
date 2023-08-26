use glium::{Display, Frame};
use crate::systems::System;


static mut SYSTEMS: Vec<Box<dyn System>> = vec![];

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

#[derive(Debug, Clone)]
pub struct CallList {
    pub immut_call: Vec<String>,
    pub mut_call: Vec<String>
}
