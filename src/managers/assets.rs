use crate::assets::{
    model_asset::ModelAsset, shader_asset::ShaderAsset, sound_asset::SoundAsset,
    texture_asset::TextureAsset,
};

use super::debugger::{crash, error};
use std::{collections::HashMap, env};

#[derive(Default)]
pub struct AssetManager {
    loaded_model_assets: HashMap<String, ModelAsset>,
    loaded_sound_assets: HashMap<String, SoundAsset>,
    loaded_texture_assets: HashMap<String, TextureAsset>,
    loaded_shader_assets: HashMap<String, ShaderAsset>,
}

impl AssetManager {
    pub fn preload_model_asset(
        &mut self,
        asset_id: String,
        asset: ModelAsset,
    ) -> Result<ModelAssetId, AssetManagerError> {
        match self.loaded_model_assets.get(&asset_id) {
            Some(_) => {
                error(
                    &format!("AssetManager error!\nFailed to preload ModelAsset '{}'\nError: ModelAsset with this id already exists!", asset_id)
                );
                Err(AssetManagerError::AssetAlreadyLoaded)
            }
            None => {
                self.loaded_model_assets.insert(asset_id.clone(), asset);
                Ok(ModelAssetId { id: asset_id })
            }
        }
    }

    pub fn preload_sound_asset(
        &mut self,
        asset_id: String,
        asset: SoundAsset,
    ) -> Result<SoundAssetId, AssetManagerError> {
        match self.loaded_sound_assets.get(&asset_id) {
            Some(_) => {
                error(
                    &format!("AssetManager error!\nFailed to preload SoundAsset '{}'\nError: SoundAsset with this id already exists!", asset_id)
                );
                Err(AssetManagerError::AssetAlreadyLoaded)
            }
            None => {
                self.loaded_sound_assets.insert(asset_id.clone(), asset);
                Ok(SoundAssetId { id: asset_id })
            }
        }
    }

    pub fn preload_texture_asset(
        &mut self,
        asset_id: String,
        asset: TextureAsset,
    ) -> Result<TextureAssetId, AssetManagerError> {
        match self.loaded_texture_assets.get(&asset_id) {
            Some(_) => {
                error(
                    &format!("AssetManager error!\nFailed to preload TextureAsset '{}'\nError: TextureAsset with this id already exists!", asset_id)
                );
                Err(AssetManagerError::AssetAlreadyLoaded)
            }
            None => {
                self.loaded_texture_assets.insert(asset_id.clone(), asset);
                Ok(TextureAssetId { id: asset_id })
            }
        }
    }

    pub fn preload_shader_asset(
        &mut self,
        asset_id: String,
        asset: ShaderAsset,
    ) -> Result<ShaderAssetId, AssetManagerError> {
        match self.loaded_shader_assets.get(&asset_id) {
            Some(_) => {
                error(
                    &format!("AssetManager error!\nFailed to preload ShaderAsset '{}'\nError: ShaderAsset with this id already exists!", asset_id)
                );
                Err(AssetManagerError::AssetAlreadyLoaded)
            }
            None => {
                self.loaded_shader_assets.insert(asset_id.clone(), asset);
                Ok(ShaderAssetId { id: asset_id })
            }
        }
    }

    pub fn get_model_asset_id(&self, id: &str) -> Option<ModelAssetId> {
        match self.loaded_model_assets.get(id) {
            Some(_) => Some(ModelAssetId { id: id.into() }),
            None => None,
        }
    }

    pub fn get_texture_asset_id(&self, id: &str) -> Option<TextureAssetId> {
        match self.loaded_texture_assets.get(id) {
            Some(_) => Some(TextureAssetId { id: id.into() }),
            None => None,
        }
    }

    pub fn get_sound_asset_id(&self, id: &str) -> Option<SoundAssetId> {
        match self.loaded_sound_assets.get(id) {
            Some(_) => Some(SoundAssetId { id: id.into() }),
            None => None,
        }
    }

    pub fn get_sound_asset(&self, asset_id: &SoundAssetId) -> Option<&SoundAsset> {
        match self.loaded_sound_assets.get(asset_id.get_id()) {
            Some(sound_asset) => Some(&sound_asset),
            None => None,
        }
    }

    pub fn get_texture_asset(&self, asset_id: &TextureAssetId) -> Option<&TextureAsset> {
        match self.loaded_texture_assets.get(asset_id.get_id()) {
            Some(texture_asset) => Some(&texture_asset),
            None => None,
        }
    }

    pub fn get_default_texture_asset(&self) -> Option<&TextureAsset> {
        match self.loaded_texture_assets.get("default") {
            Some(texture_asset) => Some(&texture_asset),
            None => None,
        }
    }

    pub fn get_model_asset(&self, asset_id: &ModelAssetId) -> Option<&ModelAsset> {
        match self.loaded_model_assets.get(asset_id.get_id()) {
            Some(model_asset) => Some(&model_asset),
            None => None,
        }
    }
}

pub fn get_full_asset_path(path: &str) -> String {
    let mut exec_path: String = "".to_string();

    match env::current_exe() {
        Ok(exe_path) => {
            let executable_path = exe_path.to_str();
            match executable_path {
                Some(executable_path_string) => exec_path = executable_path_string.to_owned(),
                None => crash("Getting current exe path error!"),
            }
        }
        Err(err) => crash(&format!("Getting current exe path error!\nError: {}", err)),
    };

    let full_exec_path_splitted: Vec<&str> = exec_path.split("/").collect();

    let mut full_path: String = "".to_string();

    for i in 0..full_exec_path_splitted.len() - 1 {
        full_path += full_exec_path_splitted[i];
        full_path += "/";
    }

    full_path += "assets/";
    full_path += path;

    if cfg!(windows) {
        return full_path.replace("/", r"\");
    }

    full_path
}

#[derive(Debug)]
pub enum AssetManagerError {
    AssetAlreadyLoaded,
    AssetCreationError,
}

#[derive(Debug, Clone)]
pub struct ModelAssetId {
    id: String,
}

impl ModelAssetId {
    pub fn get_id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug)]
pub struct SoundAssetId {
    id: String,
}

impl SoundAssetId {
    pub fn get_id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug, Clone)]
pub struct TextureAssetId {
    id: String,
}

impl TextureAssetId {
    pub fn get_id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug)]
pub struct ShaderAssetId {
    id: String,
}

impl ShaderAssetId {
    pub fn get_id(&self) -> &str {
        &self.id
    }
}
