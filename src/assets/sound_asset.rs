use crate::{
    framework::Framework,
    managers::{assets::get_full_asset_path, debugger},
};

pub struct SoundAsset {
    pub asset: ez_al::SoundAsset,
}

impl SoundAsset {
    pub fn from_wav(framework: &Framework, path: &str) -> Result<SoundAsset, ()> {
        if let Some(al) = &framework.al {
            let wav = ez_al::SoundAsset::from_wav(al, &get_full_asset_path(path));
            match wav {
                Ok(wav) => Ok(SoundAsset { asset: wav }),
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

    pub fn preload_sound_asset_from_wav(
        framework: &mut Framework,
        asset_id: String,
        path: &str,
    ) -> Result<(), ()> {
        match Self::from_wav(framework, path) {
            Ok(asset) => {
                if let Err(err) = framework.assets.preload_sound_asset(asset_id, asset) {
                    debugger::error(&format!(
                        "Failed to preload the SoundAsset!\nAssetManager error: {:?}\nPath: {}",
                        err, path
                    ));
                    return Err(());
                }
            }
            Err(err) => {
                debugger::error(&format!("Failed to preload the SoundAsset!\nFailed to load the asset\nError: {:?}\nPath: {}", err, path));
                return Err(());
            }
        }
        Ok(())
    }

    pub fn from_mp3(framework: &Framework, path: &str) -> Result<SoundAsset, ()> {
        if let Some(al) = &framework.al {
            let mp3 = ez_al::SoundAsset::from_mp3(al, &get_full_asset_path(path));
            match mp3 {
                Ok(mp3) => Ok(SoundAsset { asset: mp3 }),
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

    pub fn preload_sound_asset_from_mp3(
        framework: &mut Framework,
        asset_id: String,
        path: &str,
    ) -> Result<(), ()> {
        match Self::from_mp3(framework, path) {
            Ok(asset) => {
                if let Err(err) = framework.assets.preload_sound_asset(asset_id, asset) {
                    debugger::error(&format!(
                        "Failed to preload the SoundAsset!\nAssetManager error: {:?}\nPath: {}",
                        err, path
                    ));
                    return Err(());
                }
            }
            Err(err) => {
                debugger::error(&format!("Failed to preload the SoundAsset!\nFailed to load the asset\nError: {:?}\nPath: {}", err, path));
                return Err(());
            }
        }
        Ok(())
    }
}
