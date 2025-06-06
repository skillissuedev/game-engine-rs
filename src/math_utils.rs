use glam::{Mat3, Mat4, Quat, Vec3};
use noise::{NoiseFn, Perlin};
use std::f32::consts::PI;

pub fn rad_to_deg(rad: f32) -> f32 {
    rad * 180.0 / PI
}

pub fn deg_to_rad(deg: f32) -> f32 {
    deg * PI / 180.0
}

pub fn deg_vec_to_rad(deg_vec: Vec3) -> Vec3 {
    let x = deg_to_rad(deg_vec.x);
    let y = deg_to_rad(deg_vec.y);
    let z = deg_to_rad(deg_vec.z);

    Vec3::new(x, y, z)
}

pub fn rad_vec_to_deg(rad_vec: Vec3) -> Vec3 {
    let x = rad_to_deg(rad_vec.x);
    let y = rad_to_deg(rad_vec.y);
    let z = rad_to_deg(rad_vec.z);

    Vec3::new(x, y, z)
}

pub fn rotate_vector(direction: Vec3, rotation: Vec3) -> Vec3 {
    let global_rotation = deg_vec_to_rad(rotation);
    let rotation_mat = Mat4::from_euler(glam::EulerRot::XYZ, global_rotation.x, global_rotation.y, global_rotation.z);

    let direction = rotation_mat.transform_vector3(direction);

    direction
}

//https://www.opengl-tutorial.org/intermediate-tutorials/tutorial-17-quaternions/
pub fn look_at_rotation(from: Vec3, to: Vec3) -> Vec3 {
    let matrix = Mat4::look_at_rh(from, to, Vec3::Y).inverse();

    let euler = matrix.to_scale_rotation_translation().1
        .to_euler(glam::EulerRot::XYZ);

    let degrees = rad_vec_to_deg(Vec3::new(euler.0, euler.1, euler.2));
    degrees
}

pub struct PerlinNoise {
    noise: Perlin,
}

impl PerlinNoise {
    pub fn new(seed: u32) -> PerlinNoise {
        let noise = Perlin::new(seed);
        PerlinNoise {
            noise
        }
    }

    pub fn get_x(&self, coordinate: f32) -> f32 {
        self.noise.get([coordinate as f64, 1.0]) as f32
    }

    pub fn get_y(&self, coordinate: f32) -> f32 {
        self.noise.get([coordinate as f64, 2.0]) as f32
    }
}
