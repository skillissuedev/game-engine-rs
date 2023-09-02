use data_url::DataUrl;
use gltf::Gltf;
use splines::{Spline, Key};
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
pub struct ModelAsset {
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
    pub x_axis_spline: Spline<f32, f32>,
    pub y_axis_spline: Spline<f32, f32>,
    pub z_axis_spline: Spline<f32, f32>,
}

#[derive(Debug, Clone)]
pub enum AnimationChannelType {
    Translation,
    Rotation,
    Scale,
}

impl ModelAsset {
    pub fn from_file(path: &str) -> Result<ModelAsset, ModelAssetError> {
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
                return Err(ModelAssetError::LoadError);
            }
        }

        let mut buffer_data: Vec<Vec<u8>> = Vec::new();
        for buffer in gltf.buffers() {
            match buffer.source() {
                gltf::buffer::Source::Bin => {
                    error(&format!("mesh asset loading error!\nasset path: {}\nerror: .glb loading is not supported", &full_path));
                    return Err(ModelAssetError::GlbError);
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
                            return Err(ModelAssetError::BufferDecodingError);
                        }
                    }
                }
            }
        }

        let mut objects: Vec<Object> = Vec::new();
        for scene in gltf.scenes() {
            for node in scene.nodes() {
                let node_index = node.index();
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

        let mut animations: Vec<Animation> = Vec::new();
        for anim in gltf.animations() {
            let mut channels: Vec<AnimationChannel> = Vec::new();
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
                            //return Err(ModelAssetError::SparseKeyframesError);
                        }
                    }
                }

                if let Some(outputs) = reader.read_outputs() {
                    match outputs {
                        gltf::animation::util::ReadOutputs::Translations(translation) => {
                            let mut x_axis_keys: Vec<Key<f32, f32>> = vec![];
                            let mut y_axis_keys: Vec<Key<f32, f32>> = vec![];
                            let mut z_axis_keys: Vec<Key<f32, f32>> = vec![];

                            let mut current_keyframe_id = 0;
                            translation.for_each(|tr| {
                                let vector: Vec<f32> = tr.into();
                                keyframes.push(vector);

                                x_axis_keys.push(Key::new(keyframe_timestamps[current_keyframe_id], tr[0], splines::Interpolation::Linear));
                                y_axis_keys.push(Key::new(keyframe_timestamps[current_keyframe_id], tr[1], splines::Interpolation::Linear));
                                z_axis_keys.push(Key::new(keyframe_timestamps[current_keyframe_id], tr[2], splines::Interpolation::Linear));
                                current_keyframe_id += 1;
                            });

                            let x_axis_spline = Spline::from_vec(x_axis_keys);
                            let y_axis_spline = Spline::from_vec(y_axis_keys);
                            let z_axis_spline = Spline::from_vec(z_axis_keys);

                            channels.push(AnimationChannel {
                                channel_type: AnimationChannelType::Translation,
                                node_index: channel.target().node().index(),
                                x_axis_spline,
                                y_axis_spline,
                                z_axis_spline,
                            });
                        },
                        gltf::animation::util::ReadOutputs::Rotations(rotation) => {
                            let mut x_axis_keys: Vec<Key<f32, f32>> = vec![];
                            let mut y_axis_keys: Vec<Key<f32, f32>> = vec![];
                            let mut z_axis_keys: Vec<Key<f32, f32>> = vec![];

                            let rot_iter = rotation.into_f32();
                            let mut current_keyframe_id = 0;
                            rot_iter.for_each(|rot| {
                                keyframes.push(rot.into());

                                x_axis_keys.push(Key::new(keyframe_timestamps[current_keyframe_id], rot[0], splines::Interpolation::Linear));
                                y_axis_keys.push(Key::new(keyframe_timestamps[current_keyframe_id], rot[1], splines::Interpolation::Linear));
                                z_axis_keys.push(Key::new(keyframe_timestamps[current_keyframe_id], rot[2], splines::Interpolation::Linear));
                                current_keyframe_id += 1;
                            });

                            let x_axis_spline = Spline::from_vec(x_axis_keys);
                            let y_axis_spline = Spline::from_vec(y_axis_keys);
                            let z_axis_spline = Spline::from_vec(z_axis_keys);

                            channels.push(AnimationChannel {
                                channel_type: AnimationChannelType::Rotation,
                                node_index: channel.target().node().index(),
                                x_axis_spline,
                                y_axis_spline,
                                z_axis_spline,
                            });
                        }
                        gltf::animation::util::ReadOutputs::Scales(scale) => {
                            let mut x_axis_keys: Vec<Key<f32, f32>> = vec![];
                            let mut y_axis_keys: Vec<Key<f32, f32>> = vec![];
                            let mut z_axis_keys: Vec<Key<f32, f32>> = vec![];

                            let mut current_keyframe_id = 0;
                            scale.for_each(|sc| {
                                let vector: Vec<f32> = sc.into();
                                keyframes.push(vector);

                                x_axis_keys.push(Key::new(keyframe_timestamps[current_keyframe_id], sc[0], splines::Interpolation::Linear));
                                y_axis_keys.push(Key::new(keyframe_timestamps[current_keyframe_id], sc[1], splines::Interpolation::Linear));
                                z_axis_keys.push(Key::new(keyframe_timestamps[current_keyframe_id], sc[2], splines::Interpolation::Linear));
                                current_keyframe_id += 1;
                            });

                            let x_axis_spline = Spline::from_vec(x_axis_keys);
                            let y_axis_spline = Spline::from_vec(y_axis_keys);
                            let z_axis_spline = Spline::from_vec(z_axis_keys);

                            channels.push(AnimationChannel {
                                channel_type: AnimationChannelType::Scale,
                                node_index: channel.target().node().index(),
                                x_axis_spline,
                                y_axis_spline,
                                z_axis_spline,
                            });
                        },
                        gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => (),
                    }
                };
            });

            let animation_name = match anim.name() {
                Some(name) => name.to_string(), 
                None => "".to_string()
            };

            animations.push(Animation { name: animation_name, channels });
        }
        Ok(ModelAsset { objects, animations })
    }

    pub fn get_animations_list(&self) -> Option<&Vec<Animation>> {
        match self.animations.is_empty() {
            true => None,
            false => Some(&self.animations),
        }
    }

    pub fn get_animations_names_list(&self) -> Option<Vec<String>> {
        match self.animations.is_empty() {
            false => {
                let mut list: Vec<String> = Vec::new();
                for anim in &self.animations {
                    list.push(anim.name.clone());
                }

                return Some(list);
            }
            true => return None,
        }
    }

    pub fn contains_animation(&self, anim_name: &str) -> bool {
        for anim in &self.animations {
            if anim.name == anim_name {
                return true;
            }
        }

        return false;
    }

    pub fn find_animation(&self, anim_name: &str) -> Option<Animation> {
        for anim in &self.animations {
            if anim.name == anim_name {
                return Some(anim.clone());
            }
        }

        return None;
    }
}

#[derive(Debug)]
pub enum ModelAssetError {
    LoadError,
    SparseKeyframesError,
    BufferDecodingError,
    GlbError,
    ChannelCurveBuildingError,
}
