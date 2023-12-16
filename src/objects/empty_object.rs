use glium::Display;
use crate::managers::physics::ObjectBodyParameters;

use super::{Object, Transform};

#[derive(Debug)]
pub struct EmptyObject {
    pub name: String,
    pub transform: Transform,
    pub parent_transform: Option<Transform>,
    pub children: Vec<Box<dyn Object>>,
    pub body: Option<ObjectBodyParameters>
}

impl EmptyObject {
    pub fn new(name: &str) -> Self {
        EmptyObject { transform: Transform::default(), children: vec![], name: name.to_string(), parent_transform: None, body: None }
    }
}


impl Object for EmptyObject {
    fn start(&mut self) { }

    fn update(&mut self) { }

    fn render(&mut self, _display: &mut Display, _target: &mut glium::Frame) { }

    fn get_children_list(&self) -> &Vec<Box<dyn Object>> {
        &self.children
    }

    fn get_children_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.children
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_object_type(&self) -> &str {
        "EmptyObject"
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn get_local_transform(&self) -> Transform {
        self.transform
    }



    fn set_local_transform(&mut self, transform: Transform) {
        self.transform = transform
    }

    fn get_parent_transform(&self) -> Option<Transform> {
        self.parent_transform
    }

    fn set_parent_transform(&mut self, transform: Transform) {
        self.parent_transform = Some(transform);
    }

    fn call(&mut self, name: &str, args: Vec<&str>) -> Option<&str> {
        if name == "test" {
            println!("test message {}", args[0])
        }
        None
    }

    fn set_body_parameters(&mut self, rigid_body: Option<ObjectBodyParameters>) {
        self.body = rigid_body
    }

    fn get_body_parameters(&mut self) -> Option<ObjectBodyParameters> {
        self.body
    }
}
