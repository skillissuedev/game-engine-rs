use glium::{implement_vertex, Frame, Surface, VertexBuffer, IndexBuffer, index::PrimitiveType, Display, Program, uniform};
use glam::{Mat4, Vec3, Quat};
use crate::math_utils::deg_to_rad;

use super::physics::RenderColliderType;

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
        COLLIDER_CUBOID_INDEX_BUFFER = 
            Some(IndexBuffer::new(display, PrimitiveType::TrianglesList, &CUBE_INDICES_LIST).unwrap());
        COLLIDER_CUBOID_SHADER = Some(Program::from_source(
            display,
            include_str!("../assets/collider_shader.vert"),
            include_str!("../assets/collider_shader.frag"),
            None,
        ).unwrap());
    }
}

/// Call only after drawing everything.
pub fn debug_draw(target: &mut Frame) {
    let colliders = unsafe { &mut RENDER_COLLIDERS };


    colliders.iter().for_each(|collider| {
        let uniforms = uniform! {
            mvp: calculate_collider_mvp(collider),
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
            RENDER_COLLIDERS.clear();
        }
    });
}

pub fn add_collider_to_draw(col: RenderColliderType) {
    unsafe {
        RENDER_COLLIDERS.push(col);
    }
}

pub fn draw(target: &mut Frame) {
    target.clear_color_srgb_and_depth((0.1, 0.1, 0.1, 1.0), 1.0);
    update_camera_vectors();
}

/* some consts to make code cleaner */
const ZERO_VEC3: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
const DEFAULT_UP_VECTOR: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
const DEFAULT_FRONT_VECTOR: Vec3 = Vec3 { x: 0.0, y: 0.0, z: -1.0 };

pub static mut LIGHT_POSITION: Vec3 = ZERO_VEC3;

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

pub fn set_light_position(pos: Vec3) {
    unsafe {
        LIGHT_POSITION = pos;
    }
}

pub fn get_light_position() -> Vec3 {
    unsafe {
        LIGHT_POSITION
    }
}

pub fn get_view_matrix() -> Mat4 {
    unsafe {
        Mat4::look_at_lh(
            CAMERA_LOCATION.position,
            -(CAMERA_LOCATION.position + CAMERA_LOCATION.front),
            DEFAULT_UP_VECTOR,
        )
    }
}

pub fn get_projection_matrix() -> Mat4 {
    unsafe {
        //ultraviolet::projection::perspective_gl(CAMERA_LOCATION.fov, ASPECT_RATIO, 0.001, 560.0)
        Mat4::perspective_rh_gl(CAMERA_LOCATION.fov, ASPECT_RATIO, 0.001, 560.0)
    }
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

const CUBE_VERTS_LIST: [Vertex; 8] = 
    [Vertex { position: [1.0, 1.0, -1.0], normal: [0.0, 0.0, 0.0], joints: [0.0, 0.0, 0.0, 0.0], tex_coords: [0.0, 0.0], weights: [0.0, 0.0, 0.0, 0.0] },
    Vertex { position: [1.0, -1.0, -1.0], normal: [0.0, 0.0, 0.0], joints: [0.0, 0.0, 0.0, 0.0], tex_coords: [0.0, 0.0], weights: [0.0, 0.0, 0.0, 0.0] },
    Vertex { position: [1.0, 1.0, 1.0], normal: [0.0, 0.0, 0.0], joints: [0.0, 0.0, 0.0, 0.0], tex_coords: [0.0, 0.0], weights: [0.0, 0.0, 0.0, 0.0] },
    Vertex { position: [1.0, -1.0, 1.0], normal: [0.0, 0.0, 0.0], joints: [0.0, 0.0, 0.0, 0.0], tex_coords: [0.0, 0.0], weights: [0.0, 0.0, 0.0, 0.0] },
    Vertex { position: [-1.0, 1.0, -1.0], normal: [0.0, 0.0, 0.0], joints: [0.0, 0.0, 0.0, 0.0], tex_coords: [0.0, 0.0], weights: [0.0, 0.0, 0.0, 0.0] },
    Vertex { position: [-1.0, -1.0, -1.0], normal: [0.0, 0.0, 0.0], joints: [0.0, 0.0, 0.0, 0.0], tex_coords: [0.0, 0.0], weights: [0.0, 0.0, 0.0, 0.0] },
    Vertex { position: [-1.0, 1.0, 1.0], normal: [0.0, 0.0, 0.0], joints: [0.0, 0.0, 0.0, 0.0], tex_coords: [0.0, 0.0], weights: [0.0, 0.0, 0.0, 0.0] },
    Vertex { position: [-1.0, -1.0, 1.0], normal: [0.0, 0.0, 0.0], joints: [0.0, 0.0, 0.0, 0.0], tex_coords: [0.0, 0.0], weights: [0.0, 0.0, 0.0, 0.0] }]; 
const CUBE_INDICES_LIST: [u32; 36] = [1, 2, 0, 3, 6, 2, 7, 5, 6, 5, 0, 4, 6, 0, 2, 3, 5, 7, 1, 3, 2, 3, 7, 6, 7, 5, 4, 5, 1, 0, 6, 4, 0, 3, 1, 5];
static mut COLLIDER_CUBOID_VERTEX_BUFFER: Option<VertexBuffer<Vertex>> = None;
static mut COLLIDER_CUBOID_INDEX_BUFFER: Option<IndexBuffer<u32>> = None;
static mut RENDER_COLLIDERS: Vec<RenderColliderType> = vec![];
static mut COLLIDER_CUBOID_SHADER: Option<Program> = None;

pub fn calculate_collider_mvp(collider: &RenderColliderType) -> [[f32; 4]; 4] {
    let view = get_view_matrix();
    let proj = get_projection_matrix();

    let rot_quat;
    let position_vector;

    match collider {
        RenderColliderType::Ball(pos, rot, radius) => {
            match rot {
                Some(rot) => rot_quat = Quat::from_euler(glam::EulerRot::XYZ, deg_to_rad(rot.x), deg_to_rad(rot.y), deg_to_rad(rot.z)),
                None => rot_quat = Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
            }
            match pos {
                Some(pos) => position_vector = pos,
                None => position_vector = &Vec3::ZERO,
            }

            let scale = Vec3::new(*radius * 2.0, *radius * 2.0, *radius * 2.0);
            let transform = Mat4::from_scale_rotation_translation(scale, rot_quat, *position_vector);

            return (proj * view * transform).to_cols_array_2d();
        },
        RenderColliderType::Cuboid(pos, rot, half_x, half_y, half_z) => {
            match rot {
                Some(rot) => rot_quat = Quat::from_euler(glam::EulerRot::XYZ, deg_to_rad(rot.x), deg_to_rad(rot.y), deg_to_rad(rot.z)),
                None => rot_quat = Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
            }
            match pos {
                Some(pos) => position_vector = pos,
                None => position_vector = &Vec3::ZERO,
            }

            let scale = Vec3::new(*half_x * 2.0 + 0.001, *half_y * 2.0 + 0.001, *half_z * 2.0 + 0.001);
            let transform = Mat4::from_scale_rotation_translation(scale, rot_quat, *position_vector);

            return (proj * view * transform).to_cols_array_2d();
        }
        RenderColliderType::Capsule(pos, rot, radius, height) => {
            match rot {
                Some(rot) => rot_quat = Quat::from_euler(glam::EulerRot::XYZ, deg_to_rad(rot.x), deg_to_rad(rot.y), deg_to_rad(rot.z)),
                None => rot_quat = Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
            }
            match pos {
                Some(pos) => position_vector = pos,
                None => position_vector = &Vec3::ZERO,
            }

            let scale = Vec3::new(*radius * 2.0, *height * 2.0, *radius * 2.0);
            let transform = Mat4::from_scale_rotation_translation(scale, rot_quat, *position_vector);

            return (proj * view * transform).to_cols_array_2d();
        }
        RenderColliderType::Cylinder(pos, rot, radius, height) => {
            match rot {
                Some(rot) => rot_quat = Quat::from_euler(glam::EulerRot::XYZ, deg_to_rad(rot.x), deg_to_rad(rot.y), deg_to_rad(rot.z)),
                None => rot_quat = Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
            }
            match pos {
                Some(pos) => position_vector = pos,
                None => position_vector = &Vec3::ZERO,
            }

            let scale = Vec3::new(*radius * 2.0, *height * 2.0, *radius * 2.0);
            let transform = Mat4::from_scale_rotation_translation(scale, rot_quat, *position_vector);

            return (proj * view * transform).to_cols_array_2d();
        },
    }
}

//indices = IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0, 1, 3, 0, 3, 2, 0, 4, 6, 0, 6, 2, 6, 4, 5, 5, 7, 6, 5, 1, 3, 5, 7, 3]);
