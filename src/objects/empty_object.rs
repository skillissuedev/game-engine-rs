use glium::Display;
use crate::managers::physics::ObjectBodyParameters;

use super::{Object, Transform, gen_object_id};

#[derive(Debug)]
pub struct EmptyObject {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    body: Option<ObjectBodyParameters>,
    id: u128
}

impl EmptyObject {
    pub fn new(name: &str) -> Self {
        EmptyObject { transform: Transform::default(), children: vec![], name: name.to_string(), parent_transform: None, body: None, id: gen_object_id() }
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

    fn set_body_parameters(&mut self, rigid_body: Option<ObjectBodyParameters>) {
        self.body = rigid_body
    }

    fn get_body_parameters(&self) -> Option<ObjectBodyParameters> {
        self.body
    }

    fn get_object_id(&self) -> &u128 {
        &self.id
    }

    fn call(&mut self, name: &str, args: Vec<&str>) -> Option<String> {
        if name == "test" {
            println!("test message {}", args[0])
        }
        None
    }
}
