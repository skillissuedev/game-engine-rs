use glium::{Frame, Display};
use glam::Vec3;
use serde::{Serialize, Deserialize};

use crate::managers::physics::{ObjectBodyParameters, BodyType, self};

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
    fn set_body_parameters(&mut self, rigid_body: Option<ObjectBodyParameters>);
    fn get_body_parameters(&mut self) -> Option<ObjectBodyParameters>;

    fn call(&mut self, _name: &str, _args: Vec<&str>) -> Option<&str> { 
        println!("call function is not implemented in this object.");
        return None; 
    } 

    // premade fns:
    fn get_global_transform(&self) -> Transform {
        let base_transformations = self.get_local_transform();
        let additional_transformations: Transform;
        match self.get_parent_transform() {
            Some(transform) => additional_transformations = transform,
            None => additional_transformations = Transform::default(),
        }
        //dbg!(self.get_local_transform());
        //dbg!(base_transformations.position + additional_transformations.position);

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

    
    fn update_transform(&mut self) {
        if let Some(parameters) = self.get_body_parameters() {
            let position_and_rotation_option = physics::get_body_transformations(parameters);
            //dbg!(position_and_rotation_option);

            if let Some((pos, rot)) = position_and_rotation_option {
                self.set_position(pos, false);
                self.set_rotation(rot, false);
            }
        }
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

    fn set_position(&mut self, position: Vec3, set_rigid_body_position: bool) {
        let mut transform = self.get_local_transform();
        transform.position = position;
        self.set_local_transform(transform);

        if let Some(parameters) = self.get_body_parameters() {
            if set_rigid_body_position == true {
                physics::set_body_position(parameters, position);
            }
        }
    }

    fn set_rotation(&mut self, rotation: Vec3, set_rigid_body_rotation: bool) {
        let mut transform = self.get_local_transform();
        transform.rotation = rotation;
        self.set_local_transform(transform);

        if let Some(parameters) = self.get_body_parameters() {
            if set_rigid_body_rotation == true {
                physics::set_body_rotation(parameters, rotation);
            }
        }
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

    fn build_object_rigid_body(&mut self, body_type: Option<BodyType>, mass: f32) {
        match body_type {
            Some(body_type) => {
                dbg!(self.get_global_transform());
                self.set_body_parameters(Some(physics::new_rigid_body(body_type, Some(self.get_global_transform()), mass)))
            },
            None => {
                match self.get_body_parameters() {
                    Some(body) => {
                        physics::remove_rigid_body(body);
                    },
                    None => (),
                }
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3, 
    pub scale: Vec3
}

impl Default for Transform {
    fn default() -> Self {
        Transform { position: Vec3::ZERO, rotation: Vec3::ZERO, scale: Vec3::ONE }
    }
}

/*pub enum ObjectType {
    EmptyObject,
    ModelObject,
    SoundEmitterObject,
    CameraPositionObject
}*/

