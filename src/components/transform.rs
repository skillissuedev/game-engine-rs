use std::{collections::HashMap, fmt};

use rcrefcell::RcCell;
use ultraviolet::vec::Vec3;

use crate::object::Object;
use super::component::Component;

pub trait Transform {
    fn get_position(&self) -> &Vec3;
    fn get_rotation(&self) -> &Vec3;
    fn get_scale(&self) -> &Vec3;
}

pub struct BasicTransform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    owner: Option<RcCell<Object>>
}

impl BasicTransform {
    pub fn new(position: Vec3, rotation: Vec3, scale: Vec3) -> BasicTransform {
        BasicTransform { position, rotation, scale, owner: None }
    }

    pub fn empty() -> BasicTransform {
        BasicTransform { position: Vec3::zero(), rotation: Vec3::zero(), scale: Vec3::zero(), owner: None }
    }
}

impl Component for BasicTransform {
    fn get_component_type(&self) -> &str {
        "Transform"
    }

    fn set_owner(&mut self, owner: RcCell<Object>) {
        self.owner = Some(owner);
    }

    fn get_owner(&self) -> &Option<RcCell<Object>> {
        &self.owner
    }

    fn get_data(&self) -> Option<HashMap<&str, String>> {
        let mut data: HashMap<&str, String> = HashMap::new();
        data.insert("position", format!("{};{};{}", self.position.x, self.position.y, self.position.z));
        data.insert("rotation", format!("{};{};{}", self.rotation.x, self.rotation.y, self.rotation.z));
        data.insert("scale", format!("{};{};{}", self.scale.x, self.scale.y, self.scale.z));
        return Some(data);
    }
}

impl From<HashMap<&str, String>> for BasicTransform {
     fn from(value: HashMap<&str, String>) -> Self {
        let position: Vec3;
        let rotation: Vec3;
        let scale: Vec3;
        let pos_splited: Vec<&str> = value["position"].split(";").collect();
        let rot_splited: Vec<&str> = value["rotation"].split(";").collect();
        let scale_splited: Vec<&str> = value["scale"].split(";").collect();

        position = Vec3::new(
            pos_splited[0].parse().unwrap(),
            pos_splited[1].parse().unwrap(),
            pos_splited[2].parse().unwrap()
        );

        rotation = Vec3::new(
            rot_splited[0].parse().unwrap(),
            rot_splited[1].parse().unwrap(),
            rot_splited[2].parse().unwrap()
        );

        scale = Vec3::new(
            scale_splited[0].parse().unwrap(),
            scale_splited[1].parse().unwrap(),
            scale_splited[2].parse().unwrap()
        );

        return BasicTransform {
            position,
            rotation,
            scale,
            owner: None
        };
    }
}

impl Transform for BasicTransform {
    fn get_position(&self) -> &Vec3 {
        &self.position
    }

    fn get_rotation(&self) -> &Vec3 {
        &self.rotation
    }

    fn get_scale(&self) -> &Vec3 {
        &self.scale
    }
}

impl fmt::Debug for BasicTransform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BasicTransform")
         .field("position", &self.position)
         .field("rotation", &self.rotation)
         .field("scale", &self.scale)
         .finish()
    }
}
