use std::fs::read_to_string;

use crate::{
    framework::Framework,
    managers::{assets::get_full_asset_path, debugger::error},
};

pub static mut DEFAULT_VERTEX_SHADER_PATH: &str = "shaders/default.vert";
pub static mut DEFAULT_FRAGMENT_SHADER_PATH: &str = "shaders/default.frag";

pub static mut DEFAULT_FRAMEBUFFER_VERTEX_SHADER_PATH: &str = "shaders/default_framebuffer.vert";
pub static mut DEFAULT_FRAMEBUFFER_FRAGMENT_SHADER_PATH: &str = "shaders/default_framebuffer.frag";

pub static mut DEFAULT_INSTANCED_VERTEX_SHADER_PATH: &str = "shaders/default_instanced.vert";
pub static mut DEFAULT_INSTANCED_FRAGMENT_SHADER_PATH: &str = "shaders/default_instanced.frag";

#[derive(Debug, Clone)]
pub struct ShaderAsset {
    pub vertex_shader_source: String,
    pub fragment_shader_source: String,
}

#[derive(Debug, Clone)]
pub struct ShaderAssetPath {
    pub vertex_shader_path: String,
    pub fragment_shader_path: String,
}

impl ShaderAsset {
    pub fn load_default_shader() -> Result<ShaderAsset, ShaderError> {
        let shader_path = ShaderAssetPath {
            vertex_shader_path: get_default_vertex_shader_path().to_string(),
            fragment_shader_path: get_default_fragment_shader_path().to_string(),
        };

        ShaderAsset::load_from_file(&shader_path)
    }

    pub fn load_default_framebuffer_shader() -> Result<ShaderAsset, ShaderError> {
        let shader_path = ShaderAssetPath {
            vertex_shader_path: get_default_framebuffer_vertex_shader_path().to_string(),
            fragment_shader_path: get_default_framebuffer_fragment_shader_path().to_string(),
        };

        ShaderAsset::load_from_file(&shader_path)
    }

    pub fn load_default_instanced_shader() -> Result<ShaderAsset, ShaderError> {
        let shader_path = ShaderAssetPath {
            vertex_shader_path: get_default_instanced_vertex_shader_path().to_string(),
            fragment_shader_path: get_default_instanced_fragment_shader_path().to_string(),
        };

        ShaderAsset::load_from_file(&shader_path)
    }

    pub fn load_shadow_shader() -> Result<ShaderAsset, ShaderError> {
        let shader_path = ShaderAssetPath {
            vertex_shader_path: "shaders/shadow_map.vert".into(),
            fragment_shader_path: "shaders/shadow_map.frag".into(),
        };

        ShaderAsset::load_from_file(&shader_path)
    }

    pub fn load_from_file(path: &ShaderAssetPath) -> Result<ShaderAsset, ShaderError> {
        let vertex_shader_source = read_to_string(get_full_asset_path(&path.vertex_shader_path));
        let fragment_shader_source =
            read_to_string(get_full_asset_path(&path.fragment_shader_path));

        if vertex_shader_source.is_err() {
            let vertex_shader_source = vertex_shader_source.err().unwrap();
            error(&format!(
                "vertex shader asset loading error!\npath: {}\nerror:{}",
                path.vertex_shader_path, vertex_shader_source
            ));
            return Err(ShaderError::VertShaderLoadErr);
        }

        if fragment_shader_source.is_err() {
            let fragment_shader_source = fragment_shader_source.err().unwrap();
            error(&format!(
                "vertex shader asset loading error!\npath: {}\nerror:{}",
                path.vertex_shader_path, fragment_shader_source
            ));
            return Err(ShaderError::FragShaderLoadErr);
        }

        let vertex_shader_source = vertex_shader_source.ok().unwrap();
        let fragment_shader_source = fragment_shader_source.ok().unwrap();

        let asset = ShaderAsset {
            vertex_shader_source,
            fragment_shader_source,
        };

        Ok(asset)
    }

    pub fn preload_shader_asset(
        framework: &mut Framework,
        asset_id: String,
        path: ShaderAssetPath,
    ) -> Result<(), ()> {
        match Self::load_from_file(&path) {
            Ok(asset) => {
                if let Err(err) = framework.assets.preload_shader_asset(asset_id, asset) {
                    error(&format!(
                        "Failed to preload the ShaderAsset!\nAssetManager error: {:?}\nPath: {:?}",
                        err, path
                    ));
                    return Err(());
                }
            }
            Err(err) => {
                error(&format!("Failed to preload the ShaderAsset!\nFailed to load the asset\nError: {:?}\nPath: {:?}", err, path));
                return Err(());
            }
        }
        Ok(())
    }
}

pub fn get_default_vertex_shader_path() -> String {
    unsafe { DEFAULT_VERTEX_SHADER_PATH.into() }
}

pub fn get_default_fragment_shader_path() -> String {
    unsafe { DEFAULT_FRAGMENT_SHADER_PATH.into() }
}

pub fn get_default_framebuffer_vertex_shader_path() -> String {
    unsafe { DEFAULT_FRAMEBUFFER_VERTEX_SHADER_PATH.into() }
}

pub fn get_default_framebuffer_fragment_shader_path() -> String {
    unsafe { DEFAULT_FRAMEBUFFER_FRAGMENT_SHADER_PATH.into() }
}

pub fn get_default_instanced_vertex_shader_path() -> String {
    unsafe { DEFAULT_INSTANCED_VERTEX_SHADER_PATH.into() }
}

pub fn get_default_instanced_fragment_shader_path() -> String {
    unsafe { DEFAULT_INSTANCED_FRAGMENT_SHADER_PATH.into() }
}

#[derive(Debug)]
pub enum ShaderError {
    FragShaderLoadErr,
    VertShaderLoadErr,
}
