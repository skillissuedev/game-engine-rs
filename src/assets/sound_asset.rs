use ez_al::WavAsset;

use crate::{framework::Framework, managers::{assets::get_full_asset_path, debugger}};

pub struct SoundAsset {
    pub wav: WavAsset,
}

impl SoundAsset {
    pub fn from_wav(framework: &Framework, path: &str) -> Result<SoundAsset, ()> {
        if let Some(al) = &framework.al {
            let wav = WavAsset::from_wav(al, &get_full_asset_path(path));
            match wav {
                Ok(wav) => Ok(SoundAsset { wav }),
                Err(err) => {
                    debugger::error(&format!("failed to create a SoundAsset\nerr: {:?}", err));
                    Err(())
                }
            }
        } else {
            debugger::error("Failed to create a SoundAsset!\nFramework's al value = None, probably running without render");
            Err(())
        }

    }
}
