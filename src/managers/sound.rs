use ez_al::SoundError;
use glam::Vec3;

pub fn init() -> Result<(), SoundError> {
    ez_al::init()
}

pub fn set_listener_position(position: Vec3) {
    ez_al::set_listener_position(position.into())
}

pub fn set_listener_orientation(at: Vec3) {
    ez_al::set_listener_orientation(at.into(), [0.0, 1.0, 0.0]);
}

pub fn set_listener_transform(position: Vec3, at: Vec3) {
    set_listener_position(position);
    set_listener_orientation(at);
}
