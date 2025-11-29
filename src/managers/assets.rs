use crate::{assets::{
    model_asset::{self, ModelAsset}, shader_asset::ShaderAsset, sound_asset::SoundAsset, texture_asset::TextureAsset
}, managers::debugger::warn};

use super::debugger::{crash, error};
use std::{collections::HashMap, env, sync::{Arc, RwLock}, time::Instant};

#[derive(Default)]
pub struct AssetManager {
    loaded_model_assets: Arc<RwLock<HashMap<String, Arc<ModelAsset>>>>,
    loaded_sound_assets: HashMap<String, SoundAsset>,
    loaded_texture_assets: HashMap<String, TextureAsset>,
    loaded_shader_assets: HashMap<String, ShaderAsset>,
}

impl AssetManager {
    pub(crate) fn preload_model_asset(
        &mut self,
        asset_id: String,
        path: String,
    ) -> Result<ModelAssetId, AssetManagerError> {
        // don't preload the asset if it's already there
        {
            let loaded_model_assets = self.loaded_model_assets.read().expect("loaded_model_assets is poisoned! :(");
            let preloaded_asset = loaded_model_assets.get(&asset_id);
            if let Some(_) = preloaded_asset {
                warn(
                    &format!("AssetManager warning! Failed to preload ModelAsset '{}'! Error: ModelAsset with this id already exists!", asset_id)
                );
                return Err(AssetManagerError::AssetAlreadyLoaded);
            }
        }

        match ModelAsset::from_gltf(&path) {
            Ok(asset) => {
                let asset = Arc::new(asset);
                self.loaded_model_assets.write().unwrap().insert(asset_id.clone(), asset);
                println!("write finished!");
            }
            Err(err) => {
                error(&format!("Failed to preload the ModelAsset!\nFailed to load the asset\nError: {:?}\nPath: {}", err, path));
                return Err(AssetManagerError::AssetCreationError);
            }
        }

        Ok(ModelAssetId { id: asset_id })
    }

    pub(crate) fn background_preload_model_asset(
        &mut self,
        asset_id: String,
        path: String,
    ) -> Result<ModelAssetId, AssetManagerError> {
        // don't preload the asset if it's already there
        {
            let loaded_model_assets = self.loaded_model_assets.read().expect("loaded_model_assets is poisoned! :(");
            let preloaded_asset = loaded_model_assets.get(&asset_id);
            if let Some(_) = preloaded_asset {
                warn(
                    &format!("AssetManager warning: Failed to preload ModelAsset '{}'! Error: ModelAsset with this id already exists!", asset_id)
                );
                return Err(AssetManagerError::AssetAlreadyLoaded)
            }
        }

        // Making an asset that'll be in place of the real asset while it's being loaded
        let temporary_asset = ModelAsset { 
            is_loaded: false, 
            path: path.to_owned(), 
            root: model_asset::ModelAssetObject {
                render_data: Vec::new(),
                children: HashMap::new(),
                object_name: None,
                default_transform: glam::Mat4::IDENTITY,
            }, 
            animations: HashMap::new(),
        };

        let loaded_model_assets = self.loaded_model_assets.clone();
        loaded_model_assets.write().unwrap().insert(asset_id.clone(), Arc::new(temporary_asset));
        let asset_id_2 = asset_id.clone();

        // start loading the real asset
        std::thread::spawn(move || {
            match ModelAsset::from_gltf(&path) {
                Ok(asset) => {
                    loaded_model_assets.write().unwrap().insert(asset_id_2, Arc::new(asset));
                }
                Err(err) => {
                    error(&format!("Failed to preload the ModelAsset!\nFailed to load the asset\nError: {:?}\nPath: {}", err, path));
                }
            }
        });

        Ok(ModelAssetId { id: asset_id })
    }

    pub fn preload_sound_asset(
        &mut self,
        asset_id: String,
        asset: SoundAsset,
    ) -> Result<SoundAssetId, AssetManagerError> {
        match self.loaded_sound_assets.get(&asset_id) {
            Some(_) => {
                warn(
                    &format!("AssetManager warning: Failed to preload SoundAsset '{}'! Error: SoundAsset with this id already exists!", asset_id)
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
                warn(
                    &format!("AssetManager warning: Failed to preload TextureAsset '{}'! Error: TextureAsset with this id already exists!", asset_id)
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
                warn(
                    &format!("AssetManager error: Failed to preload ShaderAsset '{}'! Error: ShaderAsset with this id already exists!", asset_id)
                );
                Err(AssetManagerError::AssetAlreadyLoaded)
            }
            None => {
                self.loaded_shader_assets.insert(asset_id.clone(), asset);
                Ok(ShaderAssetId { id: asset_id })
            }
        }
    }

    pub fn get_shader_asset_id(&self, id: &str) -> Option<ShaderAssetId> {
        match self.loaded_shader_assets.get(id) {
            Some(_) => Some(ShaderAssetId { id: id.into() }),
            None => None,
        }
    }

    pub fn get_model_asset_id(&self, id: &str) -> Option<ModelAssetId> {
        match self.loaded_model_assets.read().expect("loaded_model_assets is poisoned :(").get(id) {
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

    pub fn get_shader_asset(&self, asset_id: &ShaderAssetId) -> Option<&ShaderAsset> {
        match self.loaded_shader_assets.get(asset_id.get_id()) {
            Some(shader_asset) => Some(&shader_asset),
            None => None,
        }
    }

    pub fn get_shader_asset_mut(&mut self, asset_id: &ShaderAssetId) -> Option<&mut ShaderAsset> {
        match self.loaded_shader_assets.get_mut(asset_id.get_id()) {
            Some(shader_asset) => Some(shader_asset),
            None => None,
        }
    }

    pub fn get_default_texture_asset(&self) -> Option<&TextureAsset> {
        match self.loaded_texture_assets.get("default") {
            Some(texture_asset) => Some(&texture_asset),
            None => None,
        }
    }

    pub(crate) fn get_model_asset(&self, asset_id: &ModelAssetId) -> Option<Arc<ModelAsset>> {
        let loaded_model_assets = self.loaded_model_assets.read().expect("loaded_model_assets is poisoned :(");
        match loaded_model_assets.get(asset_id.get_id()) {
            Some(model_asset) => Some(model_asset.clone()),
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

#[derive(Debug, Clone)]
pub struct ShaderAssetId {
    id: String,
}

impl ShaderAssetId {
    pub fn get_id(&self) -> &str {
        &self.id
    }
}
