use std::collections::HashMap;

use glium::{draw_parameters, framebuffer::SimpleFrameBuffer, glutin::surface::WindowSurface, texture::DepthTexture2d, uniform, uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler, UniformBuffer}, Display, DrawParameters, Surface};

use crate::managers::render::RenderLayer;

use super::{assets::AssetManager, render::{RenderCamera, RenderObjectData}};

pub(crate) fn render_objects(layer_1: &mut SimpleFrameBuffer, layer_2: &mut SimpleFrameBuffer, 
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
        layer_1, layer_2, close_shadow_texture, far_shadow_texture,
        &distance_objects, assets, display, view_matrix, proj_matrix, false
    );

    draw_objects(
        layer_1, layer_2, close_shadow_texture, far_shadow_texture,
        &transparent_distance_objects, assets, display, view_matrix, proj_matrix, true
    );
}

fn draw_objects(layer_1: &mut SimpleFrameBuffer, layer_2: &mut SimpleFrameBuffer, 
    close_shadow_texture: &DepthTexture2d, far_shadow_texture: &DepthTexture2d, distance_objects: &Vec<(f32, &RenderObjectData)>,
    assets: &AssetManager, display: &Display<WindowSurface>, view_matrix: [[f32; 4]; 4], proj_matrix: [[f32; 4]; 4], transparent: bool) {
    for (_,render_object) in distance_objects {
        // Render it!
        let shader = match &render_object.shader {
            crate::managers::render::RenderShader::NotLinked(_) => continue, // We'll skip rendering this object for now,
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

        let uniforms = uniform! {
            view: view_matrix,
            proj: proj_matrix,
            model: render_object.transform.to_cols_array_2d(),
            model_object: render_object.model_object_transform.to_cols_array_2d(),
            tex: Sampler(&texture_asset.texture, texture_sampler_behavior),
            joint_matrices: &joints,
            inverse_bind_matrices: &inverse_bind_matrices,
        };

        let mut draw_parameters = DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                ..Default::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            polygon_mode: glium::draw_parameters::PolygonMode::Fill,
            ..Default::default()
        };

        //dbg!((render_object.transform * render_object.model_object_transform).to_scale_rotation_translation());

        if transparent == true {
            draw_parameters.blend = draw_parameters::Blend::alpha_blending();
        }

        let vbo = &render_object.vbo;
        let ibo = &render_object.ibo;
        match render_object.layer {
            RenderLayer::Layer1 => layer_1.draw(vbo, ibo, shader, &uniforms, &draw_parameters)
                .expect("Failed to render the object to the layer 1"),
            RenderLayer::Layer2 => layer_2.draw(vbo, ibo, shader, &uniforms, &draw_parameters)
                .expect("Failed to render the object to the layer 2"),
        }
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
