use crate::managers::{debugger::{error, Error}, assets, render::Vertex};

#[derive(Debug, Clone)]
pub struct Object {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>
}

#[derive(Debug, Clone)]
pub struct MeshAsset {
    pub objects: Vec<Object>
}

impl MeshAsset {
    pub fn from_gltf(path: &str) -> Result<MeshAsset, Error> {
        let full_path = assets::get_full_asset_path(path);
        let gltf_result = easy_gltf::load(full_path);
        let scenes: Vec<easy_gltf::Scene>;
        match gltf_result {
            Ok(result) => scenes = result,
            Err(err) => {
                error(&format!("mesh asset creating error\nError: {:?}", err));
                return Err(Error::FileLoadingError);
            },
        }


        let mut objects: Vec<Object> = Vec::new();
        for scene in scenes {
            for model in scene.models {
                let mut indices: Vec<u16> = vec![];
                let gltf_vertices = model.vertices();
                let mut vertices: Vec<Vertex> = Vec::new();

                match model.indices() {
                    Some(ind) => {
                        let indices_usize = ind.clone();
                        for i in indices_usize {
                            indices.push(i as u16);
                        }
                    }, 
                    None => {
                        error("Can't get indices!");
                        return Err(Error::FileLoadingError);
                    },
                };

                for i in gltf_vertices { 
                    vertices.push(
                        Vertex {
                            position: [i.position.x, i.position.y, i.position.z],
                            tex_coords: [i.tex_coords.x, i.tex_coords.y],
                        }
                    );
                }

                objects.push(Object { vertices, indices });
            }
        }
        for i in &objects {
            println!("{:?}", i.vertices.len());
        }
        println!("obj {}", objects.len());

        Ok(MeshAsset { objects })
    }
}
