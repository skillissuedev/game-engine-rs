use ultraviolet::Vec3;

use super::component::Component;

struct CameraPosition {
    pub position: Vec3,
    pub rotation: Vec3,
    pub fov: f32,
}
