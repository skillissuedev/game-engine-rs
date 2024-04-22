use glium::glutin::event::VirtualKeyCode;

use crate::{framework::{get_debug_mode, set_debug_mode, DebugMode}, managers::{input::{self, InputEventType}, systems::add_system}, systems::test_system::TestSystem};

pub fn start() {
    input::new_bind("debug_toggle", vec![InputEventType::Key(VirtualKeyCode::Grave)]);

    add_system(Box::new(TestSystem::new()));
}

pub fn update() {
    if input::is_bind_pressed("debug_toggle") {
        match get_debug_mode() {
            DebugMode::None => set_debug_mode(DebugMode::ShowFps),
            _ => {
                set_debug_mode(DebugMode::None);
            }
        }
    }
}

//pub fn render() {}
