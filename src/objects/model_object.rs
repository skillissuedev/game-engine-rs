use std::time::Instant;
use crate::assets::model_asset::{ModelAsset, Animation};
use super::{Object, Transform};

#[derive(Debug)]
pub struct ModelObject {
    pub name: String,
    pub transform: Transform,
    pub parent_transform: Option<Transform>,
    pub children: Vec<Box<dyn Object>>,
    pub asset: ModelAsset,
    pub animation_settings: CurrentAnimationSettings
}

impl ModelObject {
    pub fn new(name: &str, asset: ModelAsset) -> Self {
        ModelObject {
            transform: Transform::default(),
            children: vec![],
            name: name.to_string(),
            parent_transform: None, asset,
            animation_settings: CurrentAnimationSettings { animation: None, looping: false, timer: None }
        }
    }
}


impl Object for ModelObject {
    fn get_object_type(&self) -> &str {
        "ModelObject"
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }


    fn start(&mut self) { }

    fn update(&mut self) {
        match &self.animation_settings.animation {
            None => (),
            Some(_animation) => {
            }
        };
    }

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

impl ModelObject {
    pub fn get_asset(&self) -> &ModelAsset {
        &self.asset
    }

    pub fn play_animation(&mut self, anim_name: &str) -> Result<(), ModelObjectError> {
        let anim_option = self.asset.find_animation(anim_name);

        match anim_option {
            Some(animation) => {
                self.animation_settings = CurrentAnimationSettings {
                    animation: Some(animation),
                    looping: self.animation_settings.looping,
                    timer: Some(Instant::now())
                };

                return Ok(());
            },
            None => return Err(ModelObjectError::AnimationNotFound)
        }
    }
}

#[derive(Debug)]
pub struct CurrentAnimationSettings {
    pub animation: Option<Animation>, 
    pub looping: bool,
    pub timer: Option<Instant>
}

pub enum ModelObjectError {
    AnimationNotFound
}
