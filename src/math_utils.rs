use glam::Vec3;
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

    return Vec3::new(x, y, z);
}

pub fn rad_vec_to_deg(rad_vec: Vec3) -> Vec3 {
    let x = rad_to_deg(rad_vec.x);
    let y = rad_to_deg(rad_vec.y);
    let z = rad_to_deg(rad_vec.z);

    return Vec3::new(x, y, z);
}
