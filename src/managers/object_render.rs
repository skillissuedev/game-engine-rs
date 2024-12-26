use std::collections::HashMap;
use glam::{Mat4, Vec3};
use glium::{framebuffer::SimpleFrameBuffer, glutin::surface::WindowSurface, uniform, uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerWrapFunction, UniformBuffer}, Display, IndexBuffer, Surface};
use crate::{assets::model_asset::ModelAsset, math_utils::deg_vec_to_rad, objects::{model_object::NodeTransform, Transform}};
use super::{assets::AssetManager, debugger, render::{Cascades, ModelData, ShadowTextures}};

#[derive(Debug)]
struct SetupMatrixResult {
    pub mvp: Mat4,
    pub model: Mat4,
}

pub fn render_opaque_models(
    cascades: &Cascades, 
    shadow_textures: &ShadowTextures,
    display: &Display<WindowSurface>,
    layer_1: &mut SimpleFrameBuffer,
    layer_2: &mut SimpleFrameBuffer,
    assets: &AssetManager,
    models_list: &HashMap<u128, ModelData>,
    light_direction: Vec3,
    camera_position: Vec3,
    view_mat: Mat4,
    projection_mat: Mat4) {

    // Vec<(ModelData, distance)>
    let mut sorted: Vec<(&ModelData, f32)> = Vec::new();

    for (_, model) in models_list {
        let position = model.transform.position;
        let distance = position.distance(camera_position);

        let mut last_element_with_less_distance: usize = 0;
        for (idx, (_, element_distance)) in sorted.iter().enumerate() {
            if &distance > element_distance {
                last_element_with_less_distance = idx;
            } else {
                break
            }
        }
        if last_element_with_less_distance > 0 {
            sorted.insert(last_element_with_less_distance + 1, (model, distance));
        } else {
            sorted.push((model, distance));
        }
    }

    for (model, model_distance) in sorted {
        let closest_shadow_view_proj_cols = cascades.closest_view_proj.to_cols_array_2d();
        let furthest_shadow_view_proj_cols = cascades.furthest_view_proj.to_cols_array_2d();

        if let Some(asset) = assets.get_model_asset(&model.model_asset_id) {
            let vertex_buffers = &asset.vertex_buffers.as_ref().unwrap();
            for i in 0..asset.objects.len() {
                let vertex_buffer = &vertex_buffers[i];
                let object = &asset.objects[i];

                let indices = IndexBuffer::new(
                    display,
                    glium::index::PrimitiveType::TrianglesList,
                    &object.indices,
                );

                let mut node_transform: Option<&NodeTransform> = None;
                for tr in &model.nodes_transforms {
                    if tr.node_id == asset.objects[i].node_index {
                        node_transform = Some(tr);
                        break;
                    }
                }

                match node_transform {
                    Some(_) => (),
                    None => {
                        debugger::error("no node transform found!");
                        return;
                    }
                }

                let setup_mat_result = setup_mat(view_mat, projection_mat, &model.transform, node_transform.unwrap());
                let mvp_matrix: Mat4 = setup_mat_result.mvp;
                let model_matrix: Mat4 = setup_mat_result.model;

                let texture: &glium::texture::Texture2d;
                match model.texture_asset_id.as_ref() {
                    Some(texture_id) => {
                        match assets.get_texture_asset(texture_id) {
                            Some(texture_asset) => texture = &texture_asset.texture,
                            None => texture = &assets.get_default_texture_asset()
                                .expect(
                                    "Failed to get default texture asset from preloaded assets!",
                                )
                                .texture,
                        };
                    }
                    None => {
                        texture = &assets.get_default_texture_asset()
                            .expect("Failed to get default texture asset from preloaded assets!")
                            .texture
                    }
                }

                let mvp_matrix_cols = mvp_matrix.to_cols_array_2d();
                let model_matrix_cols = model_matrix.to_cols_array_2d();

                let joints =
                    UniformBuffer::new(display, get_joints_transforms(&model.nodes_transforms, asset)).unwrap();
                let inverse_bind_mats =
                    UniformBuffer::new(display, asset.joints_inverse_bind_mats).unwrap();
                let camera_position: [f32; 3] = camera_position.into();

                let sampler_behaviour = glium::uniforms::SamplerBehavior {
                    minify_filter: MinifySamplerFilter::Linear,
                    magnify_filter: MagnifySamplerFilter::Linear,
                    max_anisotropy: 8,
                    wrap_function: (
                        SamplerWrapFunction::Repeat,
                        SamplerWrapFunction::Repeat,
                        SamplerWrapFunction::Repeat,
                    ),
                    ..Default::default()
                };

                let uniforms = uniform! {
                    is_instanced: false,
                    jointsMats: &joints,
                    jointsInverseBindMats: &inverse_bind_mats,
                    mesh: object.transform,
                    mvp: [
                        mvp_matrix_cols[0],
                        mvp_matrix_cols[1],
                        mvp_matrix_cols[2],
                        mvp_matrix_cols[3],
                    ],
                    model: [
                        model_matrix_cols[0],
                        model_matrix_cols[1],
                        model_matrix_cols[2],
                        model_matrix_cols[3],
                    ],
                    tex: glium::uniforms::Sampler(texture, sampler_behaviour),
                    lightPos: light_direction.to_array(),
                    closestShadowTexture: &shadow_textures.closest,
                    furthestShadowTexture: &shadow_textures.furthest,
                    closestShadowViewProj: [
                        closest_shadow_view_proj_cols[0],
                        closest_shadow_view_proj_cols[1],
                        closest_shadow_view_proj_cols[2],
                        closest_shadow_view_proj_cols[3],
                    ],
                    furthestShadowViewProj: [
                        furthest_shadow_view_proj_cols[0],
                        furthest_shadow_view_proj_cols[1],
                        furthest_shadow_view_proj_cols[2],
                        furthest_shadow_view_proj_cols[3],
                    ],
                    cameraPosition: camera_position,
                };

                let draw_params = glium::DrawParameters {
                    depth: glium::Depth {
                        test: glium::draw_parameters::DepthTest::IfLess,
                        write: true,
                        ..Default::default()
                    },
                    //blend: glium::draw_parameters::Blend::alpha_blending(),
                    backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                    polygon_mode: glium::draw_parameters::PolygonMode::Fill,
                    ..Default::default()
                };

                match model.layer {
                    crate::managers::render::RenderLayers::Layer1 => {
                        layer_1
                            .draw(
                                vertex_buffer,
                                &indices.unwrap(),
                                &model.programs[i],
                                &uniforms,
                                &draw_params,
                            )
                            .unwrap();
                    },
                    crate::managers::render::RenderLayers::Layer2 => {
                        layer_2
                            .draw(
                                vertex_buffer,
                                &indices.unwrap(),
                                &model.programs[i],
                                &uniforms,
                                &draw_params,
                            )
                            .unwrap();
                    },
                };
            }
        }
    }
}

pub fn render_transparent_models(
    cascades: &Cascades, 
    shadow_textures: &ShadowTextures,
    display: &Display<WindowSurface>,
    layer_1: &mut SimpleFrameBuffer,
    layer_2: &mut SimpleFrameBuffer,
    assets: &AssetManager,
    models_list: &HashMap<u128, ModelData>,
    light_direction: Vec3,
    camera_position: Vec3,
    view_mat: Mat4,
    projection_mat: Mat4) {

    // Vec<(ModelData, distance)>
    let mut sorted: Vec<(&ModelData, f32)> = Vec::new();

    for (_, model) in models_list {
        let position = model.transform.position;
        let distance = position.distance(camera_position);

        let mut last_element_with_less_distance: usize = 0;
        for (idx, (_, element_distance)) in sorted.iter().enumerate() {
            if &distance > element_distance {
                last_element_with_less_distance = idx;
            } else {
                break
            }
        }
        if last_element_with_less_distance > 0 {
            sorted.insert(last_element_with_less_distance + 1, (model, distance));
        } else {
            sorted.push((model, distance));
        }
    }

    for (model, _) in sorted.iter().rev() {
        let closest_shadow_view_proj_cols = cascades.closest_view_proj.to_cols_array_2d();
        let furthest_shadow_view_proj_cols = cascades.furthest_view_proj.to_cols_array_2d();

        if let Some(asset) = assets.get_model_asset(&model.model_asset_id) {
            let vertex_buffers = &asset.vertex_buffers.as_ref().unwrap();
            for i in 0..asset.objects.len() {
                let vertex_buffer = &vertex_buffers[i];
                let object = &asset.objects[i];

                let indices = IndexBuffer::new(
                    display,
                    glium::index::PrimitiveType::TrianglesList,
                    &object.indices,
                );

                let mut node_transforms: Option<&NodeTransform> = None;
                for tr in &model.nodes_transforms {
                    if tr.node_id == asset.objects[i].node_index {
                        node_transforms = Some(tr);
                        break;
                    }
                }

                match node_transforms {
                    Some(_) => (),
                    None => {
                        debugger::error("no node transform found!");
                        return;
                    }
                }

                let setup_mat_result = setup_mat(view_mat, projection_mat, &model.transform, node_transforms.unwrap());
                let mvp_matrix: Mat4 = setup_mat_result.mvp;
                let model_matrix: Mat4 = setup_mat_result.model;

                let texture: &glium::texture::Texture2d;
                match model.texture_asset_id.as_ref() {
                    Some(texture_id) => {
                        match assets.get_texture_asset(texture_id) {
                            Some(texture_asset) => texture = &texture_asset.texture,
                            None => texture = &assets.get_default_texture_asset()
                                .expect(
                                    "Failed to get default texture asset from preloaded assets!",
                                )
                                .texture,
                        };
                    }
                    None => {
                        texture = &assets.get_default_texture_asset()
                            .expect("Failed to get default texture asset from preloaded assets!")
                            .texture
                    }
                }

                let mvp_matrix_cols = mvp_matrix.to_cols_array_2d();
                let model_matrix_cols = model_matrix.to_cols_array_2d();

                let joints =
                    UniformBuffer::new(display, get_joints_transforms(&model.nodes_transforms, asset)).unwrap();
                let inverse_bind_mats =
                    UniformBuffer::new(display, asset.joints_inverse_bind_mats).unwrap();
                let camera_position: [f32; 3] = camera_position.into();

                let sampler_behaviour = glium::uniforms::SamplerBehavior {
                    minify_filter: MinifySamplerFilter::Linear,
                    magnify_filter: MagnifySamplerFilter::Linear,
                    max_anisotropy: 8,
                    wrap_function: (
                        SamplerWrapFunction::Repeat,
                        SamplerWrapFunction::Repeat,
                        SamplerWrapFunction::Repeat,
                    ),
                    ..Default::default()
                };

                let uniforms = uniform! {
                    is_instanced: false,
                    jointsMats: &joints,
                    jointsInverseBindMats: &inverse_bind_mats,
                    mesh: object.transform,
                    mvp: [
                        mvp_matrix_cols[0],
                        mvp_matrix_cols[1],
                        mvp_matrix_cols[2],
                        mvp_matrix_cols[3],
                    ],
                    model: [
                        model_matrix_cols[0],
                        model_matrix_cols[1],
                        model_matrix_cols[2],
                        model_matrix_cols[3],
                    ],
                    tex: glium::uniforms::Sampler(texture, sampler_behaviour),
                    lightPos: light_direction.to_array(),
                    closestShadowTexture: &shadow_textures.closest,
                    furthestShadowTexture: &shadow_textures.furthest,
                    closestShadowViewProj: [
                        closest_shadow_view_proj_cols[0],
                        closest_shadow_view_proj_cols[1],
                        closest_shadow_view_proj_cols[2],
                        closest_shadow_view_proj_cols[3],
                    ],
                    furthestShadowViewProj: [
                        furthest_shadow_view_proj_cols[0],
                        furthest_shadow_view_proj_cols[1],
                        furthest_shadow_view_proj_cols[2],
                        furthest_shadow_view_proj_cols[3],
                    ],
                    cameraPosition: camera_position,
                };

                let draw_params = glium::DrawParameters {
                    depth: glium::Depth {
                        test: glium::draw_parameters::DepthTest::IfLess,
                        write: true,
                        ..Default::default()
                    },
                    blend: glium::draw_parameters::Blend::alpha_blending(),
                    backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                    polygon_mode: glium::draw_parameters::PolygonMode::Fill,
                    ..Default::default()
                };

                match model.layer {
                    crate::managers::render::RenderLayers::Layer1 => {
                        layer_1
                            .draw(
                                vertex_buffer,
                                &indices.unwrap(),
                                &model.programs[i],
                                &uniforms,
                                &draw_params,
                            )
                            .unwrap();
                    },
                    crate::managers::render::RenderLayers::Layer2 => {
                        layer_2
                            .draw(
                                vertex_buffer,
                                &indices.unwrap(),
                                &model.programs[i],
                                &uniforms,
                                &draw_params,
                            )
                            .unwrap();
                    },
                };
            }
        }
    }
}

fn setup_mat(
    view: Mat4,
    proj: Mat4,
    transform: &Transform,
    node_transform: &NodeTransform,
) -> SetupMatrixResult {
    let node_global_transform = node_transform.global_transform;

    let model_object_translation: [f32; 3] = transform.position.into();
    let model_object_rotation_vec = deg_vec_to_rad(transform.rotation.into());
    let model_object_scale = transform.scale;
    let model_object_translation = [-model_object_translation[0], model_object_translation[1], -model_object_translation[2]];


    let rotation = model_object_rotation_vec;//node_rotation.transform_vector3(model_object_rotation_vec);
    let transform = Mat4::from_translation(Vec3::from_array(model_object_translation))
        * Mat4::from_rotation_z(-rotation.z)
        * Mat4::from_rotation_y(-rotation.y)
        * Mat4::from_rotation_x(-rotation.x)
        * Mat4::from_scale(model_object_scale);
    let transform = transform * node_global_transform;

    let mvp = proj * view * transform;

    SetupMatrixResult {
        mvp,
        model: transform,
    }
}

fn get_joints_transforms(nodes_transforms: &Vec<NodeTransform>, asset: &ModelAsset) -> [[[f32; 4]; 4]; 128] {
    let mut joints_vec: Vec<&NodeTransform> = Vec::new();

    for joint in &asset.joints {
        for node_transform in nodes_transforms {
            if node_transform.node_id == joint.node_index {
                joints_vec.push(node_transform);
            }
        }
    }

    let identity_mat: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
        let mut joints_mat_options_vec: Vec<Option<[[f32; 4]; 4]>> = vec![None; 200];
        let mut joints_mat_vec: Vec<[[f32; 4]; 4]> = vec![identity_mat; 200];

        if joints_vec.len() > 128 {
            debugger::warn("model object warning! model contains more than 128 joints!\nonly 100 joints will be used");
        }
        joints_vec.into_iter().for_each(|joint| {
            joints_mat_options_vec.insert(joint.node_id, Some(joint.global_transform.to_cols_array_2d()));
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

