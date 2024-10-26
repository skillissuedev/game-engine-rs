use std::collections::HashMap;

use super::physics::{RenderColliderType, RenderRay};
use crate::math_utils::deg_to_rad;
use glam::{Mat4, Quat, Vec3, Vec4};
use glium::{
    framebuffer::SimpleFrameBuffer, glutin::surface::WindowSurface, implement_vertex,
    index::PrimitiveType, texture::DepthTexture2d, uniform, Display, Frame, IndexBuffer, Program,
    Surface, VertexBuffer,
};

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
    pub joints: [f32; 4],
    pub weights: [f32; 4],
}
implement_vertex!(Vertex, position, normal, tex_coords, joints, weights);

/* some consts to make code cleaner */
const ZERO_VEC3: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 0.0,
};
const DEFAULT_UP_VECTOR: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};
const DEFAULT_FRONT_VECTOR: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: -1.0,
};

#[derive(Debug)]
pub struct CameraLocation {
    pub position: Vec3,
    pub rotation: Vec3,
    pub fov: f32,
    pub front: Vec3,
    left: Vec3,
    up: Vec3,
}

const CUBE_VERTS_LIST: [Vertex; 8] = [
    Vertex {
        position: [1.0, 1.0, -1.0],
        normal: [0.0, 0.0, 0.0],
        joints: [0.0, 0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
        weights: [0.0, 0.0, 0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, -1.0],
        normal: [0.0, 0.0, 0.0],
        joints: [0.0, 0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
        weights: [0.0, 0.0, 0.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0],
        normal: [0.0, 0.0, 0.0],
        joints: [0.0, 0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
        weights: [0.0, 0.0, 0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 1.0],
        normal: [0.0, 0.0, 0.0],
        joints: [0.0, 0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
        weights: [0.0, 0.0, 0.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, -1.0],
        normal: [0.0, 0.0, 0.0],
        joints: [0.0, 0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
        weights: [0.0, 0.0, 0.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, -1.0],
        normal: [0.0, 0.0, 0.0],
        joints: [0.0, 0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
        weights: [0.0, 0.0, 0.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0],
        normal: [0.0, 0.0, 0.0],
        joints: [0.0, 0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
        weights: [0.0, 0.0, 0.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 1.0],
        normal: [0.0, 0.0, 0.0],
        joints: [0.0, 0.0, 0.0, 0.0],
        tex_coords: [0.0, 0.0],
        weights: [0.0, 0.0, 0.0, 0.0],
    },
];
const CUBE_INDICES_LIST: [u32; 36] = [
    1, 2, 0, 3, 6, 2, 7, 5, 6, 5, 0, 4, 6, 0, 2, 3, 5, 7, 1, 3, 2, 3, 7, 6, 7, 5, 4, 5, 1, 0, 6, 4,
    0, 3, 1, 5,
];

struct SunCamera {
    pub view: Mat4,
    pub proj: Mat4,
}

pub struct Cascades {
    closest: SunCamera,
    furthest: SunCamera,
    pub closest_view_proj: Mat4,
    pub furthest_view_proj: Mat4,
}

impl Cascades {
    pub fn new(fov: f32, aspect_ratio: f32, light_dir: Vec3, view: Mat4) -> Cascades {
        let closest = SunCamera::new(fov, aspect_ratio, light_dir, view, 0.0, Some(50.0), None);
        let furthest = SunCamera::new(fov, aspect_ratio, light_dir, view, 50.0, None, None); //Some(100.0));
        let closest_view_proj = closest.as_mat4();
        let furthest_view_proj = furthest.as_mat4();

        Cascades {
            closest,
            furthest,
            closest_view_proj,
            furthest_view_proj,
        }
    }
}

impl SunCamera {
    fn get_sun_camera_projection_matrix(corners: &CameraCorners) -> Mat4 {
        Mat4::orthographic_rh_gl(
            corners.min_x - 50.0,
            corners.max_x + 20.0,
            corners.min_y - 50.0,
            corners.max_y + 50.0,
            corners.min_z - 100.0,
            corners.max_z + 200.0,
        )
    }

    fn get_sun_camera_view_matrix(
        light_dir: Vec3,
        corners: &CameraCorners,
        additional_y: Option<f32>,
    ) -> Mat4 {
        //let direction = get_light_direction().normalize();
        let view_center = corners.get_center();
        let view_up = Vec3::new(0.0, 1.0, 0.0);

        //let main_camera_position = get_camera_position();
        let additional_y = if let Some(additional_y) = additional_y {
            Vec3::new(0.0, additional_y, 0.0)
        } else {
            Vec3::ZERO
        };

        let sun_camera_position =
            light_dir + view_center + Vec3::new(0.0, 20.0, 0.0) + additional_y; //Vec3::new(0.0, corners.max_y/* / 2.0*/, 0.0);

        let view_matrix = Mat4::look_at_rh(sun_camera_position, view_center, view_up);

        view_matrix
    }

    pub fn new(
        fov: f32,
        aspect_ratio: f32,
        light_dir: Vec3,
        view: Mat4,
        start_distance: f32,
        end_distance: Option<f32>,
        additional_y: Option<f32>,
    ) -> SunCamera {
        let corners = CameraCorners::new(fov, aspect_ratio, start_distance, end_distance, view);
        let proj = Self::get_sun_camera_projection_matrix(&corners);
        let view = Self::get_sun_camera_view_matrix(light_dir, &corners, additional_y);
        SunCamera { view, proj }
    }

    pub fn as_mat4(&self) -> Mat4 {
        self.proj * self.view
    }
}

struct CameraCorners {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    min_z: f32,
    max_z: f32,
}

impl CameraCorners {
    // https://learnopengl.com/Guest-Articles/2021/CSM
    fn get_camera_corners(proj: Mat4, view: Mat4) -> Vec<Vec3> {
        let proj_view = proj * view;
        let inv = proj_view.inverse();

        let mut frustum_corners = Vec::new();
        let mut vec3_frustum_corners = Vec::new();
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    let pt = inv
                        * Vec4::new(
                            2.0 * x as f32 - 1.0,
                            2.0 * y as f32 - 1.0,
                            2.0 * z as f32 - 1.0,
                            1.0,
                        );
                    frustum_corners.push(pt / pt.w);
                }
            }
        }

        for corner in &frustum_corners {
            vec3_frustum_corners.push(Vec3::new(corner.x, corner.y, corner.z));
        }

        vec3_frustum_corners
    }

    pub fn get_camera_proj(
        fov: f32,
        aspect_ratio: f32,
        mut start_distance: f32,
        end_distance: Option<f32>,
    ) -> Mat4 {
        start_distance += 0.01;
        let end_distance = match end_distance {
            Some(distance) => distance,
            None => 500.0,
        };
        Mat4::perspective_rh_gl(fov.to_radians(), aspect_ratio, start_distance, end_distance)
    }

    /*pub fn get_sun_eye(&self) -> Vec3 {
        Vec3::new(self.min_x, self.max_y, self.min_z)
    }*/

    pub fn new(
        fov: f32,
        aspect_ratio: f32,
        start_distance: f32,
        end_distance: Option<f32>,
        view: Mat4,
    ) -> CameraCorners {
        let corners = Self::get_camera_corners(
            Self::get_camera_proj(fov, aspect_ratio, start_distance, end_distance),
            view,
        );

        let mut min_x = 0.0;
        let mut min_y = 0.0;
        let mut min_z = 0.0;

        let mut max_x = 0.0;
        let mut max_y = 0.0;
        let mut max_z = 0.0;

        for corner in corners {
            if corner.x > max_x {
                max_x = corner.x;
            }
            if corner.x < min_x {
                min_x = corner.x;
            }

            if corner.y > max_y {
                max_y = corner.y;
            }
            if corner.y < min_y {
                min_y = corner.y;
            }

            if corner.z > max_z {
                max_z = corner.z;
            }
            if corner.z < min_z {
                min_z = corner.z;
            }
        }

        CameraCorners {
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
        }
    }

    pub fn get_center(&self) -> Vec3 {
        let center_x = (self.min_x + self.max_x) / 2.0;
        let center_y = (self.min_y + self.max_y) / 2.0;
        let center_z = (self.min_z + self.max_z) / 2.0;

        Vec3::new(center_x, center_y, center_z)
    }
}

pub struct ShadowTextures {
    pub closest: DepthTexture2d,
    pub furthest: DepthTexture2d,
}

impl ShadowTextures {
    pub fn new(
        display: &Display<WindowSurface>,
        closest_size: u32,
        furthest_size: u32,
    ) -> ShadowTextures {
        let closest =
            glium::texture::DepthTexture2d::empty(display, closest_size, closest_size).unwrap(); // 1st Cascade
        let furthest =
            glium::texture::DepthTexture2d::empty(display, furthest_size, furthest_size).unwrap(); // 2st Cascade

        ShadowTextures { closest, furthest }
    }
}

pub struct RenderManager {
    pub light_direction: Vec3,
    pub camera_location: CameraLocation,
    pub aspect_ratio: f32,
    pub display: Display<WindowSurface>,
    pub shadow_textures: ShadowTextures,
    pub target: Option<Frame>,
    pub render_rays: Vec<RenderRay>,
    pub render_colliders: Vec<RenderColliderType>,
    pub cascades: Cascades,
    pub ray_shader: Program,
    pub collider_cuboid_shader: Program,
    pub collider_cuboid_vertex_buffer: VertexBuffer<Vertex>,
    pub collider_cuboid_index_buffer: IndexBuffer<u32>,
    pub instanced_positions: HashMap<String, Vec<Mat4>>,
}

impl RenderManager {
    pub fn new(display: Display<WindowSurface>) -> Self {
        let ray_shader = Program::from_source(
            &display,
            include_str!("../assets/ray_shader.vert"),
            include_str!("../assets/ray_shader.frag"),
            None,
        )
        .unwrap();
        let collider_cuboid_shader = Program::from_source(
            &display,
            include_str!("../assets/collider_shader.vert"),
            include_str!("../assets/collider_shader.frag"),
            None,
        )
        .unwrap();
        let collider_cuboid_vertex_buffer = VertexBuffer::new(&display, &CUBE_VERTS_LIST).unwrap();
        let collider_cuboid_index_buffer =
            IndexBuffer::new(&display, PrimitiveType::TrianglesList, &CUBE_INDICES_LIST).unwrap();
        let shadow_textures = ShadowTextures::new(&display, 4096, 4096);

        Self {
            light_direction: Vec3::new(-1.0, 0.0, 0.0),
            camera_location: CameraLocation {
                position: Vec3::ZERO,
                rotation: Vec3::ZERO,
                fov: 47.0,
                front: DEFAULT_FRONT_VECTOR,
                left: Vec3::ZERO,
                up: DEFAULT_UP_VECTOR,
            },
            aspect_ratio: 1.0,
            shadow_textures,
            display,
            target: None,
            render_rays: Vec::new(),
            render_colliders: Vec::new(),
            cascades: Cascades::new(47.0, 1.0, Vec3::new(-1.0, 0.0, 0.0), Mat4::IDENTITY),
            ray_shader,
            collider_cuboid_shader,
            collider_cuboid_vertex_buffer,
            collider_cuboid_index_buffer,
            instanced_positions: HashMap::new(),
        }
    }

    pub fn set_camera_position(&mut self, pos: Vec3) {
        self.camera_location.position = pos;
    }

    pub fn set_camera_rotation(&mut self, rot: Vec3) {
        self.camera_location.rotation = rot;
    }

    pub fn set_camera_fov(&mut self, fov: f32) {
        self.camera_location.fov = fov;
    }

    pub fn set_light_direction(&mut self, dir: Vec3) {
        self.light_direction = dir;
    }

    pub fn get_light_direction(&self) -> Vec3 {
        self.light_direction
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        let mut translation = self.camera_location.position;
        translation.y = -translation.y;
        Mat4::from_rotation_x(self.camera_location.rotation.x.to_radians())
            * Mat4::from_rotation_y(self.camera_location.rotation.y.to_radians())
            * Mat4::from_rotation_z(self.camera_location.rotation.z.to_radians())
            * Mat4::from_translation(translation)
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh_gl(self.camera_location.fov.to_radians(), self.aspect_ratio, 0.001, 500.0)
    }

    fn update_camera_vectors(&mut self) {
        let mut translation = self.camera_location.position;
        translation.y = -translation.y;
        let transformations = Mat4::from_rotation_x(self.camera_location.rotation.x.to_radians())
            * Mat4::from_rotation_y(self.camera_location.rotation.y.to_radians())
            * Mat4::from_rotation_z(self.camera_location.rotation.z.to_radians())
            * Mat4::from_translation(translation);
        let left_row = transformations.row(0);
        let up_row = transformations.row(1);
        let front_row = transformations.row(2);

        let front = Vec3 {
            x: front_row.x,
            y: front_row.y,
            z: front_row.z,
        };
        self.camera_location.front = front.normalize();
        self.camera_location.left = Vec3 {
            x: -left_row.x,
            y: -left_row.y,
            z: -left_row.z,
        }.normalize();

        self.camera_location.up = Vec3 {
            x: up_row.x,
            y: up_row.y,
            z: up_row.z,
        }.normalize();
    }

    pub fn get_camera_position(&self) -> Vec3 {
        self.camera_location.position
    }

    pub fn get_camera_front(&mut self) -> Vec3 {
        let front = self.camera_location.front;
        Vec3 {
            x: front.x,
            y: -front.y,
            z: front.z,
        }
    }

    pub fn get_camera_left(&self) -> Vec3 {
        self.camera_location.left
    }

    pub fn get_camera_rotation(&self) -> Vec3 {
        self.camera_location.rotation
    }

    pub fn get_camera_fov(&self) -> f32 {
        self.camera_location.fov
    }

    pub fn draw(&mut self) {
        //target.clear_color_and_depth((0.6, 0.91, 0.88, 1.0), 1.0);
        let mut target = self.display.draw();
        target.clear_color_srgb_and_depth((0.7, 0.7, 0.9, 1.0), 1.0);
        self.target = Some(target);
        let display = &self.display;
        let shadow_textures = &self.shadow_textures;

        let mut closest_shadow_fbo =
            SimpleFrameBuffer::depth_only(display, &shadow_textures.closest).unwrap();
        closest_shadow_fbo.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
        closest_shadow_fbo.clear_depth(1.0);

        let mut furthest_shadow_fbo =
            SimpleFrameBuffer::depth_only(display, &shadow_textures.furthest).unwrap();
        furthest_shadow_fbo.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
        furthest_shadow_fbo.clear_depth(1.0);

        let view = self.get_view_matrix();
        self.update_camera_vectors();
        self.cascades = Cascades::new(
            self.get_camera_fov(),
            self.aspect_ratio,
            self.get_light_direction(),
            view,
        );
    }

    /// Call only after drawing everything.
    pub fn debug_draw(&mut self) {
        let proj = self.get_projection_matrix().to_cols_array_2d();
        let view = self.get_view_matrix().to_cols_array_2d();

        self.render_colliders.clone().iter().for_each(|collider| {
            let mvp_and_sensor = self.calculate_collider_mvp_and_sensor(&collider);
            let uniforms = uniform! {
                mvp: mvp_and_sensor.0,
                sensor: mvp_and_sensor.1
            };

            let draw_params = glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                blend: glium::draw_parameters::Blend::alpha_blending(),
                ..Default::default()
            };

            let vert_buffer = &self.collider_cuboid_vertex_buffer;
            let index_buffer = &self.collider_cuboid_index_buffer;
            let shader = &self.collider_cuboid_shader;

            self.target
                .as_mut()
                .unwrap() // drawing solid semi-transparent cuboid
                .draw(vert_buffer, index_buffer, shader, &uniforms, &draw_params)
                .unwrap();
        });
        self.render_colliders.clear();

        self.render_rays.iter().for_each(|ray| {
            let uniforms = uniform! {
                proj: proj,
                view: view,
            };

            let draw_params = glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                blend: glium::draw_parameters::Blend::alpha_blending(),
                ..Default::default()
            };

            let origin = ray.origin;
            let dir = origin + ray.direction;

            let verts_list: [Vertex; 9] = [
                Vertex {
                    position: [origin.x - 0.15, origin.y + 0.15, origin.z],
                    normal: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 0.0],
                    joints: [0.0, 0.0, 0.0, 0.0],
                    weights: [0.0, 0.0, 0.0, 0.0],
                },
                Vertex {
                    position: [origin.x + 0.15, origin.y + 0.15, origin.z],
                    normal: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 0.0],
                    joints: [0.0, 0.0, 0.0, 0.0],
                    weights: [0.0, 0.0, 0.0, 0.0],
                },
                Vertex {
                    position: [origin.x - 0.15, origin.y - 0.15, origin.z],
                    normal: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 0.0],
                    joints: [0.0, 0.0, 0.0, 0.0],
                    weights: [0.0, 0.0, 0.0, 0.0],
                },
                Vertex {
                    position: [origin.x + 0.15, origin.y - 0.15, origin.z],
                    normal: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 0.0],
                    joints: [0.0, 0.0, 0.0, 0.0],
                    weights: [0.0, 0.0, 0.0, 0.0],
                },
                Vertex {
                    position: [dir.x - 0.15, dir.y + 0.15, dir.z],
                    normal: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 0.0],
                    joints: [0.0, 0.0, 0.0, 0.0],
                    weights: [0.0, 0.0, 0.0, 0.0],
                },
                Vertex {
                    position: [dir.x + 0.15, dir.y + 0.15, dir.z],
                    normal: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 0.0],
                    joints: [0.0, 0.0, 0.0, 0.0],
                    weights: [0.0, 0.0, 0.0, 0.0],
                },
                Vertex {
                    position: [dir.x - 0.15, dir.y - 0.15, dir.z],
                    normal: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 0.0],
                    joints: [0.0, 0.0, 0.0, 0.0],
                    weights: [0.0, 0.0, 0.0, 0.0],
                },
                Vertex {
                    position: [dir.x + 0.15, dir.y - 0.15, dir.z],
                    normal: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 0.0],
                    joints: [0.0, 0.0, 0.0, 0.0],
                    weights: [0.0, 0.0, 0.0, 0.0],
                },
                Vertex {
                    position: [dir.x, dir.y, dir.z + 0.15],
                    normal: [0.0, 0.0, 0.0],
                    tex_coords: [0.0, 0.0],
                    joints: [0.0, 0.0, 0.0, 0.0],
                    weights: [0.0, 0.0, 0.0, 0.0],
                },
            ];

            let indices: [u32; 48] = [
                0, 1, 2, 1, 3, 2, 4, 5, 6, 6, 7, 5, 0, 2, 4, 0, 2, 6, 1, 3, 5, 1, 3, 7, 0, 1, 4, 0,
                1, 5, 2, 3, 6, 2, 3, 7, 4, 5, 8, 4, 6, 8, 5, 7, 8, 6, 7, 8,
            ];

            let vert_buffer = VertexBuffer::new(&self.display, &verts_list)
                .expect("failed to create vertex buffer while debug rendering rays");
            let index_buffer =
                IndexBuffer::new(&self.display, PrimitiveType::TriangleFan, &indices)
                    .expect("failed to create index buffer while debug rendering rays");

            let shader = &self.ray_shader;

            self.target
                .as_mut()
                .unwrap() // drawing solid semi-transparent cuboid
                .draw(
                    &vert_buffer,
                    &index_buffer,
                    &shader,
                    &uniforms,
                    &draw_params,
                )
                .unwrap();
        });
        self.render_rays.clear();
    }

    pub fn finish_render(&mut self) {
        let target = self.target.take();
        if let Some(target) = target {
            target.finish().unwrap();
        }
    }

    pub fn calculate_collider_mvp_and_sensor(
        &self,
        collider: &RenderColliderType,
    ) -> ([[f32; 4]; 4], bool) {
        let view = self.get_view_matrix();
        let proj = self.get_projection_matrix();

        let rot_quat;
        let position_vector;

        match collider {
            RenderColliderType::Ball(pos, rot, radius, sensor) => {
                match rot {
                    Some(rot) => {
                        rot_quat = Quat::from_euler(
                            glam::EulerRot::XYZ,
                            deg_to_rad(rot.x),
                            deg_to_rad(rot.y),
                            deg_to_rad(rot.z),
                        )
                    }
                    None => rot_quat = Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
                }
                match pos {
                    Some(pos) => position_vector = pos,
                    None => position_vector = &Vec3::ZERO,
                }

                let scale = Vec3::new(*radius, *radius, *radius);
                let transform =
                    Mat4::from_scale_rotation_translation(scale, rot_quat, *position_vector);

                ((proj * view * transform).to_cols_array_2d(), *sensor)
            }
            RenderColliderType::Cuboid(pos, rot, half_x, half_y, half_z, sensor) => {
                match rot {
                    Some(rot) => {
                        rot_quat = Quat::from_euler(
                            glam::EulerRot::XYZ,
                            deg_to_rad(rot.x),
                            deg_to_rad(rot.y),
                            deg_to_rad(rot.z),
                        )
                    }
                    None => rot_quat = Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
                }
                match pos {
                    Some(pos) => position_vector = pos,
                    None => position_vector = &Vec3::ZERO,
                }

                let scale = Vec3::new(*half_x + 0.001, *half_y + 0.001, *half_z + 0.001);
                let transform =
                    Mat4::from_scale_rotation_translation(scale, rot_quat, *position_vector);

                ((proj * view * transform).to_cols_array_2d(), *sensor)
            }
            RenderColliderType::Capsule(pos, rot, radius, height, sensor) => {
                match rot {
                    Some(rot) => {
                        rot_quat = Quat::from_euler(
                            glam::EulerRot::XYZ,
                            deg_to_rad(rot.x),
                            deg_to_rad(rot.y),
                            deg_to_rad(rot.z),
                        )
                    }
                    None => rot_quat = Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
                }
                match pos {
                    Some(pos) => position_vector = pos,
                    None => position_vector = &Vec3::ZERO,
                }

                let scale = Vec3::new(*radius, *height, *radius);
                let transform =
                    Mat4::from_scale_rotation_translation(scale, rot_quat, *position_vector);

                ((proj * view * transform).to_cols_array_2d(), *sensor)
            }
            RenderColliderType::Cylinder(pos, rot, radius, height, sensor) => {
                match rot {
                    Some(rot) => {
                        rot_quat = Quat::from_euler(
                            glam::EulerRot::XYZ,
                            deg_to_rad(rot.x),
                            deg_to_rad(rot.y),
                            deg_to_rad(rot.z),
                        )
                    }
                    None => rot_quat = Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
                }
                match pos {
                    Some(pos) => position_vector = pos,
                    None => position_vector = &Vec3::ZERO,
                }

                let scale = Vec3::new(*radius, *height, *radius);
                let transform =
                    Mat4::from_scale_rotation_translation(scale, rot_quat, *position_vector);

                ((proj * view * transform).to_cols_array_2d(), *sensor)
            }
        }
    }

    pub fn add_collider_to_draw(&mut self, col: RenderColliderType) {
        self.render_colliders.push(col);
    }

    pub fn add_ray_to_draw(&mut self, ray: RenderRay) {
        self.render_rays.push(ray);
    }

    pub fn update(&mut self) {
        self.instanced_positions.clear();
    }

    pub fn add_instance_position(&mut self, instance: &str, position: Mat4) {
        match self.instanced_positions.get_mut(instance) {
            Some(positions) => positions.push(position),
            None => {
                self.instanced_positions
                    .insert(instance.into(), vec![position]);
            }
        }
    }

    pub fn add_instance_positions_vec(&mut self, instance: &str, positions: &Vec<Mat4>) {
        match self.instanced_positions.get_mut(instance) {
            Some(instanced_positions) => instanced_positions.extend(positions.iter()),
            None => {
                self.instanced_positions
                    .insert(instance.into(), positions.to_owned());
            }
        }
    }

    pub fn get_instance_positions(&self, instance: &str) -> Option<&Vec<Mat4>> {
        match self.instanced_positions.get(instance) {
            Some(positions) => Some(positions),
            None => None,
        }
    }

    pub fn prepare_for_normal_render(&mut self) {}

    pub fn closest_shadow_fbo(&self) -> SimpleFrameBuffer<'_> {
        let mut closest_shadow_fbo =
            SimpleFrameBuffer::depth_only(&self.display, &self.shadow_textures.closest).unwrap();
        closest_shadow_fbo.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
        closest_shadow_fbo.clear_depth(1.0);
        closest_shadow_fbo
    }

    pub fn furthest_shadow_fbo(&self) -> SimpleFrameBuffer<'_> {
        let mut furthest_shadow_fbo =
            SimpleFrameBuffer::depth_only(&self.display, &self.shadow_textures.furthest).unwrap();
        furthest_shadow_fbo.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
        furthest_shadow_fbo.clear_depth(1.0);
        furthest_shadow_fbo
    }
}

pub enum CurrentCascade {
    Closest,
    Furthest,
}
