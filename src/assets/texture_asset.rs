use glium::texture::{MipmapsOption, UncompressedFloatFormat};

use crate::{framework::Framework, managers::{assets::get_full_asset_path, debugger, render::RenderManager}};

pub static mut DEFAULT_TEXTURE_PATH: &str = "textures/default_texture.png";

#[derive(Debug)]
pub struct TextureAsset {
    pub texture: glium::texture::texture2d::Texture2d,
    pub dimensions: (u32, u32)
}

#[derive(Debug, Clone)]
pub enum TextureAssetError {
    LoadError,
    TextureCreationError,
}

impl TextureAsset {
    fn from_file(path: &str, render: &RenderManager) -> Result<TextureAsset, TextureAssetError> {
        let image = image::open(get_full_asset_path(path));
        match image {
            Err(error) => {
                debugger::error(&format!(
                    "failed to create image asset(path: {})\nimage error: {:?}",
                    path, error
                ));
                return Err(TextureAssetError::LoadError);
            }
            _ => (),
        }

        let image = image.unwrap().to_rgba8();
        let dimensions = image.dimensions();

        let image = image.into_raw();

        let image = glium::texture::RawImage2d::from_raw_rgba(
            image,
            dimensions,
        );
        let texture = 
            glium::texture::texture2d::Texture2d::with_format(&render.display, image, UncompressedFloatFormat::F32F32F32, MipmapsOption::NoMipmap)
            .unwrap();

        Ok(TextureAsset {
            texture,
            dimensions
        })
    }

    fn default_texture(render: &RenderManager) -> Result<TextureAsset, TextureAssetError> {
        let image = image::open(get_full_asset_path(&get_default_texture_path()));
        match image {
            Err(error) => {
                debugger::error(&format!(
                    "failed to create image asset(default_texture)\nimage error: {:?}",
                    error
                ));
                return Err(TextureAssetError::LoadError);
            }
            _ => (),
        }

        let image = image.unwrap().to_rgba8();
        let dimensions = image.dimensions();

        let image = image.into_raw();
        let image = glium::texture::RawImage2d::from_raw_rgba(
            image,
            dimensions,
        );

        let texture = 
            glium::texture::texture2d::Texture2d::with_format(&render.display, image, UncompressedFloatFormat::F32F32F32, MipmapsOption::NoMipmap)
            .unwrap();


        Ok(TextureAsset {
            texture,
            dimensions
        })
    }

    pub fn preload_texture_asset(framework: &mut Framework, asset_id: String, path: &str) -> Result<(), ()> {
        match &framework.render {
            Some(render) => {
                match Self::from_file(&path, render) {
                    Ok(asset) => {
                        if let Err(err) = framework.assets.preload_texture_asset(asset_id, asset) {
                            debugger::error(&format!("Failed to preload the TexutreAsset!\nAssetManager error: {:?}\nPath: {:?}", err, path));
                            return Err(())
                        }
                    },
                    Err(err) => {
                        debugger::error(&format!("Failed to preload the TexutreAsset!\nFailed to load the asset\nError: {:?}\nPath: {:?}", err, path));
                        return Err(())
                    },
                }
                Ok(())
            },
            None => {
                debugger::error("Failed to preload TextureAsset!\nFramework's render = None, probably running a server!");
                Err(())
            },
        }
    }
}

pub fn set_default_texture_path(path: &'static str) {
    unsafe {
        DEFAULT_TEXTURE_PATH = path;
    }
}

pub fn get_default_texture_path() -> String {
    unsafe { DEFAULT_TEXTURE_PATH.into() }
}
