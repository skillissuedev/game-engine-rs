use glium::{implement_vertex, Frame, Surface};
use ultraviolet::{Mat4, Vec3};

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub joints: [f32; 4],
    pub weights: [f32; 4],
}
implement_vertex!(Vertex, position, tex_coords, joints, weights);

pub fn draw(target: &mut Frame) {
    target.clear_color_srgb_and_depth((0.1, 0.1, 0.1, 1.0), 1.0);
    update_camera_vectors();
}

/* some consts to make code cleaner */
const ZERO_VEC3: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
const DEFAULT_UP_VECTOR: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
const DEFAULT_FRONT_VECTOR: Vec3 = Vec3 { x: 0.0, y: 0.0, z: -1.0 };

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

pub fn get_view_matrix() -> Mat4 {
    unsafe {
        Mat4::look_at(
            CAMERA_LOCATION.position,
            CAMERA_LOCATION.position + CAMERA_LOCATION.front,
            DEFAULT_UP_VECTOR,
        )
    }
}

pub fn get_projection_matrix() -> Mat4 {
    unsafe {
        ultraviolet::projection::perspective_gl(CAMERA_LOCATION.fov, ASPECT_RATIO, 0.001, 560.0)
    }
}

fn update_camera_vectors() {
    unsafe {
        let front = Vec3 {
            x: CAMERA_LOCATION.rotation.x.to_radians().cos()
                * CAMERA_LOCATION.rotation.y.to_radians().cos(),
            y: CAMERA_LOCATION.rotation.y.to_radians().sin(),
            z: CAMERA_LOCATION.rotation.y.to_radians().sin()
                * CAMERA_LOCATION.rotation.x.to_radians().cos(),
        };
        CAMERA_LOCATION.front = front.normalized();
        // Also re-calculate the Right and Up vector
        CAMERA_LOCATION.right = CAMERA_LOCATION.front.cross(DEFAULT_UP_VECTOR).normalized(); // Normalize the vectors, because their length gets closer to 0 the more you look up or down which results in slower movement.
        CAMERA_LOCATION.up = CAMERA_LOCATION
            .right
            .cross(CAMERA_LOCATION.front)
            .normalized();
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
