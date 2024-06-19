use crate::{framework::Framework, managers::{assets::get_full_asset_path, debugger}};

pub static mut DEFAULT_TEXTURE_PATH: &str = "textures/default_texture.png";

#[derive(Debug, Clone)]
pub struct TextureAsset {
    pub image_raw: Vec<u8>,
    pub image_dimensions: (u32, u32),
}

#[derive(Debug, Clone)]
pub enum TextureAssetError {
    LoadError,
    TextureCreationError,
}

impl TextureAsset {
    pub fn from_file(path: &str) -> Result<TextureAsset, TextureAssetError> {
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
        let image_dimensions = image.dimensions();

        let image = image.into_raw();

        Ok(TextureAsset {
            image_raw: image,
            image_dimensions,
        })

        //let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);

        /*
        let texture = glium::texture::texture2d::Texture2d::new(framework::get_display(), image);

        match texture {
        Ok(tx) => return Ok(TextureAsset { texture: tx }),
        Err(err) => {
        error(&format!("Texture creation error!\nError: {}", err));
        return Err(TextureAssetError::TextureCreationError)
        },
        }
        */
    }

    pub fn default_texture() -> Result<TextureAsset, TextureAssetError> {
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
        let image_dimensions = image.dimensions();

        let image = image.into_raw();

        Ok(TextureAsset {
            image_raw: image,
            image_dimensions,
        })
    }

    pub fn preload_texture_asset(framework: &mut Framework, asset_id: String, path: &str) -> Result<(), ()> {
        match Self::from_file(&path) {
            Ok(asset) => 
                if let Err(err) = framework.assets.preload_texture_asset(asset_id, asset) {
                    debugger::error(&format!("Failed to preload the TexutreAsset!\nAssetManager error: {:?}\nPath: {:?}", err, path));
                    return Err(())
                },
            Err(err) => {
                debugger::error(&format!("Failed to preload the TexutreAsset!\nFailed to load the asset\nError: {:?}\nPath: {:?}", err, path));
                return Err(())
            },
        }
        Ok(())
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
