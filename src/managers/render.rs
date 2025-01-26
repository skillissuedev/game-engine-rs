use std::collections::HashMap;

use glam::{Mat4, Vec3};
use glium::{framebuffer::SimpleFrameBuffer, glutin::surface::WindowSurface, implement_vertex, texture::DepthTexture2d, Display, IndexBuffer, Program, Surface, Texture2d, VertexBuffer};

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

pub(crate) struct RenderManager {
    pub(crate) objects: HashMap<u128, HashMap<usize, Vec<RenderObjectData>>>,
    pub(crate) window_size: (u32, u32),
    pub(crate) textures: RenderManagerTextures,
    pub(crate) camera: RenderCamera,
    pub(crate) pixelation_amount: f32,
    pub(crate) display: Display<WindowSurface>,
}

impl RenderManager {
    pub fn new(display: Display<WindowSurface>) -> RenderManager {
        let pixelation_amount = 2.0;
        let textures = RenderManagerTextures {
            close_shadow_texture: 
                DepthTexture2d::empty(&display, (1280.0 / pixelation_amount) as u32, (720.0 / pixelation_amount) as u32)
                    .expect("close_shadow_texture creation error!"),
            far_shadow_texture:
                DepthTexture2d::empty(&display, (1280.0 / pixelation_amount) as u32, (720.0 / pixelation_amount) as u32)
                    .expect("far_shadow_texture creation error!"),
            layer_1_texture:
                Texture2d::empty(&display, (1280.0 / pixelation_amount) as u32, (720.0 / pixelation_amount) as u32)
                    .expect("layer_1_texture creation error!"),
            layer_2_texture:
                Texture2d::empty(&display, (1280.0 / pixelation_amount) as u32, (720.0 / pixelation_amount) as u32)
                    .expect("layer_2_texture creation error!"),
            layer_1_depth: 
                DepthTexture2d::empty(&display, (1280.0 / pixelation_amount) as u32, (720.0 / pixelation_amount) as u32)
                    .expect("layer_1_depth creation error!"),
            layer_2_depth:
                DepthTexture2d::empty(&display, (1280.0 / pixelation_amount) as u32, (720.0 / pixelation_amount) as u32)
                    .expect("layer_2_depth creation error!"),
        };
        let camera = RenderCamera::new((1280, 720));

        RenderManager {
            objects: HashMap::new(),
            window_size: (1280, 720),
            textures,
            camera,
            pixelation_amount,
            display,
        }
    }

    pub fn render_scene(&mut self, assets: &AssetManager) {
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
        frame.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        layer1_framebuffer.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0); // change to the BG color later
        layer2_framebuffer.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0); // change to the BG color later

        // 2. Render shadow map - to do
        // 3. Rendering objects
        object_render::render_objects(
            &mut layer1_framebuffer,
            &mut layer2_framebuffer,
            &self.textures.close_shadow_texture,
            &self.textures.far_shadow_texture,
            &self.objects,
            &self.camera,
            assets,
            &self.display
        );

        frame.finish().expect("Swap buffer failed! - RenderManager(render_scene)");
    }

    pub fn resize(&mut self, window_size: (u32, u32)) {
        let display = &self.display;

        self.window_size = window_size;
        let pixelation_amount = self.pixelation_amount;
        let x = window_size.0 as f32;
        let y = window_size.1 as f32;

        self.textures = RenderManagerTextures {
            close_shadow_texture: 
                DepthTexture2d::empty(display, (x / pixelation_amount) as u32, (y / pixelation_amount) as u32)
                    .expect("close_shadow_texture creation error!"),
            far_shadow_texture:
                DepthTexture2d::empty(display, (x / pixelation_amount) as u32, (y / pixelation_amount) as u32)
                    .expect("far_shadow_texture creation error!"),
            layer_1_texture:
                Texture2d::empty(display, (x / pixelation_amount) as u32, (y / pixelation_amount) as u32)
                    .expect("layer_1_texture creation error!"),
            layer_2_texture:
                Texture2d::empty(display, (x / pixelation_amount) as u32, (y / pixelation_amount) as u32)
                    .expect("layer_2_texture creation error!"),
            layer_1_depth: 
                DepthTexture2d::empty(display, (x / pixelation_amount) as u32, (y / pixelation_amount) as u32)
                    .expect("layer_1_depth creation error!"),
            layer_2_depth:
                DepthTexture2d::empty(display, (x / pixelation_amount) as u32, (y / pixelation_amount) as u32)
                    .expect("layer_2_depth creation error!"),
        };
    }

    pub fn add_object(&mut self, object_id: u128, object_data: HashMap<usize, Vec<RenderObjectData>>) {
        if self.objects.insert(object_id, object_data).is_some() {
            debugger::warn("Engine error! add_object() in render.rs, object is already inserted");
        }
    }

    pub fn get_object(&mut self, object_id: u128) -> Option<&mut HashMap<usize, Vec<RenderObjectData>>> {
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
}

pub(crate) struct RenderCamera {
    pub translation: Vec3,
    pub rotation: Vec3,
    pub y_fov_deg: f32,
    pub window_size: (u32, u32),
    pub render_distance: f32
}

impl RenderCamera {
    fn camera_transformtations(&self) -> Mat4 {
        Mat4::from_rotation_x(self.rotation.x.to_radians())
            * Mat4::from_rotation_y(self.rotation.y.to_radians())
            * Mat4::from_rotation_z(self.rotation.z.to_radians())
            * Mat4::from_translation(self.translation)
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        let transformations = self.camera_transformtations();
        let up_row = transformations.row(1);
        let front_row = transformations.row(2);

        let front = Vec3 {
            x: front_row.x,
            y: front_row.y,
            z: front_row.z,
        }.normalize();
        dbg!(front);

        let up = Vec3 {
            x: up_row.x,
            y: up_row.y,
            z: up_row.z,
        }.normalize();

        Mat4::look_at_lh(self.translation, self.translation + front, up)
    }

    pub fn front(&self) -> Vec3 {
        let transformations = self.camera_transformtations();
        let front_row = transformations.row(2);

        Vec3 {
            x: front_row.x,
            y: front_row.y,
            z: front_row.z,
        }.normalize()
    }

    pub fn left(&self) -> Vec3 {
        let transformations = Mat4::from_rotation_x(self.rotation.x.to_radians())
            * Mat4::from_rotation_y(self.rotation.y.to_radians())
            * Mat4::from_rotation_z(self.rotation.z.to_radians())
            * Mat4::from_translation(self.translation);
        let left_row = transformations.row(0);

        Vec3 {
            x: left_row.x,
            y: left_row.y,
            z: left_row.z,
        }.normalize()
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        let aspect_ratio = self.window_size.0 as f32 / self.window_size.1 as f32;
        Mat4::perspective_rh_gl(deg_to_rad(self.y_fov_deg), aspect_ratio, 0.001, self.render_distance)
    }

    pub fn new(window_size: (u32, u32)) -> RenderCamera {
        RenderCamera {
            translation: Vec3::ZERO,
            rotation: Vec3::ZERO,
            y_fov_deg: 59.0,
            window_size,
            render_distance: 500.0,
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
    pub(crate) joint_matrices: [[[f32; 4]; 4]; 512],
    pub(crate) joint_inverse_bind_matrices: [[[f32; 4]; 4]; 512],
}

impl RenderObjectData {
    pub fn set_transform(&mut self, transform_matrix: Mat4) {
        self.transform = transform_matrix;
    }

    pub fn add_uniform(&mut self, uniform_name: String, uniform_value: RenderUniformValue) {
        self.uniforms.insert(uniform_name, uniform_value);
    }

    pub fn remove_uniform(&mut self, uniform_name: String) {
        self.uniforms.remove(&uniform_name);
    }
}

#[derive(Debug, Clone)]
pub enum RenderLayer {
    Layer1,
    Layer2,
}

#[derive(Debug)]
pub(crate) enum RenderUniformValue {
    Mat4(Mat4),
    Vec3(Vec3),
    Float(f32),
    Texture(TextureAssetId)
}

#[derive(Debug)]
pub(crate) enum RenderShader {
    NotLinked(ShaderAsset),
    Program(Program)
}
