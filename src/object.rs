use rcrefcell::RcCell;

use crate::{components::component::Component, managers::scene};

pub struct Object {
    pub name: String, 
    pub components_indexes: Vec<usize>,
}

pub struct GameObject {
    pub obj: RcCell<Object>
}

impl GameObject {
    pub fn add_component(&mut self, component: Box<dyn Component>) {
        scene::add_component(component, &self.obj);
    }
}

pub fn new_object(name: &str) -> &mut GameObject {
    let obj = Object { name: name.to_string(), components_indexes: vec![], };

    unsafe {
        scene::OBJECTS.push(GameObject { obj: RcCell::new(obj) });
    }

    unsafe {
        return scene::OBJECTS.last_mut().unwrap();
    }
}

impl Object {
    pub fn get_component(&self, component_type: &str) -> Option<&Box<dyn Component>> {
        for i in &self.components_indexes {
            unsafe {
                match &scene::COMPONENTS[i.clone()] {
                    Some(component) => {
                        if component.get_component_type() == component_type {
                            return Some(component);
                        }
                    },
                    None => continue
                }
            }
        }

        return None;
    }

    pub fn remove_component(&self, component_type: &str) {
        for i in &self.components_indexes {
            unsafe {
                match &scene::COMPONENTS[i.clone()] {
                    Some(component) => {
                        if component.get_component_type() == component_type {
                            scene::COMPONENTS[i.clone()] = None;
                        }
                    },
                    None => continue
                }
            }
        }
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        for i in &self.components_indexes {
            unsafe {
                match scene::COMPONENTS[i.clone()] {
                    Some(_) => scene::COMPONENTS[i.clone()] = None, 
                    None => ()
                }
            }
        }
    }
}

