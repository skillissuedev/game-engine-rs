use glium::texture::RawImage2d;

use crate::managers::{assets::get_full_asset_path, debugger};

pub struct TextureAsset {
    pub image_raw: Vec<u8>,
    pub image_dimensions: (u32, u32)
}

#[derive(Debug)]
pub enum TextureAssetError {
    LoadError,
    TextureCreationError
}

impl TextureAsset {
    pub fn from_file(path: &str) -> Result<TextureAsset, TextureAssetError> {
        let image = image::open(get_full_asset_path(path));
        match image {
            Err(error) => {
                debugger::error(&format!("failed to create image asset\nimage error: {:?}", error));
                return Err(TextureAssetError::LoadError);
            },
            _ => ()
        }

        let image = image.unwrap().to_rgba8();
        let image_dimensions = image.dimensions();

        let image = image.into_raw();

        Ok(TextureAsset { image_raw: image, image_dimensions })

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
}

