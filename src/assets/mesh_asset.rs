use data_url::DataUrl;
use gltf::Gltf;

use crate::managers::{
    assets,
    debugger::{error, warn},
    render::Vertex,
};

#[derive(Debug, Clone)]
pub struct Object {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub transform: [[f32; 4]; 4],
    pub node_index: usize,
}

#[derive(Debug, Clone)]
pub struct MeshAsset {
    pub objects: Vec<Object>,
    pub animations: Vec<Animation>,
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub name: String,
    pub channels: Vec<AnimationChannel>,
}

#[derive(Debug, Clone)]
pub struct AnimationChannel {
    pub channel_type: AnimationChannelType,
    pub node_index: usize,
    pub keyframe_timestamps: Vec<f32>,
    pub keyframe: Vec<Vec<f32>>,
}

#[derive(Debug, Clone)]
pub enum AnimationChannelType {
    Translation,
    Rotation,
    Scale,
}

impl MeshAsset {
    pub fn from_file(path: &str) -> Result<MeshAsset, MeshAssetError> {
        let full_path = assets::get_full_asset_path(path);
        let gltf_result = Gltf::open(&full_path);
        let gltf: Gltf;
        match gltf_result {
            Ok(result) => gltf = result,
            Err(err) => {
                error(&format!(
                    "mesh asset loading error!\nasset path: {}\nerror: {:?}",
                    &full_path, err
                ));
                return Err(MeshAssetError::LoadError);
            }
        }

        let mut buffer_data: Vec<Vec<u8>> = Vec::new();
        for buffer in gltf.buffers() {
            match buffer.source() {
                gltf::buffer::Source::Bin => {
                    error(&format!("mesh asset loading error!\nasset path: {}\nerror: .glb loading is not supported", &full_path));
                    return Err(MeshAssetError::GlbError);
                }
                gltf::buffer::Source::Uri(uri) => {
                    let url = match DataUrl::process(uri) {
                        Ok(url) => url,
                        Err(_) => {
                            // This is probably not a `data` URI? Maybe it's a https:// link or something?
                            continue;
                        }
                    };

                    match url.decode_to_vec() {
                        Ok(res) => {
                            buffer_data.push(res.0);
                        }
                        Err(_) => {
                            // The base64 was malformed!
                            error(&format!(
                                    "got an error when creating mesh asset\nasset path: {}\nerror: can't decode a buffer: bad base64",
                                    &full_path));
                            return Err(MeshAssetError::BufferDecodingError);
                        }
                    }
                }
            }
        }

        let mut objects: Vec<Object> = Vec::new();
        for scene in gltf.scenes() {
            for node in scene.nodes() {
                let node_index = node.index();
                //println!("Node '{}'", node.name().unwrap());
                let mesh_option = node.mesh();
                match mesh_option {
                    Some(mesh) => {
                        let primitives = mesh.primitives();
                        let transform = node.transform().matrix();

                        primitives.for_each(|primitive| {
                            let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

                            let mut vertices = Vec::new();
                            if let Some(vertex_attribute) = reader.read_positions() {
                                vertex_attribute.for_each(|vertex| {
                                    vertices.push(Vertex {
                                        position: vertex,
                                        tex_coords: Default::default(),
                                    })
                                });
                            } else {
                                warn(&format!("mesh asset loading warning\npath: {}\nwarning: no vertices", &full_path));
                            }
                            /*if let Some(normal_attribute) = reader.read_normals() {
                                let mut normal_index = 0;
                                normal_attribute.for_each(|normal| {
                                    //vertices[normal_index].normal = normal;

                                    normal_index += 1;
                                });
                            }*/
                            if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
                                let mut tex_coord_index = 0;
                                tex_coord_attribute.for_each(|tex_coord| {
                                    vertices[tex_coord_index].tex_coords = tex_coord;

                                    tex_coord_index += 1;
                                });
                            } else {
                                warn(&format!("mesh asset loading warning\npath: {}\nwarning: no texture coords", &full_path));
                            }

                            let mut indices = Vec::new();
                            if let Some(indices_raw) = reader.read_indices() {
                                let u32_indices = indices_raw.into_u32().collect::<Vec<u32>>();
                                u32_indices.iter().for_each(|ind| {
                                    indices.push(*ind as u16);
                               });
                            } else {
                                warn(&format!("mesh asset loading warning\npath: {}\nwarning: no texture coords", &full_path));
                            }

                            objects.push(Object { vertices, indices, transform, node_index });
                        });
                    }
                    None => (),
                }
            }
        }

        for anim in gltf.animations() {
            let channels: Vec<AnimationChannel> = Vec::new();
            anim.channels().for_each(|channel| {
                let mut keyframe_timestamps: Vec<f32> = vec![];
                let mut keyframes: Vec<Vec<f32>> = vec![];

                let reader = channel.reader(|buffer| Some(&buffer_data[buffer.index()]));
                if let Some(inputs) = reader.read_inputs() {
                    match inputs {
                        gltf::accessor::Iter::Standard(times) => {
                            let times_vec: Vec<f32> = times.collect();
                            keyframe_timestamps = times_vec;
                        }
                        gltf::accessor::Iter::Sparse(_) => {
                            error(&format!(
                                    "mesh asset loading error\npath: {}\nerror: sparse keyframes are not supported", &full_path));
                        }
                    }
                }

                if let Some(outputs) = reader.read_outputs() {
                    match outputs {
                        gltf::animation::util::ReadOutputs::Translations(translation) => {
                            translation.for_each(|tr| {
                                let vector: Vec<f32> = tr.into();
                                keyframes.push(vector);
                            });
                        },
                        gltf::animation::util::ReadOutputs::Rotations(rotation) => {
                            let rot_iter = rotation.into_f32();
                            rot_iter.for_each(|rot| {
                                keyframes.push(rot.into());
                            });
                        }
                        gltf::animation::util::ReadOutputs::Scales(scale) => {
                            scale.for_each(|sc| {
                                let vector: Vec<f32> = sc.into();
                                keyframes.push(vector);
                            });
                        },
                        gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => (),
                    }
                };
            });
        }
        todo!();
        //Ok(MeshAsset { objects })
    }
}

#[derive(Debug)]
pub enum MeshAssetError {
    LoadError,
    SparseKeyframesError,
    BufferDecodingError,
    GlbError,
}
