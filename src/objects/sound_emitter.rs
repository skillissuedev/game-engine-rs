use std::fmt::Debug;
use ez_al::{SoundSource, SoundSourceType, SoundError};
use glam::Vec3;
use crate::{assets::sound_asset::SoundAsset, managers::{debugger::warn, physics::ObjectBodyParameters}};
use super::{Transform, Object};

pub struct SoundEmitter {
    pub name: String,
    pub transform: Transform,
    pub parent_transform: Option<Transform>,
    pub children: Vec<Box<dyn Object>>,
    pub source_type: SoundSourceType,
    pub source: SoundSource,
    body: Option<ObjectBodyParameters>
}

impl SoundEmitter {
    pub fn new(name: &str, asset: &SoundAsset, emitter_type: SoundSourceType) -> Result<SoundEmitter, SoundError> {
        let source = SoundSource::new(&asset.wav, emitter_type.clone());
        match source {
            Ok(source) => {
                return Ok(SoundEmitter {
                    name: name.to_string(),
                    transform: Transform::default(),
                    parent_transform: None,
                    children: vec![],
                    source_type: emitter_type,
                    source,
                    body: None
                });
            },
            Err(err) => {
                return Err(err);
            },
        }
    }

    pub fn set_looping(&mut self, should_loop: bool) {
        self.source.set_looping(should_loop);
    }

    pub fn is_looping(&self) -> bool {
        self.source.is_looping()
    }

    pub fn play_sound(&mut self) {
        self.source.play_sound();
    }

    pub fn set_max_distance(&mut self, distance: f32) -> Result<(), SoundError> {
        match self.source_type {
            SoundSourceType::Simple => {
                warn("tried to set max distance when emitter type is simple");
                return Err(SoundError::WrongSoundSourceType);
            }
            SoundSourceType::Positional => {
                let _ = self.source.set_max_distance(distance);
                return Ok(());
            }
        }
    }

    pub fn get_max_distance(&mut self) -> Result<f32, SoundError> {
        match self.source_type {
            SoundSourceType::Simple => {
                warn("tried to get max distance when emitter type is simple");
                return Err(SoundError::WrongSoundSourceType);
            }
            SoundSourceType::Positional => return Ok(self.source.get_max_distance().unwrap()),
        }
    }

    pub fn update_sound_transforms(&mut self, sound_position: Vec3) {
        let _ = self.source.update(sound_position.into());
    }
}

impl Object for SoundEmitter {
    fn start(&mut self) { }

    fn update(&mut self) {
        self.update_sound_transforms(self.get_global_transform().position);
    }

    fn render(&mut self, _display: &mut glium::Display, _target: &mut glium::Frame) { }

    fn get_children_list(&self) -> &Vec<Box<dyn Object>> {
        &self.children
    }

    fn get_children_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.children
    }

    fn get_name(&self) -> &str {
        self.name.as_str()
    }
    fn get_object_type(&self) -> &str {
        "SoundEmitter"
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn get_local_transform(&self) -> Transform {
        self.transform
    }

    fn set_local_transform(&mut self, transform: Transform) {
        self.transform = transform;
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

    fn get_body_parameters(&mut self) -> Option<ObjectBodyParameters> {
        self.body
    }

    fn call(&mut self, name: &str, args: Vec<&str>) -> Option<&str> {
        if name == "play" {
            self.play_sound();
        }
        None
    }
}

impl Debug for SoundEmitter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SoundEmitter")
            .field("name", &self.name)
            .field("object_type", &self.get_object_type())
            .field("transform", &self.transform)
            .field("parent_transform", &self.parent_transform)
            .field("children", &self.children)
            .field("emitter_type", &self.source_type)
            .field("looping", &self.is_looping())
            .finish()
    }
}
