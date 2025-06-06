use std::{collections::HashMap, rc::Rc, usize};
use glam::{Mat4, Quat, Vec3};
use russimp::{mesh::Mesh, node::Node, scene::{PostProcess, Scene}};
//use gltf::{animation::util::ReadOutputs, Gltf};
use splines::{Interpolation, Key, Spline};
use crate::{framework::Framework, managers::{assets, debugger, render::{RenderManager, Vertex}}};

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

#[derive(Debug, Clone)]
pub(crate) struct ModelAsset {
    pub path: String,
    pub(crate) root: ModelAssetObject,
    pub(crate) animations: HashMap<String, ModelAssetAnimation>,
}

#[derive(Debug, Clone)]
pub(crate) struct ModelAssetObject {
    pub(crate) render_data: Vec<ModelAssetObjectRenderData>,
    pub(crate) children: HashMap<String, ModelAssetObject>,
    pub(crate) object_name: Option<String>,
    pub(crate) default_transform: Mat4,
}

#[derive(Debug, Clone)]
pub(crate) struct ModelAssetObjectRenderData {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
    pub(crate) bone_names: HashMap<usize, String>,
    pub(crate) bone_offsets: HashMap<String, Mat4>,
}

#[derive(Debug, Clone)]
pub(crate) struct ModelAssetAnimation {
    pub(crate) channels: HashMap<String, ModelAssetAnimationChannel>
}

#[derive(Debug, Clone)]
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
        match Self::from_gltf(path) {
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

    pub fn from_gltf(path: &str) -> Result<ModelAsset, ModelAssetError> {
        let path = &assets::get_full_asset_path(path);
        let scene = Scene::from_file(path, vec![
            PostProcess::Triangulate,
            PostProcess::FlipUVs,
        ]);

        match scene {
            Ok(scene) => {
                let mut root = ModelAssetObject {
                    render_data: Vec::new(),
                    children: HashMap::new(),
                    object_name: None,
                    default_transform: Mat4::IDENTITY,
                };

                if let Some(node) = &scene.root {
                    root = read_object(node, &scene.meshes);
                }

                let animations = read_animations(&scene);

                Ok(ModelAsset {
                    root,
                    path: path.to_owned(),
                    animations,
                })
            },
            Err(err) => {
                debugger::error(
                    &format!("ModelAsset from_gltf error: failed to load the file! Error: {}", err)
                );
                return Err(ModelAssetError::GltfOpenError)
            },
        }
    }
}

fn read_object(node: &Rc<Node>, meshes: &Vec<Mesh>) -> ModelAssetObject {
    let object_name = Some(node.name.clone());
    let mut render_data: Vec<ModelAssetObjectRenderData> = Vec::new();

    for mesh_idx in &node.meshes {
        let mesh = &meshes[*mesh_idx as usize];
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        
        for position in &mesh.vertices {
            // Let's push a Vertex struct that's not filled out for now
            vertices.push(Vertex {
                position: [position.x, position.y, position.z],
                material: mesh.material_index,
                ..Default::default()
            });
        }
        
        if let Some(texture_coordinates) = mesh.texture_coords.get(0) {
            if let Some(texture_coordinates) = texture_coordinates { // feels bad man
                for (idx, coord) in texture_coordinates.iter().enumerate() {
                    vertices[idx].tex_coords = [coord.x, coord.y];
                }
            }
        }
        
        for face in &mesh.faces {
            for index in &face.0 {
                indices.push(*index);
            }
        }

        for (idx, normal) in mesh.normals.iter().enumerate() {
            vertices[idx].normal = [normal.x, normal.y, normal.z];
        }

        let mut bone_names: HashMap<usize, String> = HashMap::new();
        let mut bone_offsets: HashMap<String, Mat4> = HashMap::new();
        for (bone_idx, bone) in mesh.bones.iter().enumerate() {
            for vertex_weights in &bone.weights {
                let vertex = &mut vertices[vertex_weights.vertex_id as usize];
                for (weight_idx, weight) in vertex.weights.iter().enumerate() {
                    if *weight <= 0.0 {
                        vertex.joints[weight_idx] = bone_idx as f32;
                        vertex.weights[weight_idx] = vertex_weights.weight;
                        let bone_offset = 
                            Mat4::from_cols_array(&[bone.offset_matrix.a1, bone.offset_matrix.b1, bone.offset_matrix.c1, bone.offset_matrix.d1, 
                            bone.offset_matrix.a2, bone.offset_matrix.b2, bone.offset_matrix.c2, bone.offset_matrix.d2,
                            bone.offset_matrix.a3, bone.offset_matrix.b3, bone.offset_matrix.c3, bone.offset_matrix.d3,
                            bone.offset_matrix.a4, bone.offset_matrix.b4, bone.offset_matrix.c4, bone.offset_matrix.d4]);
                        bone_offsets.insert(bone.name.clone(), bone_offset);
                        bone_names.insert(bone_idx, bone.name.clone());
                        
                        break
                    }
                }
            }
        }
        render_data.push(ModelAssetObjectRenderData { vertices, indices, bone_names, bone_offsets });
    }

    let default_transform = Mat4::from_cols_array(&[node.transformation.a1, node.transformation.b1, node.transformation.c1, node.transformation.d1, 
        node.transformation.a2, node.transformation.b2, node.transformation.c2, node.transformation.d2,
        node.transformation.a3, node.transformation.b3, node.transformation.c3, node.transformation.d3,
        node.transformation.a4, node.transformation.b4, node.transformation.c4, node.transformation.d4]);

    let mut children = HashMap::new();
    for child in node.children.take() {
        children.insert(child.name.clone(), read_object(&child, meshes));
    }


    ModelAssetObject {
        render_data,
        children,
        object_name,
        default_transform,
    }
}

fn read_animations(scene: &Scene) -> HashMap<String, ModelAssetAnimation> {
    let mut animations: HashMap<String, ModelAssetAnimation> = HashMap::new();
    for assimp_animation in &scene.animations {
        let name = assimp_animation.name.clone();
        let mut channels: HashMap<String, ModelAssetAnimationChannel> = HashMap::new();
        for channel in &assimp_animation.channels {
            let mut translation_x: Spline<f32, f32> = Spline::from_vec(vec![]);
            let mut translation_y: Spline<f32, f32> = Spline::from_vec(vec![]);
            let mut translation_z: Spline<f32, f32> = Spline::from_vec(vec![]);
            let mut rotation_x: Spline<f32, f32> = Spline::from_vec(vec![]);
            let mut rotation_y: Spline<f32, f32> = Spline::from_vec(vec![]);
            let mut rotation_z: Spline<f32, f32> = Spline::from_vec(vec![]);
            let mut rotation_w: Spline<f32, f32> = Spline::from_vec(vec![]);
            let mut scale_x: Spline<f32, f32> = Spline::from_vec(vec![]);
            let mut scale_y: Spline<f32, f32> = Spline::from_vec(vec![]);
            let mut scale_z: Spline<f32, f32> = Spline::from_vec(vec![]);

            let target = channel.name.clone();
            for pos_key in &channel.position_keys {
                let time = (pos_key.time / assimp_animation.ticks_per_second) as f32;
                translation_x.add(Key { t: time, value: pos_key.value.x, interpolation: Interpolation::Linear });
                translation_y.add(Key { t: time, value: pos_key.value.y, interpolation: Interpolation::Linear });
                translation_z.add(Key { t: time, value: pos_key.value.z, interpolation: Interpolation::Linear });
            }
            for rot_key in &channel.rotation_keys {
                let time = (rot_key.time / assimp_animation.ticks_per_second) as f32;
                rotation_x.add(Key { t: time, value: rot_key.value.x, interpolation: Interpolation::Linear });
                rotation_y.add(Key { t: time, value: rot_key.value.y, interpolation: Interpolation::Linear });
                rotation_z.add(Key { t: time, value: rot_key.value.z, interpolation: Interpolation::Linear });
                rotation_w.add(Key { t: time, value: rot_key.value.w, interpolation: Interpolation::Linear });
            }
            for sc_key in &channel.scaling_keys {
                let time = (sc_key.time / assimp_animation.ticks_per_second) as f32;
                scale_x.add(Key { t: time, value: sc_key.value.x, interpolation: Interpolation::Linear });
                scale_y.add(Key { t: time, value: sc_key.value.y, interpolation: Interpolation::Linear });
                scale_z.add(Key { t: time, value: sc_key.value.z, interpolation: Interpolation::Linear });
            }

            channels.insert(target, ModelAssetAnimationChannel {
                translation_x, translation_y, translation_z,
                rotation_x, rotation_y, rotation_z, rotation_w,
                scale_x, scale_y, scale_z,
            });
        }
        animations.insert(name, ModelAssetAnimation { channels });
    }

    animations
}
