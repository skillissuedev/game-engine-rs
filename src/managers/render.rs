use std::collections::HashMap;
use egui_glium::EguiGlium;
use glam::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use glium::{framebuffer::SimpleFrameBuffer, glutin::surface::WindowSurface, implement_vertex, index::{NoIndices, PrimitiveType}, texture::DepthTexture2d, uniform, Display, DrawParameters, Frame, IndexBuffer, Program, Surface, Texture2d, VertexBuffer};

use crate::{assets::shader_asset::ShaderAsset, math_utils::deg_to_rad};

use super::{assets::{AssetManager, TextureAssetId}, debugger, object_render};

#[derive(Copy, Clone, Debug, Default)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    pub joints: [f32; 4],
    pub weights: [f32; 4],
    pub material: u32,
}
implement_vertex!(Vertex, position, normal, tex_coords, joints, weights, material);

#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct Instance {
    pub(crate) model: [[f32; 4]; 4],
}
implement_vertex!(Instance, model);

pub(crate) struct RenderManager {
    pub(crate) objects: HashMap<u128, HashMap<String, Vec<RenderObjectData>>>,
    pub(crate) window_size: (u32, u32),
    pub(crate) textures: RenderManagerTextures,
    pub(crate) camera: RenderCamera,
    pub(crate) display: Display<WindowSurface>,
    pub(crate) framebuffer_vbo: VertexBuffer<Vertex>,
    pub(crate) framebuffer_program: Program,
    pub(crate) instanced_positions: HashMap<String, Vec<Mat4>>,
    pub(crate) shadow_map_shader: Program,
    pub(crate) instanced_shadow_map_shader: Program,
    pub(crate) lights: Vec<RenderPointLight>,
    pub(crate) directional_light_dir: Vec3,
    pub(crate) directional_light_strength: f32,
    pub(crate) shadow_camera: RenderShadowCamera,
}

impl RenderManager {
    pub fn new(display: Display<WindowSurface>) -> RenderManager {
        let resolution = Self::calculate_resolution((1280, 720));
        let textures = RenderManagerTextures {
            close_shadow_texture: 
                DepthTexture2d::empty(&display, 2048, 2048)
                    .expect("close_shadow_texture creation error!"),
            far_shadow_texture:
                DepthTexture2d::empty(&display, 2048, 2048)
                    .expect("far_shadow_texture creation error!"),
            layer_1_texture:
                Texture2d::empty(&display, resolution.0, resolution.1)
                    .expect("layer_1_texture creation error!"),
            layer_2_texture:
                Texture2d::empty(&display, resolution.0, resolution.1)
                    .expect("layer_2_texture creation error!"),
            layer_1_depth: 
                DepthTexture2d::empty(&display, resolution.0, resolution.1)
                    .expect("layer_1_depth creation error!"),
            layer_2_depth:
                DepthTexture2d::empty(&display, resolution.0, resolution.1)
                    .expect("layer_2_depth creation error!"),
        };
        let camera = RenderCamera::new((1280, 720));

        let directional_light_dir = Vec3::new(-0.5, -0.6, -0.4).normalize();
        let shadow_camera = RenderShadowCamera::new(&camera, directional_light_dir);

        let framebuffer_vbo = VertexBuffer::new(&display, &[
            Vertex { position: [-1.0, 1.0, 0.0], tex_coords: [0.0, 1.0], ..Default::default() },
            Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 0.0], ..Default::default() },
            Vertex { position: [1.0, 1.0, 0.0], tex_coords: [1.0, 1.0], ..Default::default() },
            Vertex { position: [1.0, 1.0, 0.0], tex_coords: [1.0, 1.0], ..Default::default() },
            Vertex { position: [-1.0, -1.0, 0.0], tex_coords: [0.0, 0.0], ..Default::default() },
            Vertex { position: [1.0, -1.0, 0.0], tex_coords: [1.0, 0.0], ..Default::default() },
        ]).expect("Failed to create framebuffer_vbo - RenderManager (new)");

        let framebuffer_shader_asset = ShaderAsset::load_default_framebuffer_shader()
            .expect("Failed to load the default framebuffer shader - RenderManager (new)");
        let framebuffer_program = Program::from_source(
            &display,
            &framebuffer_shader_asset.vertex_shader_source,
            &framebuffer_shader_asset.fragment_shader_source,
            None,
        ).expect("Failed to make the default framebuffer program - RenderManager (new)");

        let shadow_shader_asset = ShaderAsset::load_shadow_shader()
            .expect("Failed to load the shadow map shader - RenderManager (new)");
        let shadow_map_shader = Program::from_source(
            &display,
            &shadow_shader_asset.vertex_shader_source,
            &shadow_shader_asset.fragment_shader_source,
            None,
        ).expect("Failed to compile the shadow map program - RenderManager (new)");

        let instanced_shadow_shader_asset = ShaderAsset::load_instanced_shadow_shader()
            .expect("Failed to load the instanced shadow map shader - RenderManager (new)");
        let instanced_shadow_map_shader = Program::from_source(
            &display,
            &instanced_shadow_shader_asset.vertex_shader_source,
            &instanced_shadow_shader_asset.fragment_shader_source,
            None,
        ).expect("Failed to compile the instanced shadow map program - RenderManager (new)");

        RenderManager {
            objects: HashMap::new(),
            window_size: (1280, 720),
            textures,
            camera,
            display,
            framebuffer_vbo,
            framebuffer_program,
            instanced_positions: HashMap::new(),
            shadow_map_shader,
            instanced_shadow_map_shader,
            lights: Vec::new(),
            directional_light_dir,
            directional_light_strength: 0.6,
            shadow_camera,
        }
    }

    pub fn render_scene(&mut self, assets: &AssetManager, egui_glium: &mut EguiGlium) {
        // 1. Clean everything (frame, FBs etc.)
        // 2. Render shadow map (to do later)
        // 3. Render objects
        // 4. Show the framebuffers on screen
        let display = &self.display;
        let mut frame = display.draw();
        // move this it in framework? because creating an FB everyframe could be expensive
        let mut layer1_framebuffer =
            SimpleFrameBuffer::with_depth_buffer(display, &self.textures.layer_1_texture, &self.textures.layer_1_depth)
                .expect("Failed to create a SimpleFrameBuffer for the 1st layer (render_scene in render.rs)");
        let mut layer2_framebuffer = 
            SimpleFrameBuffer::with_depth_buffer(display, &self.textures.layer_2_texture, &self.textures.layer_2_depth)
                .expect("Failed to create a SimpleFrameBuffer for the 2nd layer (render_scene in render.rs)");
        
        // 1. Cleaning
        frame.clear_color_and_depth((0.6, 0.6, 0.6, 1.0), 1.0);
        layer1_framebuffer.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0); // change to the BG color later
        layer2_framebuffer.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0); // change to the BG color later

        // 2. Render shadow map
        let mut close_shadow_framebuffer =
            SimpleFrameBuffer::depth_only(display, &self.textures.close_shadow_texture)
                .expect("Failed to create a SimpleFrameBuffer for the close shadow (render_scene in render.rs)");
        let mut far_shadow_framebuffer = 
            SimpleFrameBuffer::depth_only(display, &self.textures.far_shadow_texture)
                .expect("Failed to create a SimpleFrameBuffer for the far shadow (render_scene in render.rs)");
        close_shadow_framebuffer.clear_depth(1.0);
        far_shadow_framebuffer.clear_depth(1.0);
        //if self.update_shadowmap_timer.elapsed().as_secs_f32() > 0.33 {
            //self.shadow_camera = RenderShadowCamera::new(&self.camera, self.directional_light_dir);
            //self.update_shadowmap_timer = Instant::now();
        //}
        self.shadow_camera = RenderShadowCamera::new(&self.camera, self.directional_light_dir);

        object_render::shadow_render_objects(
            &mut close_shadow_framebuffer,
            &mut far_shadow_framebuffer,
            &mut self.instanced_positions,
            &self.shadow_camera,
            &self.objects,
            &self.display,
            &self.shadow_map_shader,
            &self.instanced_shadow_map_shader
        );


        // 3. Rendering objects
        object_render::render_objects(
            &mut layer1_framebuffer,
            &mut layer2_framebuffer,
            &mut self.instanced_positions,
            &self.textures.close_shadow_texture,
            &self.textures.far_shadow_texture,
            &self.objects,
            &self.camera,
            &self.shadow_camera,
            assets,
            &self.display,
            &self.lights,
            self.directional_light_dir,
            self.directional_light_strength
        );

        self.render_framebuffer_plane(&mut frame);

        egui_glium
            .paint(display, &mut frame);

        frame.finish().expect("Frame finish failed! - RenderManager(render_scene)");

        self.instanced_positions.clear();
        self.lights.clear();
    }

    pub fn render_framebuffer_plane(&self, frame: &mut Frame) {
        frame.draw(
            &self.framebuffer_vbo, 
            NoIndices(PrimitiveType::TrianglesList),
            &self.framebuffer_program,
            &uniform! {
                tex: &self.textures.layer_1_texture
            },
            &DrawParameters {
                blend: glium::Blend::alpha_blending(),
                ..Default::default()
            },
        ).expect("Failed to render layer 1 framebuffer - RenderManager (render_framebuffer_plane)");

        frame.draw(
            &self.framebuffer_vbo, 
            NoIndices(PrimitiveType::TrianglesList),
            &self.framebuffer_program,
            &uniform! {
                tex: &self.textures.layer_2_texture
            },
            &DrawParameters {
                blend: glium::Blend::alpha_blending(),
                ..Default::default()
            },
        ).expect("Failed to render layer 2 framebuffer - RenderManager (render_framebuffer_plane)");
    }

    pub fn calculate_resolution(window_size: (u32, u32)) -> (u32, u32) {
        let aspect_ratio = window_size.0 as f32 / window_size.1 as f32;
        let target_height = 480;
        let width = (target_height as f32 * aspect_ratio).ceil() as u32;

        (width, target_height)
    }

    pub fn resize(&mut self, window_size: (u32, u32)) {
        let display = &self.display;
        display.resize(window_size);

        self.window_size = window_size;
        let resolution = Self::calculate_resolution(window_size);

        self.textures = RenderManagerTextures {
            close_shadow_texture: 
                DepthTexture2d::empty(display, 2048, 2048)
                    .expect("close_shadow_texture creation error!"),
            far_shadow_texture:
                DepthTexture2d::empty(display, 2048, 2048)
                    .expect("far_shadow_texture creation error!"),
            layer_1_texture:
                Texture2d::empty(display, resolution.0, resolution.1)
                    .expect("layer_1_texture creation error!"),
            layer_2_texture:
                Texture2d::empty(display, resolution.0, resolution.1)
                    .expect("layer_2_texture creation error!"),
            layer_1_depth: 
                DepthTexture2d::empty(display, resolution.0, resolution.1)
                    .expect("layer_1_depth creation error!"),
            layer_2_depth:
                DepthTexture2d::empty(display, resolution.0, resolution.1)
                    .expect("layer_2_depth creation error!"),
        };
    }

    pub fn add_object(&mut self, object_id: u128, object_data: HashMap<String, Vec<RenderObjectData>>) {
        if self.objects.insert(object_id, object_data).is_some() {
            debugger::warn("Engine error! add_object() in render.rs, object is already inserted");
        }
    }

    pub fn get_object(&mut self, object_id: u128) -> Option<&mut HashMap<String, Vec<RenderObjectData>>> {
        self.objects.get_mut(&object_id)
    }

    pub fn remove_object(&mut self, object_id: &u128) {
        self.objects.remove(object_id);
    }

    pub fn set_camera_position(&mut self, position: Vec3) {
        self.camera.translation = position;
    }

    pub fn set_camera_rotation(&mut self, rotation: Vec3) {
        self.camera.rotation = rotation;
    }

    pub fn camera_position(&self) -> Vec3 {
        self.camera.translation
    }

    pub fn camera_rotation(&self) -> Vec3 {
        self.camera.rotation
    }

    pub fn camera_front(&self) -> Vec3 {
        self.camera.front()
    }

    pub fn camera_left(&self) -> Vec3 {
        self.camera.left()
    }

    pub fn add_light(&mut self, light: RenderPointLight) {
        self.lights.push(light);
    }

    pub fn set_light(&mut self, direction: Vec3, strength: f32) {
        self.directional_light_strength = strength;
        self.directional_light_dir = direction;
    }
}

pub(crate) struct RenderCamera {
    pub translation: Vec3,
    pub rotation: Vec3,
    pub y_fov_deg: f32,
    pub window_size: (u32, u32),
    pub render_distance: f32,
}

impl RenderCamera {
    fn camera_transformations(&self) -> Mat4 {
        Mat4::from_rotation_x(self.rotation.x.to_radians())
            * Mat4::from_rotation_y(-self.rotation.y.to_radians())
            * Mat4::from_rotation_z(self.rotation.z.to_radians())
            * Mat4::from_translation(self.translation)
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        let transformations = self.camera_transformations();
        let front_row = transformations.row(2);

        let front = Vec3 {
            x: -front_row.x,
            y: front_row.y,
            z: -front_row.z,
        }.normalize();

        Mat4::look_at_rh(self.translation, self.translation + front, Vec3::Y)
    }

    pub fn up(&self) -> Vec3 {
        let transformations = self.camera_transformations();
        transformations.row(1).xyz().normalize()
    }

    pub fn front(&self) -> Vec3 {
        let transformations = self.camera_transformations();
        let front_row = transformations.row(2);

        Vec3 {
            x: -front_row.x,
            y: front_row.y,
            z: -front_row.z,
        }.normalize()
    }

    pub fn left(&self) -> Vec3 {
        let transformations = self.camera_transformations();

        transformations.row(0).xyz().normalize()
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        let aspect_ratio = self.window_size.0 as f32 / self.window_size.1 as f32;
        Mat4::perspective_rh_gl(deg_to_rad(self.y_fov_deg), aspect_ratio, 0.1, self.render_distance)
    }

    pub fn get_projection_matrix_with_max_distance(&self, max_distance: f32) -> Mat4 {
        let aspect_ratio = self.window_size.0 as f32 / self.window_size.1 as f32;
        Mat4::perspective_rh_gl(deg_to_rad(self.y_fov_deg), aspect_ratio, 0.1, max_distance)
    }

    pub fn get_projection_matrix_with_min_distance(&self, min_distance: f32) -> Mat4 {
        let aspect_ratio = self.window_size.0 as f32 / self.window_size.1 as f32;
        Mat4::perspective_rh_gl(deg_to_rad(self.y_fov_deg), aspect_ratio, min_distance, self.render_distance)
    }

    pub fn new(window_size: (u32, u32)) -> RenderCamera {
        RenderCamera {
            translation: Vec3::ZERO,
            rotation: Vec3::ZERO,
            y_fov_deg: 59.0,
            window_size,
            render_distance: 600.0,
        }
    }
}

pub(crate) struct RenderManagerTextures {
    close_shadow_texture: DepthTexture2d,
    far_shadow_texture: DepthTexture2d,
    layer_1_texture: Texture2d,
    layer_1_depth: DepthTexture2d,
    layer_2_texture: Texture2d,
    layer_2_depth: DepthTexture2d,
}

#[derive(Debug)]
pub(crate) struct RenderObjectData {
    pub(crate) transform: Mat4,
    pub(crate) model_object_transform: Mat4,
    pub(crate) transparent: bool,
    pub(crate) uniforms: HashMap<String, RenderUniformValue>,
    pub(crate) texture_asset_id: Option<TextureAssetId>,
    pub(crate) shader: RenderShader,
    pub(crate) layer: RenderLayer,
    pub(crate) vbo: VertexBuffer<Vertex>,
    pub(crate) ibo: IndexBuffer<u32>,
    pub(crate) joint_matrices: [[[f32; 4]; 4]; 128],
    pub(crate) joint_inverse_bind_matrices: [[[f32; 4]; 4]; 128],
    pub(crate) instanced_master_name: Option<String>,
    pub(crate) cast_shadows: bool,
}

#[derive(Debug, Clone)]
pub enum RenderLayer {
    Layer1,
    Layer2,
}

#[derive(Debug, Clone)]
pub(crate) enum RenderUniformValue {
    Mat4(Mat4),
    Vec3(Vec3),
    Float(f32),
    Texture(TextureAssetId)
}

#[derive(Debug)]
pub(crate) enum RenderShader {
    NotLinked,
    Program(Program)
}

/// 0 is position, 1 is color, 2 is attenuation
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RenderPointLight(pub Vec3, pub Vec3, pub Vec2);

#[derive(Debug)]
pub(crate) struct RenderShadowCamera {
    pub(crate) close_shadow_proj: Mat4,
    pub(crate) close_shadow_view: Mat4,
    pub(crate) far_shadow_proj: Mat4,
    pub(crate) far_shadow_view: Mat4,
}

impl RenderShadowCamera {
    pub(crate) fn new(camera: &RenderCamera, light_dir: Vec3) -> RenderShadowCamera {
        let view = camera.get_view_matrix();
        let close_corners = Self::get_frustum_corners_world_space(
            camera.get_projection_matrix_with_max_distance(120.0), view);
        let close_corners_1 = Self::get_frustum_corners(
            camera.get_projection_matrix_with_max_distance(120.0)
        );
        let far_corners = Self::get_frustum_corners_world_space(
            camera.get_projection_matrix(), view);
        let far_corners_1 = Self::get_frustum_corners(
            camera.get_projection_matrix()
        );

        let close_shadow_proj = Self::shadow_proj(&close_corners_1);
        let far_shadow_proj = Self::shadow_proj(&far_corners_1);
        let close_shadow_view = Self::shadow_view(light_dir, &close_corners);
        let far_shadow_view = Self::shadow_view(light_dir, &far_corners);

        RenderShadowCamera {
            close_shadow_proj,
            close_shadow_view,
            far_shadow_proj,
            far_shadow_view,
        }
    }

    // from https://learnopengl.com/Guest-Articles/2021/CSM
    fn get_frustum_corners(proj: Mat4) -> Vec<Vec4> {
        let inv = proj.inverse();

        let mut corners: Vec<Vec4> = Vec::new();
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    let pt: Vec4 = inv * Vec4::new(
                        2.0 * x as f32 - 1.0,
                        2.0 * y as f32 - 1.0,
                        2.0 * z as f32 - 1.0,
                        1.0
                    );
                    corners.push(pt / pt.w);
                }
            }
        }

        corners
    }

    // from https://learnopengl.com/Guest-Articles/2021/CSM
    fn get_frustum_corners_world_space(proj: Mat4, view: Mat4) -> Vec<Vec4> {
        let inv = (proj * view).inverse();

        let mut corners: Vec<Vec4> = Vec::new();
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    let pt: Vec4 = inv * Vec4::new(
                        2.0 * x as f32 - 1.0,
                        2.0 * y as f32 - 1.0,
                        2.0 * z as f32 - 1.0,
                        1.0
                    );
                    corners.push(pt / pt.w);
                }
            }
        }

        corners
    }

    fn shadow_view(light_dir: Vec3, corners: &Vec<Vec4>) -> Mat4 {
        let mut center = Vec3::ZERO;
        for corner in corners {
            center += Vec3::new(corner.x, corner.y, corner.z);
        }
        center /= Vec3::new(corners.len() as f32, corners.len() as f32, corners.len() as f32);

        let light_right = Vec3::Y.cross(light_dir).normalize();
        let light_up = light_dir.cross(light_right);
        //Mat4::look_at_rh(center + Vec3::new(0.0, 30.0, 0.0) - light_dir, center, light_up)
        Mat4::look_at_rh(center + Vec3::Y - light_dir, center, light_up)
        //Mat4::look_at_rh(center - light_dir, center, light_up)
    }

    fn shadow_proj(corners: &Vec<Vec4>) -> Mat4 {
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        let mut min_z = f32::MAX;
        let mut max_z = f32::MIN;
        for corner in corners {
            if corner.x < min_x { min_x = corner.x }
            else if corner.x > max_x { max_x = corner.x }
            if corner.y < min_y { min_y = corner.y }
            else if corner.y > max_y { max_y = corner.y }
            if corner.z < min_z { min_z = corner.z }
            else if corner.z > max_z { max_z = corner.z }
        }

        Mat4::orthographic_rh_gl(min_x - 50.0, max_x + 50.0, min_y - 50.0, max_y + 50.0, min_z - 50.0, max_z + 50.0)
    }
}
