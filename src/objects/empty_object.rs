use super::{gen_object_id, Object, ObjectGroup, Transform};
use crate::managers::physics::ObjectBodyParameters;

#[derive(Debug)]
pub struct EmptyObject {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    body: Option<ObjectBodyParameters>,
    id: u128,
    groups: Vec<ObjectGroup>,
}

impl EmptyObject {
    pub fn new(name: &str) -> Self {
        EmptyObject {
            transform: Transform::default(),
            children: vec![],
            name: name.to_string(),
            parent_transform: None,
            body: None,
            id: gen_object_id(),
            groups: vec![],
        }
    }
}

impl Object for EmptyObject {
    fn start(&mut self) {}

    fn update(&mut self) {}

    fn children_list(&self) -> &Vec<Box<dyn Object>> {
        &self.children
    }

    fn children_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.children
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn object_type(&self) -> &str {
        "EmptyObject"
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn local_transform(&self) -> Transform {
        self.transform
    }

    fn set_local_transform(&mut self, transform: Transform) {
        self.transform = transform
    }

    fn parent_transform(&self) -> Option<Transform> {
        self.parent_transform
    }

    fn set_parent_transform(&mut self, transform: Transform) {
        self.parent_transform = Some(transform);
    }

    fn set_body_parameters(&mut self, rigid_body: Option<ObjectBodyParameters>) {
        self.body = rigid_body
    }

    fn body_parameters(&self) -> Option<ObjectBodyParameters> {
        self.body
    }

    fn object_id(&self) -> &u128 {
        &self.id
    }

    fn groups_list(&mut self) -> &mut Vec<super::ObjectGroup> {
        &mut self.groups
    }

    fn call(&mut self, name: &str, args: Vec<&str>) -> Option<String> {
        if name == "test" {
            println!("test message {}", args[0])
        }
        None
    }
}
