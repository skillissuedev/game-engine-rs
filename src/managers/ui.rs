conrod_winit::v023_conversion_fns!();

#[macro_export]
macro_rules! convert_key {
    ($keycode:expr) => {{
        match $keycode {
            glium::glutin::event::VirtualKeyCode::Key0 => conrod_core::input::keyboard::Key::D0,
            glium::glutin::event::VirtualKeyCode::Key1 => conrod_core::input::keyboard::Key::D1,
            glium::glutin::event::VirtualKeyCode::Key2 => conrod_core::input::keyboard::Key::D2,
            glium::glutin::event::VirtualKeyCode::Key3 => conrod_core::input::keyboard::Key::D3,
            glium::glutin::event::VirtualKeyCode::Key4 => conrod_core::input::keyboard::Key::D4,
            glium::glutin::event::VirtualKeyCode::Key5 => conrod_core::input::keyboard::Key::D5,
            glium::glutin::event::VirtualKeyCode::Key6 => conrod_core::input::keyboard::Key::D6,
            glium::glutin::event::VirtualKeyCode::Key7 => conrod_core::input::keyboard::Key::D7,
            glium::glutin::event::VirtualKeyCode::Key8 => conrod_core::input::keyboard::Key::D8,
            glium::glutin::event::VirtualKeyCode::Key9 => conrod_core::input::keyboard::Key::D9,
            glium::glutin::event::VirtualKeyCode::A => conrod_core::input::keyboard::Key::A,
            glium::glutin::event::VirtualKeyCode::B => conrod_core::input::keyboard::Key::B,
            glium::glutin::event::VirtualKeyCode::C => conrod_core::input::keyboard::Key::C,
            glium::glutin::event::VirtualKeyCode::D => conrod_core::input::keyboard::Key::D,
            glium::glutin::event::VirtualKeyCode::E => conrod_core::input::keyboard::Key::E,
            glium::glutin::event::VirtualKeyCode::F => conrod_core::input::keyboard::Key::F,
            glium::glutin::event::VirtualKeyCode::G => conrod_core::input::keyboard::Key::G,
            glium::glutin::event::VirtualKeyCode::H => conrod_core::input::keyboard::Key::H,
            glium::glutin::event::VirtualKeyCode::I => conrod_core::input::keyboard::Key::I,
            glium::glutin::event::VirtualKeyCode::J => conrod_core::input::keyboard::Key::J,
            glium::glutin::event::VirtualKeyCode::K => conrod_core::input::keyboard::Key::K,
            glium::glutin::event::VirtualKeyCode::L => conrod_core::input::keyboard::Key::L,
            glium::glutin::event::VirtualKeyCode::M => conrod_core::input::keyboard::Key::M,
            glium::glutin::event::VirtualKeyCode::N => conrod_core::input::keyboard::Key::N,
            glium::glutin::event::VirtualKeyCode::O => conrod_core::input::keyboard::Key::O,
            glium::glutin::event::VirtualKeyCode::P => conrod_core::input::keyboard::Key::P,
            glium::glutin::event::VirtualKeyCode::Q => conrod_core::input::keyboard::Key::Q,
            glium::glutin::event::VirtualKeyCode::R => conrod_core::input::keyboard::Key::R,
            glium::glutin::event::VirtualKeyCode::S => conrod_core::input::keyboard::Key::S,
            glium::glutin::event::VirtualKeyCode::T => conrod_core::input::keyboard::Key::T,
            glium::glutin::event::VirtualKeyCode::U => conrod_core::input::keyboard::Key::U,
            glium::glutin::event::VirtualKeyCode::V => conrod_core::input::keyboard::Key::V,
            glium::glutin::event::VirtualKeyCode::W => conrod_core::input::keyboard::Key::W,
            glium::glutin::event::VirtualKeyCode::X => conrod_core::input::keyboard::Key::X,
            glium::glutin::event::VirtualKeyCode::Y => conrod_core::input::keyboard::Key::Y,
            glium::glutin::event::VirtualKeyCode::Z => conrod_core::input::keyboard::Key::Z,
            glium::glutin::event::VirtualKeyCode::Apostrophe => {
                conrod_core::input::keyboard::Key::Unknown
            }
            glium::glutin::event::VirtualKeyCode::Backslash => {
                conrod_core::input::keyboard::Key::Backslash
            }
            glium::glutin::event::VirtualKeyCode::Back => {
                conrod_core::input::keyboard::Key::Backspace
            }
            // K::CapsLock => Key::CapsLock,
            glium::glutin::event::VirtualKeyCode::Delete => {
                conrod_core::input::keyboard::Key::Delete
            }
            glium::glutin::event::VirtualKeyCode::Comma => conrod_core::input::keyboard::Key::Comma,
            glium::glutin::event::VirtualKeyCode::Down => conrod_core::input::keyboard::Key::Down,
            glium::glutin::event::VirtualKeyCode::End => conrod_core::input::keyboard::Key::End,
            glium::glutin::event::VirtualKeyCode::Return => {
                conrod_core::input::keyboard::Key::Return
            }
            glium::glutin::event::VirtualKeyCode::Equals => {
                conrod_core::input::keyboard::Key::Equals
            }
            glium::glutin::event::VirtualKeyCode::Escape => {
                conrod_core::input::keyboard::Key::Escape
            }
            glium::glutin::event::VirtualKeyCode::F1 => conrod_core::input::keyboard::Key::F1,
            glium::glutin::event::VirtualKeyCode::F2 => conrod_core::input::keyboard::Key::F2,
            glium::glutin::event::VirtualKeyCode::F3 => conrod_core::input::keyboard::Key::F3,
            glium::glutin::event::VirtualKeyCode::F4 => conrod_core::input::keyboard::Key::F4,
            glium::glutin::event::VirtualKeyCode::F5 => conrod_core::input::keyboard::Key::F5,
            glium::glutin::event::VirtualKeyCode::F6 => conrod_core::input::keyboard::Key::F6,
            glium::glutin::event::VirtualKeyCode::F7 => conrod_core::input::keyboard::Key::F7,
            glium::glutin::event::VirtualKeyCode::F8 => conrod_core::input::keyboard::Key::F8,
            glium::glutin::event::VirtualKeyCode::F9 => conrod_core::input::keyboard::Key::F9,
            glium::glutin::event::VirtualKeyCode::F10 => conrod_core::input::keyboard::Key::F10,
            glium::glutin::event::VirtualKeyCode::F11 => conrod_core::input::keyboard::Key::F11,
            glium::glutin::event::VirtualKeyCode::F12 => conrod_core::input::keyboard::Key::F12,
            glium::glutin::event::VirtualKeyCode::F13 => conrod_core::input::keyboard::Key::F13,
            glium::glutin::event::VirtualKeyCode::F14 => conrod_core::input::keyboard::Key::F14,
            glium::glutin::event::VirtualKeyCode::F15 => conrod_core::input::keyboard::Key::F15,
            glium::glutin::event::VirtualKeyCode::Numpad0 => {
                conrod_core::input::keyboard::Key::NumPad0
            }
            glium::glutin::event::VirtualKeyCode::Numpad1 => {
                conrod_core::input::keyboard::Key::NumPad1
            }
            glium::glutin::event::VirtualKeyCode::Numpad2 => {
                conrod_core::input::keyboard::Key::NumPad2
            }
            glium::glutin::event::VirtualKeyCode::Numpad3 => {
                conrod_core::input::keyboard::Key::NumPad3
            }
            glium::glutin::event::VirtualKeyCode::Numpad4 => {
                conrod_core::input::keyboard::Key::NumPad4
            }
            glium::glutin::event::VirtualKeyCode::Numpad5 => {
                conrod_core::input::keyboard::Key::NumPad5
            }
            glium::glutin::event::VirtualKeyCode::Numpad6 => {
                conrod_core::input::keyboard::Key::NumPad6
            }
            glium::glutin::event::VirtualKeyCode::Numpad7 => {
                conrod_core::input::keyboard::Key::NumPad7
            }
            glium::glutin::event::VirtualKeyCode::Numpad8 => {
                conrod_core::input::keyboard::Key::NumPad8
            }
            glium::glutin::event::VirtualKeyCode::Numpad9 => {
                conrod_core::input::keyboard::Key::NumPad9
            }
            glium::glutin::event::VirtualKeyCode::NumpadComma => {
                conrod_core::input::keyboard::Key::NumPadDecimal
            }
            glium::glutin::event::VirtualKeyCode::NumpadEnter => {
                conrod_core::input::keyboard::Key::NumPadEnter
            }
            glium::glutin::event::VirtualKeyCode::NumpadEquals => {
                conrod_core::input::keyboard::Key::NumPadEquals
            }
            glium::glutin::event::VirtualKeyCode::LShift => {
                conrod_core::input::keyboard::Key::LShift
            }
            glium::glutin::event::VirtualKeyCode::LControl => {
                conrod_core::input::keyboard::Key::LCtrl
            }
            glium::glutin::event::VirtualKeyCode::LAlt => conrod_core::input::keyboard::Key::LAlt,
            glium::glutin::event::VirtualKeyCode::RShift => {
                conrod_core::input::keyboard::Key::RShift
            }
            glium::glutin::event::VirtualKeyCode::RControl => {
                conrod_core::input::keyboard::Key::RCtrl
            }
            glium::glutin::event::VirtualKeyCode::RAlt => conrod_core::input::keyboard::Key::RAlt,
            glium::glutin::event::VirtualKeyCode::Home => conrod_core::input::keyboard::Key::Home,
            glium::glutin::event::VirtualKeyCode::Insert => {
                conrod_core::input::keyboard::Key::Insert
            }
            glium::glutin::event::VirtualKeyCode::Left => conrod_core::input::keyboard::Key::Left,
            glium::glutin::event::VirtualKeyCode::LBracket => {
                conrod_core::input::keyboard::Key::LeftBracket
            }
            glium::glutin::event::VirtualKeyCode::Minus => conrod_core::input::keyboard::Key::Minus,
            glium::glutin::event::VirtualKeyCode::Numlock => {
                conrod_core::input::keyboard::Key::NumLockClear
            }
            glium::glutin::event::VirtualKeyCode::PageDown => {
                conrod_core::input::keyboard::Key::PageDown
            }
            glium::glutin::event::VirtualKeyCode::PageUp => {
                conrod_core::input::keyboard::Key::PageUp
            }
            glium::glutin::event::VirtualKeyCode::Pause => conrod_core::input::keyboard::Key::Pause,
            glium::glutin::event::VirtualKeyCode::Period => {
                conrod_core::input::keyboard::Key::Period
            }
            glium::glutin::event::VirtualKeyCode::Right => conrod_core::input::keyboard::Key::Right,
            glium::glutin::event::VirtualKeyCode::RBracket => {
                conrod_core::input::keyboard::Key::RightBracket
            }
            glium::glutin::event::VirtualKeyCode::Semicolon => {
                conrod_core::input::keyboard::Key::Semicolon
            }
            glium::glutin::event::VirtualKeyCode::Slash => conrod_core::input::keyboard::Key::Slash,
            glium::glutin::event::VirtualKeyCode::Space => conrod_core::input::keyboard::Key::Space,
            glium::glutin::event::VirtualKeyCode::Tab => conrod_core::input::keyboard::Key::Tab,
            glium::glutin::event::VirtualKeyCode::Up => conrod_core::input::keyboard::Key::Up,
            _ => conrod_core::input::keyboard::Key::Unknown,
        }
    }};
}
