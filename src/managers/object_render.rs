use std::collections::HashMap;

use glam::Mat4;
use glium::{draw_parameters, framebuffer::SimpleFrameBuffer, glutin::surface::WindowSurface, texture::DepthTexture2d, uniform, uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler, UniformBuffer}, Display, DrawParameters, Program, Surface};

use crate::managers::render::RenderLayer;

use super::{assets::AssetManager, render::{Instance, RenderCamera, RenderObjectData, RenderShadowCamera}};

pub(crate) fn shadow_render_objects(close_framebuffer: &mut SimpleFrameBuffer, far_framebuffer: &mut SimpleFrameBuffer, instanced_positions: &HashMap<String, Vec<Mat4>>,
        shadow_camera: RenderShadowCamera, objects_list: &HashMap<u128, HashMap<usize, Vec<RenderObjectData>>>, display: &Display<WindowSurface>,
        program: &Program, instanced_program: &Program) {

    for (_, render_objects_list) in objects_list {
        for (_, render_node) in render_objects_list {
            for render_object in render_node {
                if !render_object.transparent {
                    shadow_draw_objects(close_framebuffer, far_framebuffer, instanced_positions, &shadow_camera, render_object, display, program, instanced_program);
                } 
            }
        }
    }

}

fn shadow_draw_objects(close_framebuffer: &mut SimpleFrameBuffer, far_framebuffer: &mut SimpleFrameBuffer, instanced_positions: &HashMap<String, Vec<Mat4>>,
        shadow_camera: &RenderShadowCamera, render_object: &RenderObjectData, display: &Display<WindowSurface>, program: &Program,
        instanced_program: &Program) {
    let joints = UniformBuffer::new(display, render_object.joint_matrices)
        .expect("UniformBuffer::new() failed (joints) - object_render.rs");
    let inverse_bind_matrices =
        UniformBuffer::new(display, render_object.joint_inverse_bind_matrices)
        .expect("UniformBuffer::new() failed (inverse_bind_matrices) - object_render.rs");

    let draw_parameters = DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
        polygon_mode: glium::draw_parameters::PolygonMode::Fill,
        ..Default::default()
    };

    let vbo = &render_object.vbo;
    let ibo = &render_object.ibo;


    match &render_object.instanced_master_name {
        Some(master_name) => {
            if let Some(transforms) = instanced_positions.get(master_name) {
                let per_instance_data: Vec<Instance> = transforms.iter()
                    .map(|model| Instance { model: model.to_cols_array_2d() }).collect();
                let per_instance_buffer =
                    glium::vertex::VertexBuffer::dynamic(display, &per_instance_data).unwrap();

                let uniforms = uniform! {
                    view: shadow_camera.close_shadow_view.to_cols_array_2d(),
                    proj: shadow_camera.close_shadow_proj.to_cols_array_2d(),
                    joint_matrices: &joints,
                    inverse_bind_matrices: &inverse_bind_matrices,
                };

                close_framebuffer.draw(
                    (vbo, per_instance_buffer.per_instance().unwrap()),
                    ibo,
                    instanced_program,
                    &uniforms,
                    &draw_parameters
                ).expect("Failed to render the instanced object to the close shadow map");

                let uniforms = uniform! {
                    view: shadow_camera.far_shadow_view.to_cols_array_2d(),
                    proj: shadow_camera.far_shadow_proj.to_cols_array_2d(),
                    joint_matrices: &joints,
                    inverse_bind_matrices: &inverse_bind_matrices,
                };

                far_framebuffer.draw(
                    (vbo, per_instance_buffer.per_instance().unwrap()),
                    ibo,
                    instanced_program,
                    &uniforms,
                    &draw_parameters
                ).expect("Failed to render the instanced object to the far shadow map");
            }
        },
        None => {
            let uniforms = uniform! {
                view: shadow_camera.close_shadow_view.to_cols_array_2d(),
                proj: shadow_camera.close_shadow_proj.to_cols_array_2d(),
                model: render_object.transform.to_cols_array_2d(),
                model_object: render_object.model_object_transform.to_cols_array_2d(),
                joint_matrices: &joints,
                inverse_bind_matrices: &inverse_bind_matrices,
            };

            close_framebuffer.draw(vbo, ibo, program, &uniforms, &draw_parameters)
                .expect("Failed to render the object to the close shadow map");
            let uniforms = uniform! {
                view: shadow_camera.far_shadow_view.to_cols_array_2d(),
                proj: shadow_camera.far_shadow_proj.to_cols_array_2d(),
                model: render_object.transform.to_cols_array_2d(),
                model_object: render_object.model_object_transform.to_cols_array_2d(),
                joint_matrices: &joints,
                inverse_bind_matrices: &inverse_bind_matrices,
            };

            far_framebuffer.draw(vbo, ibo, program, &uniforms, &draw_parameters)
                .expect("Failed to render the object to the far shadow map");
        },
    };
}

pub(crate) fn render_objects(layer_1: &mut SimpleFrameBuffer, layer_2: &mut SimpleFrameBuffer, instanced_positions: &HashMap<String, Vec<Mat4>>,
    close_shadow_texture: &DepthTexture2d, far_shadow_texture: &DepthTexture2d, objects_list: &HashMap<u128, HashMap<usize, Vec<RenderObjectData>>>,
    camera: &RenderCamera, assets: &AssetManager, display: &Display<WindowSurface>) {
    // we'll let the model object set the transformations of every node of the asset

    let mut distance_objects: Vec<(f32, &RenderObjectData)> = Vec::new();
    let mut transparent_distance_objects: Vec<(f32, &RenderObjectData)> = Vec::new();

    for (_, render_objects_list) in objects_list {
        for (_, render_node) in render_objects_list {
            for render_object in render_node {
                let translation = render_object.transform.to_scale_rotation_translation().2;
                let distance_to_camera = camera.translation.distance(translation);
                if render_object.transparent {
                    transparent_distance_objects.push((distance_to_camera, render_object));
                } else {
                    distance_objects.push((distance_to_camera, render_object));
                }
            }
        }
    }

    // Sort objects by distance
    quicksort(&mut distance_objects);
    quicksort(&mut transparent_distance_objects);
    transparent_distance_objects.reverse();

    let view_matrix = camera.get_view_matrix().to_cols_array_2d();
    let proj_matrix = camera.get_projection_matrix().to_cols_array_2d();

    draw_objects(
        layer_1, layer_2, instanced_positions, close_shadow_texture, far_shadow_texture,
        &distance_objects, assets, display, view_matrix, proj_matrix, false
    );

    draw_objects(
        layer_1, layer_2, instanced_positions, close_shadow_texture, far_shadow_texture,
        &transparent_distance_objects, assets, display, view_matrix, proj_matrix, true
    );
}

fn draw_objects(layer_1: &mut SimpleFrameBuffer, layer_2: &mut SimpleFrameBuffer, instanced_positions: &HashMap<String, Vec<Mat4>>,
    close_shadow_texture: &DepthTexture2d, far_shadow_texture: &DepthTexture2d, distance_objects: &Vec<(f32, &RenderObjectData)>,
    assets: &AssetManager, display: &Display<WindowSurface>, view_matrix: [[f32; 4]; 4], proj_matrix: [[f32; 4]; 4], transparent: bool) {
    for (_, render_object) in distance_objects {
        // Render it!
        let shader = match &render_object.shader {
            crate::managers::render::RenderShader::NotLinked => continue, // We'll skip rendering this object for now,
            crate::managers::render::RenderShader::Program(program) => program,
        };

        let texture_asset = match &render_object.texture_asset_id {
            Some(id) => match assets.get_texture_asset(&id) {
                Some(texture) => texture,
                None => assets.get_default_texture_asset()
                    .expect("Failed to get the default texture asset! - object_render"),
            },
            None => assets.get_default_texture_asset()
                .expect("Failed to get the default texture asset! - object_render"),
        };

        let texture_sampler_behavior = glium::uniforms::SamplerBehavior {
            minify_filter: MinifySamplerFilter::Nearest,
            magnify_filter: MagnifySamplerFilter::Nearest,
            ..Default::default()
        };

        let joints = UniformBuffer::new(display, render_object.joint_matrices)
            .expect("UniformBuffer::new() failed (joints) - object_render.rs");
        let inverse_bind_matrices =
            UniformBuffer::new(display, render_object.joint_inverse_bind_matrices)
            .expect("UniformBuffer::new() failed (inverse_bind_matrices) - object_render.rs");

        let mut draw_parameters = DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            polygon_mode: glium::draw_parameters::PolygonMode::Fill,
            ..Default::default()
        };

        if transparent == true {
            draw_parameters.blend = draw_parameters::Blend::alpha_blending();
        }

        let vbo = &render_object.vbo;
        let ibo = &render_object.ibo;

        let uniforms = uniform! {
            view: view_matrix,
            proj: proj_matrix,
            //model: render_object.transform.to_cols_array_2d(),
            model_object: render_object.model_object_transform.to_cols_array_2d(),
            tex: Sampler(&texture_asset.texture, texture_sampler_behavior),
            joint_matrices: &joints,
            inverse_bind_matrices: &inverse_bind_matrices,
            close_shadow_tex: Sampler(close_shadow_texture, texture_sampler_behavior),
            far_shadow_tex: Sampler(far_shadow_texture, texture_sampler_behavior),
        };


        match &render_object.instanced_master_name {
            Some(master_name) => {
                if let Some(transforms) = instanced_positions.get(master_name) {
                    let per_instance_data: Vec<Instance> = transforms.iter()
                        .map(|model| Instance { model: model.to_cols_array_2d() }).collect();

                    let per_instance_buffer =
                        glium::vertex::VertexBuffer::dynamic(display, &per_instance_data).unwrap();

                    match render_object.layer {
                        RenderLayer::Layer1 => {
                            layer_1.draw(
                                (vbo, per_instance_buffer.per_instance().unwrap()),
                                ibo, shader, &uniforms, &draw_parameters,
                            ).expect("Failed to render the object to the layer 1")
                        },
                        RenderLayer::Layer2 => {
                            layer_2.draw(
                                (vbo, per_instance_buffer.per_instance().unwrap()),
                                ibo, shader, &uniforms, &draw_parameters,
                            ).expect("Failed to render the object to the layer 2")
                        },
                    };
                }
            },
            None => {
                let uniforms = uniforms.add("model", render_object.transform.to_cols_array_2d());

                match render_object.layer {
                    RenderLayer::Layer1 => layer_1.draw(vbo, ibo, shader, &uniforms, &draw_parameters)
                        .expect("Failed to render the object to the layer 1"),
                    RenderLayer::Layer2 => layer_2.draw(vbo, ibo, shader, &uniforms, &draw_parameters)
                        .expect("Failed to render the object to the layer 2"),
                }
            },
        };
    }
}

// Sorting
fn partition(a: &mut [(f32, &RenderObjectData)]) -> usize {
    let mut i = 0;
    let right = a.len() - 1;
 
    for j in 0..right {
        if a[j].0 <= a[right].0 {
            a.swap(j, i);
            i += 1;
        }
    }
 
    a.swap(i, right);
    i
}
 
fn quicksort(a: &mut [(f32, &RenderObjectData)]) {
    if a.len() > 1 {
        let q = partition(a);
        quicksort(&mut a[..q]);
        quicksort(&mut a[q+1..]);
    }
}
