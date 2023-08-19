use crate::managers::{
    assets::get_full_asset_path,
    debugger::error,
    sound::{self, return_context, SoundError},
};
use allen::{Buffer, BufferData, Channels, Context};
use hound::WavReader;

pub struct SoundAsset {
    samples: Vec<i16>,
    pub buffer: Buffer,
}

impl SoundAsset {
    pub fn load_from_wav(path: &str) -> Result<SoundAsset, SoundError> {
        let context = sound::take_context();

        let reader = WavReader::open(get_full_asset_path(path));
        match reader {
            Ok(_) => (),
            Err(err) => {
                error(&format!("error when creating sound asset\nerr: {}", err));
                return_context(context);
                return Err(SoundError::SoundAssetLoadingError);
            }
        }
        let mut reader = reader.unwrap();

        if reader.spec().channels > 1 {
            error(&format!(
                "error when creating sound asset\nwave file contains more than 1 channel(not mono)"
            ));
            return_context(context);
            return Err(SoundError::NotMonoWavFileError);
        }

        if reader.spec().bits_per_sample != 16 {
            error(&format!(
                "error when creating sound asset\nwave file contains not 16 bit sound"
            ));
            return_context(context);
            return Err(SoundError::Not16BitWavFileError);
        }

        let samples = reader
            .samples::<i16>()
            .map(|s| s.unwrap())
            .collect::<Vec<_>>();

        let buffer = context.new_buffer();
        match buffer {
            Ok(_) => (),
            Err(err) => {
                error(&format!("error when creating sound asset\nfailed to create OpenAL sound buffer\nerr: {}", err));
                return_context(context);
                return Err(SoundError::BufferCreationFailedError(err));
            }
        }
        let buffer = buffer.unwrap();

        match buffer.data(
            BufferData::I16(&samples),
            Channels::Mono,
            reader.spec().sample_rate as i32,
        ) {
            Ok(_) => (),
            Err(err) => {
                error(&format!("error when creating sound asset\nfailed to add data to a OpenAL sound buffer\nerr: {}", err));
                return_context(context);
                return Err(SoundError::BufferCreationFailedError(err));
            }
        };

        sound::return_context(context);

        return Ok(SoundAsset { samples, buffer });
    }
}
