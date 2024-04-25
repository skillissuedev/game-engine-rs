use std::path::Path;

use crate::managers::{
    assets::{self, get_full_asset_path},
    debugger::{self, error, warn},
    render::Vertex,
};
use data_url::DataUrl;
use glam::Mat4;
use gltf::Gltf;
use splines::{Key, Spline};

#[derive(Debug, Clone)]
pub struct Object {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub transform: [[f32; 4]; 4],
    pub node_index: usize,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub transform: [[f32; 4]; 4],
    pub node_index: usize,
    pub children_id: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Joint {
    pub inverse_bind_mat: [[f32; 4]; 4],
    pub transform_mat: [[f32; 4]; 4],
    pub node_index: usize,
}

#[derive(Debug, Clone)]
pub struct ModelAsset {
    pub path: String,
    pub objects: Vec<Object>,
    pub joints: Vec<Joint>,
    pub nodes: Vec<Node>,
    pub root_nodes: Vec<Node>,
    pub animations: Vec<Animation>,
    pub joints_mats: [[[f32; 4]; 4]; 128],
    pub joints_inverse_bind_mats: [[[f32; 4]; 4]; 128],
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub name: String,
    pub channels: Vec<AnimationChannel>,
    pub duration: f32,
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
                                    error(&format!(
                                            "got an error when creating mesh asset\nasset path: {}\nerror: can't decode a buffer: bad base64",
                                            &full_path));
                                    return Err(ModelAssetError::BufferDecodingError);
                                }
                            }
                        }
                        Err(err) => match err {
                            data_url::DataUrlError::NotADataUrl => {
                                let bin_path = get_full_asset_path(
                                    &Path::new(path)
                                        .with_extension("bin")
                                        .into_os_string()
                                        .into_string()
                                        .unwrap(),
                                );
                                match std::fs::read(bin_path) {
                                    Ok(bin) => buffer_data.push(bin),
                                    Err(err) => {
                                        debugger::error(&format!(
                                                "model asset loading error\nfailed to read bin file\nerr: {}",
                                                err));
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

        let mut objects: Vec<Object> = Vec::new();
        let mut nodes: Vec<Node> = Vec::new();
        let mut root_nodes: Vec<Node> = Vec::new();
        for scene in gltf.scenes() {
            for node in scene.nodes() {
                add_object_and_children(
                    &node,
                    &buffer_data,
                    &full_path,
                    &mut objects,
                    &mut nodes,
                    Some(node.transform().matrix()),
                );
                let mut children_id: Vec<usize> = Vec::new();
                for child in node.children() {
                    children_id.push(child.index());
                }

                root_nodes.push(Node {
                    transform: node.transform().matrix(),
                    node_index: node.index(),
                    children_id,
                });
            }
        }

        let mut animations: Vec<Animation> = Vec::new();
        for anim in gltf.animations() {
            let mut animation_duration: f32 = 0.0;
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

                match keyframe_timestamps.last() {
                    Some(time) => {
                        if animation_duration < *time {
                            animation_duration = time.clone();
                        }
                    },
                    None => ()
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
                None => "".to_string(),
            };
            animations.push(Animation {
                name: animation_name,
                channels,
                duration: animation_duration,
            });
        }

        let mut joints: Vec<Joint> = Vec::new();
        for skin in gltf.skins() {
            let reader = skin.reader(|buffer| Some(&buffer_data[buffer.index()]));
            let inv_mats_option = reader.read_inverse_bind_matrices();
            let mut inv_mats: Vec<[[f32; 4]; 4]> = Vec::new();
            match inv_mats_option {
                Some(iter) => iter.for_each(|inv_mat| inv_mats.push(inv_mat)),
                None => warn("mesh asset loading warning\npath: {}\nwarning: no inverse matrices"),
            }

            let mut joint_iteration = 0;
            for joint in skin.joints() {
                joints.push(Joint {
                    inverse_bind_mat: inv_mats[joint_iteration],
                    transform_mat: joint.transform().matrix(),
                    node_index: joint.index(),
                });
                joint_iteration += 1;
            }
        }

        if objects.is_empty() {
            warn("warning when creating model asset.\n0 mesh data found");
        }

        Ok(ModelAsset {
            path: path.into(),
            objects,
            animations,
            joints: joints.clone(),
            nodes,
            root_nodes,
            joints_mats: joints_vec_to_array(joints.clone()),
            joints_inverse_bind_mats: joints_vec_to_inverse_mat_array(joints.clone()),
        })
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

        false
    }

    pub fn find_animation(&self, anim_name: &str) -> Option<Animation> {
        for anim in &self.animations {
            if anim.name == anim_name {
                return Some(anim.clone());
            }
        }

        None
    }
}

fn joints_vec_to_array(joints_vec: Vec<Joint>) -> [[[f32; 4]; 4]; 128] {
    let identity_mat: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let mut joints_mat_options_vec: Vec<Option<[[f32; 4]; 4]>> = vec![None; 200];
    let mut joints_mat_vec: Vec<[[f32; 4]; 4]> = vec![identity_mat; 200];

    if joints_vec.len() > 128 {
        warn("model asset warning! model contains more than 128 joints!\nonly 100 joints would be used");
    }
    joints_vec.into_iter().for_each(|joint| {
        joints_mat_options_vec.insert(joint.node_index, Some(joint.transform_mat))
    });

    for joint_idx in 0..joints_mat_options_vec.len() {
        match joints_mat_options_vec[joint_idx] {
            Some(mat) => joints_mat_vec.insert(joint_idx, mat),
            None => joints_mat_vec.insert(joint_idx, identity_mat),
        }
    }
    joints_mat_vec.truncate(128);

    let joints_array: [[[f32; 4]; 4]; 128] = joints_mat_vec
        .try_into()
        .expect("joints_vec_to_array failed!");
    joints_array
}

fn joints_vec_to_inverse_mat_array(joints_vec: Vec<Joint>) -> [[[f32; 4]; 4]; 128] {
    let identity_mat: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let mut inv_mat_options_vec: Vec<Option<[[f32; 4]; 4]>> = vec![None; 200];
    let mut inv_mat_vec: Vec<[[f32; 4]; 4]> = vec![identity_mat; 200];

    if joints_vec.len() > 128 {
        warn("model asset warning! model contains more than 128 joints!\nonly 100 joints would be used");
    }
    joints_vec.into_iter().for_each(|joint| {
        inv_mat_options_vec.insert(joint.node_index, Some(joint.inverse_bind_mat))
    });

    for joint_idx in 0..inv_mat_options_vec.len() {
        match inv_mat_options_vec[joint_idx] {
            Some(mat) => inv_mat_vec.insert(joint_idx, mat),
            None => inv_mat_vec.insert(joint_idx, identity_mat),
        }
    }
    inv_mat_vec.truncate(128);

    let inv_mat_array: [[[f32; 4]; 4]; 128] =
        inv_mat_vec.try_into().expect("joints_vec_to_array failed!");
    inv_mat_array
}

fn add_object_and_children(
    node: &gltf::Node,
    buffer_data: &Vec<Vec<u8>>,
    full_path: &str,
    objects: &mut Vec<Object>,
    nodes: &mut Vec<Node>,
    parent_transform_mat: Option<[[f32; 4]; 4]>,
) {
    let node_index = node.index();
    let transform = node.transform().matrix();

    let global_transform_mat: Mat4;
    match parent_transform_mat {
        Some(parent_tr_mat) => {
            global_transform_mat =
                Mat4::from_cols_array_2d(&parent_tr_mat) * Mat4::from_cols_array_2d(&transform)
        }
        None => global_transform_mat = Mat4::from_cols_array_2d(&transform),
    }
    let global_transform_mat_cols = global_transform_mat.to_cols_array_2d();

    let mesh_option = node.mesh();

    match mesh_option {
        Some(mesh) => {
            let primitives = mesh.primitives();

            primitives.for_each(|primitive| {
                let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));

                let mut vertices = Vec::new();

                if let Some(vertex_attribute) = reader.read_positions() {
                    vertex_attribute.for_each(|vertex| {
                        vertices.push(Vertex {
                            position: vertex,
                            normal: Default::default(),
                            tex_coords: Default::default(),
                            joints: Default::default(),
                            weights: Default::default(),
                        })
                    });
                } else {
                    warn(&format!(
                        "mesh asset loading warning\npath: {}\nwarning: no vertices",
                        full_path
                    ));
                }

                if let Some(normal_attribute) = reader.read_normals() {
                    let mut normal_index = 0;
                    normal_attribute.for_each(|normal| {
                        vertices[normal_index].normal = normal;

                        normal_index += 1;
                    });
                } else {
                    warn(&format!(
                        "mesh asset loading warning\npath: {}\nwarning: no normals",
                        full_path
                    ));
                }

                if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
                    let mut tex_coord_index = 0;
                    tex_coord_attribute.for_each(|tex_coord| {
                        vertices[tex_coord_index].tex_coords = tex_coord;

                        tex_coord_index += 1;
                    });
                } else {
                    warn(&format!(
                        "mesh asset loading warning\npath: {}\nwarning: no texture coords",
                        &full_path
                    ));
                }

                let mut indices = Vec::new();
                if let Some(indices_raw) = reader.read_indices() {
                    let u32_indices = indices_raw.into_u32().collect::<Vec<u32>>();
                    u32_indices.iter().for_each(|ind| {
                        indices.push(*ind as u16);
                    });
                } else {
                    warn(&format!(
                        "mesh asset loading warning\npath: {}\nwarning: no texture coords",
                        &full_path
                    ));
                }

                let mut joint_index: usize = 0;
                if let Some(joint_iter) = reader.read_joints(0) {
                    joint_iter.into_u16().for_each(|joints| {
                        let joints_f32: [f32; 4] = [
                            joints[0] as f32,
                            joints[1] as f32,
                            joints[2] as f32,
                            joints[3] as f32,
                        ];
                        vertices[joint_index].joints = joints_f32;
                        joint_index += 1;
                    });
                }

                let mut weight_index: usize = 0;
                if let Some(weights_iter) = reader.read_weights(0) {
                    weights_iter.into_f32().for_each(|weights| {
                        vertices[weight_index].weights = weights;
                        weight_index += 1;
                    });
                }

                objects.push(Object {
                    vertices,
                    indices,
                    transform: global_transform_mat_cols,
                    node_index,
                });
            });
        }
        None => (),
    };

    let mut children_id: Vec<usize> = vec![];
    for child in node.children() {
        children_id.push(child.index());
    }

    nodes.push(Node {
        transform: global_transform_mat_cols,
        node_index: node.index(),
        children_id,
    });

    for child in node.children() {
        add_object_and_children(
            &child,
            buffer_data,
            full_path,
            objects,
            nodes,
            Some(global_transform_mat_cols),
        );
    }
}

#[derive(Debug)]
pub enum ModelAssetError {
    LoadError,
    //SparseKeyframesError,
    BufferDecodingError,
    GlbError,
    FailedToReadBin, //ChannelCurveBuildingError,
}
