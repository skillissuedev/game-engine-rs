use ez_al::{WavAsset, SoundError};

use crate::managers::{assets::get_full_asset_path, debugger};

pub struct SoundAsset {
    pub wav: WavAsset
}

impl SoundAsset {
    pub fn from_wav(path: &str) -> Result<SoundAsset, SoundError> {
        let wav = WavAsset::from_wav(&get_full_asset_path(path));
        return match wav {
            Ok(wav) => Ok(SoundAsset { wav }),
            Err(err) => {
                debugger::error(&format!("failed to create a SoundAsset\nerr: {:?}", err));
                return Err(err);
            },
        }
        
    }
}
