use std::fs::read_to_string;

use crate::managers::{assets::get_full_asset_path, debugger::error};

pub static mut DEFAULT_VERTEX_SHADER_PATH: &str = "shaders/default.vert";
pub static mut DEFAULT_FRAGMENT_SHADER_PATH: &str = "shaders/default.frag";

#[derive(Debug, Clone)]
pub struct ShaderAsset {
    pub vertex_shader_source: String,
    pub fragment_shader_source: String,
}

pub struct ShaderAssetPath {
    pub vertex_shader_path: String,
    pub fragment_shader_path: String,
}

impl ShaderAsset {
    pub fn load_default_shader() -> Result<ShaderAsset, ShaderError> {
        unsafe {
            let shader_path = ShaderAssetPath {
                vertex_shader_path: DEFAULT_VERTEX_SHADER_PATH.to_string(),
                fragment_shader_path: DEFAULT_FRAGMENT_SHADER_PATH.to_string(),
            };

            ShaderAsset::load_from_file(shader_path)
        }
    }

    pub fn load_from_file(path: ShaderAssetPath) -> Result<ShaderAsset, ShaderError> {
        let vertex_shader_source = read_to_string(get_full_asset_path(&path.vertex_shader_path));
        let fragment_shader_source = read_to_string(get_full_asset_path(&path.fragment_shader_path));

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
}

pub fn get_default_vertex_shader_path() -> String {
    unsafe {
        DEFAULT_VERTEX_SHADER_PATH.into()
    }
}

pub fn get_default_fragment_shader_path() -> String {
    unsafe {
        DEFAULT_FRAGMENT_SHADER_PATH.into()
    }
}


#[derive(Debug)]
pub enum ShaderError {
    FragShaderLoadErr,
    VertShaderLoadErr,
}
