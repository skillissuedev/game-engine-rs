use glium::{Display, Frame};
use rcrefcell::RcCell;

use crate::{
    components::component::Component,
    object::{GameObject, Object},
};

pub static mut OBJECTS: Vec<GameObject> = vec![];
pub static mut COMPONENTS: Vec<Option<Box<dyn Component>>> = vec![];

pub fn start(display: &Display) {}

pub fn update() {
    unsafe {
        for i in &mut COMPONENTS {
            if i.is_some() {
                i.as_mut().unwrap().update();
            }
        }
    }
}

pub fn render(display: &Display, target: &mut Frame) {
    unsafe {
        for i in &mut COMPONENTS {
            if i.is_some() {
                i.as_mut().unwrap().opaque_render(display, target);
            }
        }
        for i in &mut COMPONENTS {
            if i.is_some() {
                i.as_mut().unwrap().transparent_render(display, target);
            }
        }
    }
}

pub fn add_component(component: Box<dyn Component>, owner: &RcCell<Object>) {
    unsafe {
        COMPONENTS.push(Some(component));
        COMPONENTS
            .last_mut()
            .unwrap()
            .as_mut()
            .unwrap()
            .set_owner(owner.clone());
        COMPONENTS.last_mut().unwrap().as_mut().unwrap().start();

        owner
            .borrow_mut()
            .components_indexes
            .push(COMPONENTS.len() - 1);
    }
}
