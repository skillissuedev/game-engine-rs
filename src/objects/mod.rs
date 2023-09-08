use glium::{Display, Frame};
use ultraviolet::Vec3;

pub mod empty_object;
pub mod camera_position;
pub mod model_object;
pub mod sound_emitter;


pub trait Object: std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Object")
            .field("name", &self.get_name())
            .field("object_type", &self.get_object_type())
            .field("children", &self.get_children_list())
            .finish()
    }

    fn start(&mut self);
    fn update(&mut self);
    fn render(&mut self, display: &mut Display, target: &mut Frame);
    fn get_children_list(&self) -> &Vec<Box<dyn Object>>;
    fn get_children_list_mut(&mut self) -> &mut Vec<Box<dyn Object>>;
    fn get_name(&self) -> &str;
    fn get_object_type(&self) -> &str;
    fn set_name(&mut self, name: &str);
    fn get_local_transform(&self) -> Transform;
    fn set_local_transform(&mut self, transform: Transform);
    fn get_parent_transform(&self) -> Option<Transform>;
    fn set_parent_transform(&mut self, transform: Transform);

    fn call(&mut self, name: &str, args: Vec<&str>) -> Option<&str> { None } 

    // premade fns:
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

    fn render_children(&mut self, display: &mut Display, target: &mut Frame) {
        self.get_children_list_mut().into_iter().for_each(|child| child.render(display, target));
    }

    fn set_position(&mut self, position: Vec3) {
        let mut transform = self.get_local_transform();
        transform.position = position;
        self.set_local_transform(transform);
    }

    fn set_rotation(&mut self, rotation: Vec3) {
        let mut transform = self.get_local_transform();
        transform.rotation = rotation;
        self.set_local_transform(transform);
    }

    fn set_scale(&mut self, scale: Vec3) {
        let mut transform = self.get_local_transform();
        transform.scale = scale;
        self.set_local_transform(transform);
    }

    fn add_child(&mut self, mut object: Box<dyn Object>) {
        object.set_parent_transform(self.get_global_transform());
        self.get_children_list_mut().push(object);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3, 
    pub scale: Vec3
}

impl Default for Transform {
    fn default() -> Self {
        Transform { position: Vec3::zero(), rotation: Vec3::zero(), scale: Vec3::zero() }
    }
}
