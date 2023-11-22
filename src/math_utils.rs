use std::f32::consts::PI;

pub fn rad_to_deg(rad: f32) -> f32 {
    rad * 180.0 / PI
}

pub fn deg_to_rad(deg: f32) -> f32 {
    deg * PI / 180.0
}
