use std::sync::Mutex;

use ez_al::EzAl;
use glam::Vec3;
use once_cell::sync::Lazy;

pub fn set_listener_position(al: &EzAl, position: Vec3) {
    ez_al::set_listener_position(&al, position.into());
}

pub fn set_listener_orientation(al: &EzAl, at: Vec3) {
    ez_al::set_listener_orientation(&al, at.into(), [0.0, 1.0, 0.0]);
    //ez_al::set_listener_orientation(at.into(), [0.0, 1.0, 0.0]);
}

pub fn set_listener_transform(al: &EzAl, position: Vec3, at: Vec3) {
    set_listener_position(al, position);
    set_listener_orientation(al, at);
}
