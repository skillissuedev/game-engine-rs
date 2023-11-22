use std::fmt::Debug;
use allen::Source;
use glam::Vec3;
use crate::{assets::sound_asset::SoundAsset, managers::{sound::{self, SoundError}, debugger::{error, warn}}};
use super::{Transform, Object};

pub struct SoundEmitter {
    pub name: String,
    pub transform: Transform,
    pub parent_transform: Option<Transform>,
    pub children: Vec<Box<dyn Object>>,
    pub emitter_type: SoundEmitterType,
    source: Source,
}

#[derive(Debug)]
pub enum SoundEmitterType {
    Simple,
    Positional,
}

impl SoundEmitter {
    pub fn new(name: &str, asset: &SoundAsset, emitter_type: SoundEmitterType) -> Result<SoundEmitter, SoundError> {
        let context = sound::take_context();
        let source_result = context.new_source();
        let source: Source;
        match source_result {
            Ok(src) => source = src,
            Err(err) => {
                error(&format!("error when creating sound emitter\nfailed to create OpenAL sound source\nerr: {}", err));
                sound::return_context(context);
                return Err(SoundError::SourceCreationFailedError(err));
            }
        }

        let _ = source.set_buffer(Some(&asset.buffer));
        match emitter_type {
            SoundEmitterType::Simple => source.set_relative(true).unwrap(),
            SoundEmitterType::Positional => {
                let _ = source.set_reference_distance(0.0);
                let _ = source.set_rolloff_factor(1.0);
                let _ = source.set_min_gain(0.0);
            }
        }

        sound::return_context(context);

        return Ok(SoundEmitter {
            name: name.to_string(),
            transform: Transform::default(),
            parent_transform: None,
            children: vec![],
            emitter_type,
            source,
        });
    }

    pub fn set_looping(&mut self, should_loop: bool) {
        let _ = self.source.set_looping(should_loop);
    }

    pub fn is_looping(&self) -> bool {
        self.source.is_looping().unwrap()
    }

    pub fn play_sound(&mut self) {
        let _ = self.source.play();
    }

    pub fn set_max_distance(&mut self, distance: f32) -> Result<(), SoundError> {
        match self.emitter_type {
            SoundEmitterType::Simple => {
                warn("tried to set max distance when emitter type is simple");
                return Err(SoundError::WrongEmitterType);
            }
            SoundEmitterType::Positional => {
                let _ = self.source.set_max_distance(distance);
                return Ok(());
            }
        }
    }

    pub fn get_max_distance(&mut self) -> Result<f32, SoundError> {
        match self.emitter_type {
            SoundEmitterType::Simple => {
                warn("tried to get max distance when emitter type is simple");
                return Err(SoundError::WrongEmitterType);
            }
            SoundEmitterType::Positional => return Ok(self.source.max_distance().unwrap()),
        }
    }

    pub fn update_sound_transforms(&mut self, sound_position: Vec3) {
        let position_result_result = self.source.set_position(sound_position.into());
        match position_result_result {
            Ok(()) => (),
            Err(error) => warn(&format!("error when trying to set sound emiiter position\nerorr: {:?}", error)),
        }
    }
}

impl Object for SoundEmitter {
    fn start(&mut self) { }

    fn update(&mut self) {
        self.update_sound_transforms(self.get_global_transform().position);
    }

    fn render(&mut self, _display: &mut glium::Display, _target: &mut glium::Frame) { }

    fn get_name(&self) -> &str {
        self.name.as_str()
    }
    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn get_object_type(&self) -> &str {
        "SoundEmitter"
    }

    fn get_children_list(&self) -> &Vec<Box<dyn Object>> {
        &self.children
    }

    fn get_children_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.children
    }

    fn get_local_transform(&self) -> Transform {
        self.transform
    }

    fn get_parent_transform(&self) -> Option<Transform> {
        self.parent_transform
    }

    fn set_parent_transform(&mut self, transform: Transform) {
        self.parent_transform = Some(transform);
    }

    fn set_local_transform(&mut self, transform: Transform) {
        self.transform = transform;
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
            .field("emitter_type", &self.emitter_type)
            .field("looping", &self.is_looping())
            .finish()
    }
}
