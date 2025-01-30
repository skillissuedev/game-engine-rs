use std::{collections::HashMap, path::Path, usize};
use data_url::DataUrl;
use glam::{Mat4, Quat, Vec3};
use gltf::{animation::util::ReadOutputs, Gltf};
use splines::{Interpolation, Key, Spline};
use crate::{framework::Framework, managers::{assets::{self, get_full_asset_path}, debugger, render::{RenderManager, Vertex}}};

#[derive(Debug, Clone)]
pub(crate) enum ModelAssetError {
    LoadBuffersError,
    GlbError,
    BufferDecodingError,
    FailedToReadBin,
    GltfOpenError,
    GltfReaderError,
    CannotGetDefaultScene,
}

#[derive(Debug)]
pub(crate) struct ModelAsset {
    pub path: String,
    pub(crate) objects: HashMap<usize, ModelAssetObject>,
    pub(crate) animations: HashMap<String, ModelAssetAnimation>,
}

#[derive(Debug)]
pub(crate) struct ModelAssetObject {
    pub(crate) render_data: Vec<ModelAssetObjectRenderData>,
    pub(crate) children: HashMap<usize, ModelAssetObject>,
    pub(crate) object_name: Option<String>,
    pub(crate) default_transform: Mat4,
}

#[derive(Debug)]
pub(crate) struct ModelAssetObjectRenderData {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
    /// Vec<(joint node-id, joint joint-id XD)>
    pub(crate) joint_objects_idx: Vec<(usize, usize)>,
    pub(crate) inverse_bind_matrices: HashMap<usize, Mat4>,
}

#[derive(Debug)]
pub(crate) struct ModelAssetAnimation {
    pub(crate) channels: HashMap<usize, ModelAssetAnimationChannel>
}

#[derive(Debug)]
pub(crate) struct ModelAssetAnimationChannel {
    pub(crate) translation_x: Spline<f32, f32>,
    pub(crate) translation_y: Spline<f32, f32>,
    pub(crate) translation_z: Spline<f32, f32>,
    pub(crate) rotation_x: Spline<f32, f32>,
    pub(crate) rotation_y: Spline<f32, f32>,
    pub(crate) rotation_z: Spline<f32, f32>,
    pub(crate) rotation_w: Spline<f32, f32>,
    pub(crate) scale_x: Spline<f32, f32>,
    pub(crate) scale_y: Spline<f32, f32>,
    pub(crate) scale_z: Spline<f32, f32>,
}

impl ModelAsset {
    pub fn preload_model_asset_from_gltf(framework: &mut Framework, asset_id: String, path: &str) -> Result<(), ()> {
        match Self::from_gltf(path, framework.render.as_ref()) {
            Ok(asset) => {
                if let Err(err) = framework.assets.preload_model_asset(asset_id.to_string(), asset) {
                    debugger::error(&format!(
                        "Failed to preload the ModelAsset!\nAssetManager error: {:?}\nPath: {}",
                        err, path
                    ));
                    return Err(());
                }
            }
            Err(err) => {
                debugger::error(&format!("Failed to preload the ModelAsset!\nFailed to load the asset\nError: {:?}\nPath: {}", err, path));
                return Err(());
            }
        }
        Ok(())
    }

    pub fn from_gltf(path: &str, render: Option<&RenderManager>) -> Result<ModelAsset, ModelAssetError> {
        let path = &assets::get_full_asset_path(path);
        let gltf = Gltf::open(path);
        match gltf {
            Ok(gltf) => {
                let buffer_data = load_buffers(&gltf, path);
                match buffer_data {
                    Ok(buffer_data) => {
                        match read_gltf_scene(&gltf, &buffer_data, render) {
                            Ok(objects) => {
                                let animations = read_gltf_animations(&gltf, &buffer_data);
                                Ok(ModelAsset {
                                    objects,
                                    path: path.to_string(),
                                    animations,
                                })
                            },
                            Err(err) => return Err(err),
                        }
                    },
                    Err(err) => {
                        debugger::error(
                            &format!("ModelAsset from_gltf error: failed to load the .bin file!")
                        );
                        return Err(err)
                    },
                }
            },
            Err(err) => {
                debugger::error(
                    &format!("ModelAsset from_gltf error: failed to load the gltf file! Error: {}", err)
                );
                return Err(ModelAssetError::GltfOpenError)
            },
        }
    }
}

fn read_gltf_scene(gltf: &Gltf, buffer_data: &Vec<Vec<u8>>, render: Option<&RenderManager>) -> Result<HashMap<usize, ModelAssetObject>, ModelAssetError> {
    match gltf.default_scene() {
        Some(scene) => {
            let mut scene_objects: HashMap<usize, ModelAssetObject> = HashMap::new();

            for node in scene.nodes() {
                scene_objects.insert(node.index(), read_gltf_object(&node, buffer_data, render));
            }

            Ok(scene_objects)
        },
        None => {
            debugger::error("ModelAsset error: failed to get the default scene (read_gltf_scene)");
            Err(ModelAssetError::CannotGetDefaultScene)
        },
    }
}

fn read_gltf_object(node: &gltf::Node, buffer_data: &Vec<Vec<u8>>, render: Option<&RenderManager>) -> ModelAssetObject {
    match node.mesh() {
        Some(mesh) => {
            let mut render_data: Vec<ModelAssetObjectRenderData> = Vec::new();
            for primitive in mesh.primitives() {
                let mut indices: Vec<u32> = Vec::new();
                let mut vertices: Vec<Vertex> = Vec::new();
                let material = match primitive.material().index() {
                    Some(material) => material as u32 + 1,
                    None => 0,
                };
                let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

                match reader.read_positions() {
                    Some(positions) => {
                        for position in positions {
                            // Let's push a Vertex struct that's not filled out for now
                            vertices.push(Vertex {
                                position,
                                material,
                                ..Default::default()
                            });
                        }
                    },
                    None => debugger::warn("ModelAsset: can't read positions of a mesh! (read_gltf_object)"),
                }

                match reader.read_tex_coords(0) {
                    Some(tex_coords) => {
                        for (idx, tex_coords) in tex_coords.into_f32().enumerate() {
                            vertices[idx].tex_coords = tex_coords;
                        }
                    },
                    None => debugger::warn("ModelAsset: can't read tex coords of a mesh! (read_gltf_object)"),
                }

                match reader.read_normals() {
                    Some(normals) => {
                        for (idx, normal) in normals.enumerate() {
                            vertices[idx].normal = normal;
                        }
                    },
                    None => debugger::warn("ModelAsset: can't read normals of a mesh! (read_gltf_object)"),
                }

                match reader.read_weights(0) {
                    Some(weights) => {
                        for (idx, weight) in weights.into_f32().enumerate() {
                            vertices[idx].weights = weight;
                        }
                    },
                    None => debugger::warn("ModelAsset: can't read weights of a mesh! (read_gltf_object)"),
                }

                match reader.read_joints(0) {
                    Some(joints) => {
                        for (idx, joints) in joints.into_u16().enumerate() {
                            let f32_joints = joints.map(|joint| joint as f32);
                            vertices[idx].joints = f32_joints;
                        }
                    },
                    None => debugger::warn("ModelAsset: can't read joints of a mesh! (read_gltf_object)"),
                }

                match reader.read_indices() {
                    Some(indices_reader) => {
                        indices = indices_reader.into_u32().collect();
                    },
                    None => debugger::warn("ModelAsset: can't read indices of a mesh! (read_gltf_object)"),
                }

                let joints: Vec<(usize, usize)>;
                let mut inverse_bind_matrices: HashMap<usize, Mat4> = HashMap::new();
                match node.skin() {
                    Some(skin) => {
                        joints = skin.joints().enumerate()
                            .map(|(idx, joint)| (idx, joint.index())).collect();
                        let reader = skin.reader(|buffer| Some(&buffer_data[buffer.index()]));
                        let mut inverse_bind_matrices_vec = Vec::new();
                        if let Some(read_inverse_bind_matrices) = reader.read_inverse_bind_matrices() {
                            inverse_bind_matrices_vec = read_inverse_bind_matrices.collect();
                        }

                        for (joint_idx, (joint_object, _)) in joints.iter().enumerate() {
                            let inverse_bind_matrix =
                                Mat4::from_cols_array_2d(&inverse_bind_matrices_vec[joint_idx]);
                            inverse_bind_matrices.insert(*joint_object, inverse_bind_matrix);
                        }
                    },
                    None => {
                        joints = Vec::new();
                    }
                }

                render_data.push(ModelAssetObjectRenderData {
                    vertices,
                    indices,
                    joint_objects_idx: joints,
                    inverse_bind_matrices,
                });
            }

            let mut children = HashMap::new();
            for child in node.children() {
                children.insert(child.index(), read_gltf_object(&child, buffer_data, render));
            }

            let default_transform = Mat4::from_cols_array_2d(&node.transform().matrix());
            ModelAssetObject {
                render_data,
                children,
                object_name: node.name().map(|s| s.to_string()),
                default_transform,
            }
        },
        None => { // if object doesn't have a mesh, still add it
            let mut children = HashMap::new();
            for child in node.children() {
                children.insert(child.index(), read_gltf_object(&child, buffer_data, render));
            }

            let default_transform = Mat4::from_cols_array_2d(&node.transform().matrix());
            ModelAssetObject {
                render_data: Vec::new(),
                children,
                object_name: node.name().map(|s| s.to_string()),
                default_transform,
            }
        },
    }
}

fn read_gltf_animations(gltf: &Gltf, buffer_data: &Vec<Vec<u8>>) -> HashMap<String, ModelAssetAnimation> {
    let mut animations: HashMap<String, ModelAssetAnimation> = HashMap::new();
    for animation in gltf.animations() {
        let mut channels: HashMap<usize, ModelAssetAnimationChannel> = HashMap::new();
        let animation_name = match animation.name() {
            Some(animation_name) => animation_name.to_string(),
            None => animation.index().to_string(),
        };

        for channel in animation.channels() {
            let target_node = channel.target().node().index();

            let asset_channel = match channels.get_mut(&target_node) {
                Some(channel) => channel,
                None => {
                    channels.insert(target_node, ModelAssetAnimationChannel {
                        translation_x: Spline::from_vec(vec![]),
                        translation_y: Spline::from_vec(vec![]),
                        translation_z: Spline::from_vec(vec![]),
                        rotation_x: Spline::from_vec(vec![]),
                        rotation_y: Spline::from_vec(vec![]),
                        rotation_z: Spline::from_vec(vec![]),
                        rotation_w: Spline::from_vec(vec![]),
                        scale_x: Spline::from_vec(vec![]),
                        scale_y: Spline::from_vec(vec![]),
                        scale_z: Spline::from_vec(vec![]),
                    });
                    channels.get_mut(&target_node)
                        .expect("Should not be None, because the element was just inserted - ModelAsset (read_gltf_animations)")
                },
            };

            let reader = channel.reader(|buffer| Some(&buffer_data[buffer.index()]));

            let frame_timestamps: Vec<f32>;
            match reader.read_inputs() {
                Some(timestamps) => frame_timestamps = timestamps.collect(),
                None => {
                    debugger::warn("ModelAsset: read_inputs' result is None (read_gltf_animations)");
                    frame_timestamps = Vec::new();
                },
            };
            if frame_timestamps.is_empty() {
                // if there are 0 keyframes, skip the channel
                // had to add it because some empty would overwrite node transformations
                continue 
            }

            match reader.read_outputs() {
                Some(keyframes_iter) => {
                    match keyframes_iter {
                        ReadOutputs::Translations(translations) => {
                            for (idx, translation) in translations.enumerate() {
                                asset_channel.translation_x.add(Key::new(frame_timestamps[idx], translation[0], Interpolation::Linear));
                                asset_channel.translation_y.add(Key::new(frame_timestamps[idx], translation[1], Interpolation::Linear));
                                asset_channel.translation_z.add(Key::new(frame_timestamps[idx], translation[2], Interpolation::Linear));
                            }
                        },
                        ReadOutputs::Rotations(rotations) => {
                            for (idx, rotation) in rotations.into_f32().enumerate() {
                                asset_channel.rotation_x.add(Key::new(frame_timestamps[idx], rotation[0], Interpolation::Linear));
                                asset_channel.rotation_y.add(Key::new(frame_timestamps[idx], rotation[1], Interpolation::Linear));
                                asset_channel.rotation_z.add(Key::new(frame_timestamps[idx], rotation[2], Interpolation::Linear));
                                asset_channel.rotation_w.add(Key::new(frame_timestamps[idx], rotation[3], Interpolation::Linear));
                            }
                        },
                        ReadOutputs::Scales(scales) => {
                            for (idx, scale) in scales.enumerate() {
                                asset_channel.scale_x.add(Key::new(frame_timestamps[idx], scale[0], Interpolation::Linear));
                                asset_channel.scale_y.add(Key::new(frame_timestamps[idx], scale[1], Interpolation::Linear));
                                asset_channel.scale_z.add(Key::new(frame_timestamps[idx], scale[2], Interpolation::Linear));
                            }
                        },
                        ReadOutputs::MorphTargetWeights(_) => (),
                    }
                },
                None => {
                    debugger::warn("ModelAsset: read_outputs' result is None (read_gltf_animations)")
                },
            };
        }
        animations.insert(animation_name.into(), ModelAssetAnimation { channels });
    }

    animations
}

// Got this one from the old render :/
fn load_buffers(
    gltf: &Gltf,
    asset_path: &str
) -> Result<Vec<Vec<u8>>, ModelAssetError> {
    let mut buffer_data: Vec<Vec<u8>> = Vec::new();
    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                debugger::error(&format!("MeshAsset loading error!\nasset path: {}\nerror: .glb loading is not supported", asset_path));
                return Err(ModelAssetError::GlbError);
            }
            gltf::buffer::Source::Uri(uri) => {
                let url: DataUrl;
                match DataUrl::process(uri) {
                    Ok(url_result) => {
                        url = url_result;
                        match url.decode_to_vec() {
                            Ok(res) => {
                                buffer_data.push(res.0);
                            }
                            Err(_) => {
                                // The base64 was malformed!
                                debugger::error(&format!(
                                        "got an error when creating mesh asset\nasset path: {}\nerror: can't decode a buffer: bad base64",
                                        &asset_path));
                                return Err(ModelAssetError::BufferDecodingError);
                            }
                        }
                    }
                    Err(err) => match err {
                        data_url::DataUrlError::NotADataUrl => {
                            let bin_path = &Path::new(asset_path)
                                .with_extension("bin")
                                .into_os_string();
                            dbg!(bin_path);
                            match std::fs::read(bin_path) {
                                Ok(bin) => buffer_data.push(bin),
                                Err(err) => {
                                    debugger::error(
                                        &format!("ModelAsset loading error\nfailed to read bin file\nerr: {}", err)
                                    );
                                    return Err(ModelAssetError::FailedToReadBin);
                                }
                            };
                        }
                        _ => (),
                    },
                };
            }
        }
    }

    Ok(buffer_data)
}
