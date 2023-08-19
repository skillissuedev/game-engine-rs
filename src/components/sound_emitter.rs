use super::component::Component;
use crate::{
    assets::sound_asset::SoundAsset,
    managers::{
        debugger::{error, warn},
        sound::{self, SoundError},
    },
};
use allen::Source;
use ultraviolet::Vec3;

pub struct SoundEmitter {
    position: Vec3,
    rotation: Vec3,
    pub emitter_type: SoundEmitterType,
    source: Source,
}

pub enum SoundEmitterType {
    Simple,
    Positional,
}

impl SoundEmitter {
    pub fn new(
        asset: &SoundAsset,
        emitter_type: SoundEmitterType,
    ) -> Result<SoundEmitter, SoundError> {
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
            position: Vec3::zero(),
            rotation: Vec3::zero(),
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
}

impl Component for SoundEmitter {
    fn get_component_type(&self) -> &str {
        todo!()
    }

    fn set_owner(&mut self, owner: rcrefcell::RcCell<crate::object::Object>) {}

    fn get_owner(&self) -> &Option<rcrefcell::RcCell<crate::object::Object>> {
        todo!()
    }
}
