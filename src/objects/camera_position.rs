use crate::managers::{render, physics::ObjectBodyParameters};
use super::{Object, Transform};

#[derive(Debug)]
pub struct CameraPosition {
    pub name: String,
    pub transform: Transform,
    pub parent_transform: Option<Transform>,
    pub children: Vec<Box<dyn Object>>,
    pub body: Option<ObjectBodyParameters>
}

impl CameraPosition {
    pub fn new(name: &str) -> Self {
        CameraPosition { transform: Transform::default(), children: vec![], name: name.to_string(), parent_transform: None, body: None }
    }
}

impl Object for CameraPosition {
    fn start(&mut self) { }

    fn update(&mut self) {
        let global_transform = self.get_global_transform();
        render::set_camera_position(global_transform.position);
        render::set_camera_rotation(global_transform.rotation);
    }


    fn render(&mut self, _display: &mut glium::Display, _target: &mut glium::Frame) { }

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
        "CameraPosition"
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
        None
    }

    fn set_body_parameters(&mut self, rigid_body: Option<ObjectBodyParameters>) {
        self.body = rigid_body
    }

    fn get_body_parameters(&mut self) -> Option<ObjectBodyParameters> {
        self.body
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Object")
            .field("name", &self.get_name())
            .field("object_type", &self.get_object_type())
            .field("children", &self.get_children_list())
            .finish()
    }

    fn get_global_transform(&self) -> Transform {
        let base_transformations = self.get_local_transform();
        let additional_transformations: Transform;
        match self.get_parent_transform() {
            Some(transform) => additional_transformations = transform,
            None => additional_transformations = Transform::default(),
        }

        Transform {
            position: base_transformations.position + additional_transformations.position,
            rotation: base_transformations.rotation + additional_transformations.rotation,
            scale: base_transformations.scale + additional_transformations.scale,
        }
    }

    fn find_object(&self, object_name: &str) -> Option<&Box<dyn Object>> {
        for object in self.get_children_list() {
            if object.get_name() == object_name {
                return Some(object);
            }

            match object.find_object(object_name) {
                Some(found_obj) => return Some(found_obj),
                None => ()
            }
        }

        return None;
    }

    fn find_object_mut(&mut self, object_name: &str) -> Option<&mut Box<dyn Object>> {
        for object in self.get_children_list_mut() {
            if object.get_name() == object_name {
                return Some(object);
            }

            match object.find_object_mut(object_name) {
                Some(found_obj) => return Some(found_obj),
                None => ()
            }
        }

        return None;
    }

    fn update_children(&mut self) {
        let global_transform = self.get_global_transform();

        self.get_children_list_mut().into_iter().for_each(|child| {
            child.set_parent_transform(global_transform);
            child.update(); 
            child.update_children(); 
        });
    }

    fn render_children(&mut self, display: &mut glium::Display, target: &mut glium::Frame) {
        self.get_children_list_mut().into_iter().for_each(|child| child.render(display, target));
    }

    fn set_scale(&mut self, scale: glam::Vec3) {
        let mut transform = self.get_local_transform();
        transform.scale = scale;
        self.set_local_transform(transform);
    }

    fn add_child(&mut self, mut object: Box<dyn Object>) {
        object.set_parent_transform(self.get_global_transform());
        self.get_children_list_mut().push(object);
    }
}
