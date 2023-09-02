use super::{Object, Transform};

#[derive(Debug)]
pub struct EmptyObject {
    pub name: String,
    pub transform: Transform,
    pub parent_transform: Option<Transform>,
    pub children: Vec<Box<dyn Object>>,
}

impl EmptyObject {
    pub fn new(name: &str) -> Self {
        EmptyObject { transform: Transform::default(), children: vec![], name: name.to_string(), parent_transform: None }
    }
}


impl Object for EmptyObject {
    fn get_object_type(&self) -> &str {
        "EmptyObject"
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }



    fn start(&mut self) { }

    fn update(&mut self) { }

    fn render(&mut self, _display: &mut glium::Display, _target: &mut glium::Frame) { }



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


    fn get_children_list(&self) -> &Vec<Box<dyn Object>> {
        &self.children
    }

    fn get_children_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.children
    }
}
