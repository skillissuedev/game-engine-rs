use crate::math_utils::deg_to_rad;
use glam::{Mat4, Quat, Vec3, Vec4};
use glium::{
    framebuffer::SimpleFrameBuffer, implement_vertex, index::PrimitiveType,
    texture::DepthTexture2d, uniform, Display, Frame, IndexBuffer, Program, Surface, VertexBuffer,
};

use super::{
    physics::{RenderColliderType, RenderRay},
    systems,
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

pub fn init(display: &Display) {
    unsafe {
        COLLIDER_CUBOID_VERTEX_BUFFER = Some(VertexBuffer::new(display, &CUBE_VERTS_LIST).unwrap());
        COLLIDER_CUBOID_INDEX_BUFFER = Some(
            IndexBuffer::new(display, PrimitiveType::TrianglesList, &CUBE_INDICES_LIST).unwrap(),
        );
        COLLIDER_CUBOID_SHADER = Some(
            Program::from_source(
                display,
                include_str!("../assets/collider_shader.vert"),
                include_str!("../assets/collider_shader.frag"),
                None,
            )
            .unwrap(),
        );
        RAY_SHADER = Some(
            Program::from_source(
                display,
                include_str!("../assets/ray_shader.vert"),
                include_str!("../assets/ray_shader.frag"),
                None,
            )
            .unwrap(),
        );
    }
}

/// Call only after drawing everything.
pub fn debug_draw(display: &Display, target: &mut Frame) {
    let proj = get_projection_matrix().to_cols_array_2d();
    let view = get_view_matrix().to_cols_array_2d();

    unsafe {
        RENDER_RAYS.iter().for_each(|ray| {
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

            //dbg!(ray);
            //dbg!(origin);
            //dbg!(dir);

            let vert_buffer = VertexBuffer::new(display, &verts_list)
                .expect("failed to create vertex buffer while debug rendering rays");
            let index_buffer = IndexBuffer::new(display, PrimitiveType::TriangleFan, &indices)
                .expect("failed to create index buffer while debug rendering rays");

            let shader = RAY_SHADER.take().unwrap();

            target // drawing solid semi-transparent cuboid
                .draw(
                    &vert_buffer,
                    &index_buffer,
                    &shader,
                    &uniforms,
                    &draw_params,
                )
                .unwrap();

            RAY_SHADER = Some(shader);
        });
        RENDER_RAYS.clear();
    }

    let colliders = unsafe { &mut RENDER_COLLIDERS };
    colliders.iter().for_each(|collider| {
        let mvp_and_sensor = calculate_collider_mvp_and_sensor(collider);
        let uniforms = uniform! {
            mvp: mvp_and_sensor.0,
            sensor: mvp_and_sensor.1
        };

        unsafe {
            let draw_params = glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                blend: glium::draw_parameters::Blend::alpha_blending(),
                ..Default::default()
            };

            let vert_buffer = COLLIDER_CUBOID_VERTEX_BUFFER.take().unwrap();
            let index_buffer = COLLIDER_CUBOID_INDEX_BUFFER.take().unwrap();
            let shader = COLLIDER_CUBOID_SHADER.take().unwrap();

            target // drawing solid semi-transparent cuboid
                .draw(
                    &vert_buffer,
                    &index_buffer,
                    &shader,
                    &uniforms,
                    &draw_params,
                )
                .unwrap();

            COLLIDER_CUBOID_SHADER = Some(shader);
            COLLIDER_CUBOID_VERTEX_BUFFER = Some(vert_buffer);
            COLLIDER_CUBOID_INDEX_BUFFER = Some(index_buffer);
        }
    });

    unsafe { RENDER_COLLIDERS.clear() }
}

pub fn add_collider_to_draw(col: RenderColliderType) {
    unsafe {
        RENDER_COLLIDERS.push(col);
    }
}

pub fn add_ray_to_draw(ray: RenderRay) {
    unsafe {
        RENDER_RAYS.push(ray);
    }
}

pub fn draw(display: &Display, target: &mut Frame, shadow_textures: &ShadowTextures) {
    target.clear_color_and_depth((0.4, 0.4, 0.4, 1.0), 1.0);

    let mut closest_shadow_fbo = SimpleFrameBuffer::depth_only(display, &shadow_textures.closest).unwrap();
    closest_shadow_fbo.clear_color(1.0, 1.0, 1.0, 1.0);
    closest_shadow_fbo.clear_depth(1.0);

    let mut furthest_shadow_fbo = SimpleFrameBuffer::depth_only(display, &shadow_textures.furthest).unwrap();
    furthest_shadow_fbo.clear_color(1.0, 1.0, 1.0, 1.0);
    furthest_shadow_fbo.clear_depth(1.0);

    let view = get_view_matrix();
    let cascades = Cascades::new(view);
    systems::shadow_render(&cascades.closest.as_mat4(), display, &mut closest_shadow_fbo);
    systems::shadow_render(&cascades.furthest.as_mat4(), display, &mut furthest_shadow_fbo);

    update_camera_vectors();

    systems::render(&display, target, &cascades, shadow_textures);
}

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

pub static mut LIGHT_POSITION: Vec3 = Vec3 {
    x: -1.0,
    y: 0.0,
    z: 0.0,
};

pub static mut CAMERA_LOCATION: CameraLocation = CameraLocation {
    position: ZERO_VEC3,
    rotation: ZERO_VEC3,
    fov: 90.0,
    front: DEFAULT_FRONT_VECTOR,
    right: ZERO_VEC3,
    up: DEFAULT_UP_VECTOR,
};
pub static mut ASPECT_RATIO: f32 = 0.0;

pub fn set_camera_position(pos: Vec3) {
    unsafe {
        CAMERA_LOCATION.position = pos;
    }
}

pub fn set_camera_rotation(rot: Vec3) {
    unsafe {
        CAMERA_LOCATION.rotation = rot;
    }
}

pub fn set_camera_fov(fov: f32) {
    unsafe {
        CAMERA_LOCATION.fov = fov;
    }
}

pub fn set_light_direction(pos: Vec3) {
    unsafe {
        LIGHT_POSITION = pos;
    }
}

pub fn get_light_direction() -> Vec3 {
    unsafe { LIGHT_POSITION }
}

pub fn get_view_matrix() -> Mat4 {
    unsafe {
        let mut camera_position = CAMERA_LOCATION.position;
        camera_position.x = -camera_position.x;
        Mat4::look_at_lh(
            camera_position,
            camera_position - CAMERA_LOCATION.front,
            DEFAULT_UP_VECTOR,
        )
    }
}

pub fn get_projection_matrix() -> Mat4 {
    unsafe { Mat4::perspective_rh_gl(CAMERA_LOCATION.fov, ASPECT_RATIO, 0.001, 500.0) }
}

fn update_camera_vectors() {
    unsafe {
        let front = Vec3 {
            x: CAMERA_LOCATION.rotation.y.to_radians().sin()
                * CAMERA_LOCATION.rotation.x.to_radians().cos(),
            y: CAMERA_LOCATION.rotation.y.to_radians().sin(),
            z: CAMERA_LOCATION.rotation.x.to_radians().cos()
                * CAMERA_LOCATION.rotation.y.to_radians().cos(),
        };
        CAMERA_LOCATION.front = front.normalize();
        // Also re-calculate the Right and Up vector
        CAMERA_LOCATION.right = CAMERA_LOCATION.front.cross(DEFAULT_UP_VECTOR).normalize(); // Normalize the vectors, because their length gets closer to 0 the more you look up or down which results in slower movement.
        CAMERA_LOCATION.up = CAMERA_LOCATION
            .right
            .cross(CAMERA_LOCATION.front)
            .normalize();
    }
}

pub fn get_camera_position() -> Vec3 {
    unsafe { CAMERA_LOCATION.position }
}

pub fn get_camera_front() -> Vec3 {
    unsafe { CAMERA_LOCATION.front }
}

pub fn get_camera_rotation() -> Vec3 {
    unsafe { CAMERA_LOCATION.rotation }
}

pub fn get_camera_fov() -> f32 {
    unsafe { CAMERA_LOCATION.fov }
}

#[derive(Debug)]
pub struct CameraLocation {
    pub position: Vec3,
    pub rotation: Vec3,
    pub fov: f32,
    pub front: Vec3,
    right: Vec3,
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
static mut COLLIDER_CUBOID_VERTEX_BUFFER: Option<VertexBuffer<Vertex>> = None;
static mut COLLIDER_CUBOID_INDEX_BUFFER: Option<IndexBuffer<u32>> = None;
static mut RENDER_COLLIDERS: Vec<RenderColliderType> = vec![];
static mut RENDER_RAYS: Vec<RenderRay> = vec![];
static mut RAY_SHADER: Option<Program> = None;
static mut COLLIDER_CUBOID_SHADER: Option<Program> = None;

pub fn calculate_collider_mvp_and_sensor(collider: &RenderColliderType) -> ([[f32; 4]; 4], bool) {
    let view = get_view_matrix();
    let proj = get_projection_matrix();

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

struct SunCamera {
    pub view: Mat4,
    pub proj: Mat4,
}

pub struct Cascades {
    closest: SunCamera, 
    furthest: SunCamera, 
    pub closest_view_proj: Mat4, 
    pub furthest_view_proj: Mat4
}

impl Cascades {
    pub fn new(view: Mat4) -> Cascades {
        let closest = SunCamera::new(view, 0.0, Some(50.0));
        let furthest = SunCamera::new(view, 50.0, None);
        let closest_view_proj = closest.as_mat4();
        let furthest_view_proj = furthest.as_mat4();

        Cascades { closest, furthest, closest_view_proj, furthest_view_proj } 
    }
}

impl SunCamera {
    fn get_sun_camera_projection_matrix(corners: &CameraCorners) -> Mat4 {
        //dbg!(corners.min_x, corners.max_x);
        //dbg!(corners.min_y, corners.max_y);
        //dbg!(corners.min_z, corners.max_z);

        Mat4::orthographic_rh_gl(
            corners.min_x,
            corners.max_x,
            corners.min_y,
            corners.max_y,
            corners.min_z,
            //corners.min_z + corners.max_z / 2.0,
            corners.max_z,
        )
    }

    fn get_sun_camera_view_matrix(corners: &CameraCorners) -> Mat4 {
        //let direction = get_light_direction().normalize();
        let view_center = corners.get_center();
        let view_up = Vec3::new(0.0, 1.0, 0.0);

        //let main_camera_position = get_camera_position();
        let sun_camera_position = get_light_direction() + view_center + Vec3::new(0.0, corners.max_y / 2.0, 0.0);

        let view_matrix = Mat4::look_at_rh(sun_camera_position, view_center, view_up);

        view_matrix
    }

    pub fn new(view: Mat4, start_distance: f32, end_distance: Option<f32>) -> SunCamera {
        let corners = CameraCorners::new(start_distance, end_distance, view);
        let proj = Self::get_sun_camera_projection_matrix(&corners);
        let view = Self::get_sun_camera_view_matrix(&corners);
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

        //dbg!(&vec3_frustum_corners);
        vec3_frustum_corners
    }

    pub fn get_camera_proj(mut start_distance: f32, end_distance: Option<f32>) -> Mat4 {
        start_distance += 0.01;
        let end_distance = match end_distance {
            Some(distance) => distance,
            None => 500.0,
        };
        //dbg!(start_distance, end_distance);
        unsafe { Mat4::perspective_rh_gl(CAMERA_LOCATION.fov, ASPECT_RATIO, start_distance, end_distance) }
    }

    pub fn get_sun_eye(&self) -> Vec3 {
        Vec3::new(self.min_x, self.max_y, self.min_z)
    }

    pub fn new(start_distance: f32, end_distance: Option<f32>, view: Mat4) -> CameraCorners {
        let corners = Self::get_camera_corners(Self::get_camera_proj(start_distance, end_distance), view);
        //dbg!(&corners);

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
    pub furthest: DepthTexture2d
}

impl ShadowTextures {
    pub fn new(display: &Display, closest_size: u32, furthest_size: u32) -> ShadowTextures {
        let closest = glium::texture::DepthTexture2d::empty(display, closest_size, closest_size).unwrap(); // 1st Cascade
        let furthest = glium::texture::DepthTexture2d::empty(display, furthest_size, furthest_size).unwrap(); // 2st Cascade
        
        ShadowTextures { closest, furthest }
    }
}
