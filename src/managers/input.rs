use super::debugger;
use glam::Vec2;
//use glium::glutin::event::{DeviceEvent, ElementState, MouseButton, VirtualKeyCode, WindowEvent};
use once_cell::sync::Lazy;
use winit::{event::{DeviceEvent, ElementState, MouseButton, WindowEvent}, keyboard::{KeyCode, PhysicalKey}};
use std::collections::HashMap;

static mut BINDS: Lazy<HashMap<String, Vec<InputEventType>>> = Lazy::new(|| HashMap::new());
static mut DOWN_EVENTS: Vec<InputEventType> = vec![];
static mut JUST_PRESSED_EVENTS: Vec<InputEventType> = vec![];
static mut UP_EVENTS: Vec<InputEventType> = vec![];

static mut MOUSE_POSITION: Vec2 = Vec2::new(0.0, 0.0);
static mut MOUSE_DELTA: Vec2 = Vec2::new(0.0, 0.0);
static mut WINDOW_RESOLUTION: Vec2 = Vec2::new(0.0, 0.0);
static mut IS_MOUSE_LOCKED: bool = false;

pub fn new_bind(name: &str, input_events: Vec<InputEventType>) {
    unsafe {
        match BINDS.get_mut(name) {
            Some(bind_evs) => {
                debugger::warn("Binds '{}' already exist, adding new key in list");
                input_events.iter().for_each(|ev| bind_evs.push(*ev));
            }
            None => {
                BINDS.insert(name.into(), input_events);
            }
        }
    }
}

pub fn get_bind_keys(name: String) -> Option<Vec<InputEventType>> {
    unsafe {
        match BINDS.get(&name) {
            Some(bind) => Some(bind.to_owned()),
            None => None,
        }
    }
}

pub fn reg_device_event(event: &DeviceEvent) {
    unsafe {
        match event {
            DeviceEvent::MouseMotion { delta } => MOUSE_DELTA = Vec2::new(delta.0 as f32, delta.1 as f32),
            _ => ()
        }
    }
}

pub fn reg_event(event: &WindowEvent) {
    unsafe {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                //input,
                is_synthetic: _,
                event: input
            } => {
                BINDS.iter().for_each(|bind_events| {
                    for bind_ev in bind_events.1 {
                        match bind_ev {
                            InputEventType::Key(keycode) => {
                                if keycode == &input.physical_key {
                                    let input_ev_type = InputEventType::Key(*keycode);
                                    match input.state {
                                        ElementState::Pressed => {
                                            if DOWN_EVENTS.contains(&input_ev_type) == false {
                                                DOWN_EVENTS.push(input_ev_type);
                                                JUST_PRESSED_EVENTS.push(input_ev_type);
                                            }
                                        }
                                        ElementState::Released => {
                                            if UP_EVENTS.contains(&input_ev_type) == false {
                                                UP_EVENTS.push(input_ev_type);
                                            }
                                            DOWN_EVENTS.iter().enumerate().for_each(
                                                |(idx, value)| {
                                                    if value == &input_ev_type {
                                                        DOWN_EVENTS.remove(idx);
                                                    }
                                                },
                                            );
                                        }
                                    };
                                }
                            }
                            InputEventType::Mouse(_) => (),
                        }
                    }
                });
            }
            #[allow(deprecated)]
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                let input_ev_type = InputEventType::Mouse(*button);
                match state {
                    ElementState::Pressed => {
                        if DOWN_EVENTS.contains(&input_ev_type) == false {
                            DOWN_EVENTS.push(input_ev_type);
                            JUST_PRESSED_EVENTS.push(input_ev_type);
                        }
                    }
                    ElementState::Released => {
                        if UP_EVENTS.contains(&input_ev_type) == false {
                            UP_EVENTS.push(input_ev_type);
                        }
                        DOWN_EVENTS.iter().enumerate().for_each(|(idx, value)| {
                            if value == &input_ev_type {
                                DOWN_EVENTS.remove(idx);
                            }
                        });
                    }
                }
            }
            WindowEvent::CursorMoved { device_id: _, position } => {
                MOUSE_POSITION = Vec2::new(position.x as f32, position.y as f32);
            }
            WindowEvent::Resized(new_size) => {
                WINDOW_RESOLUTION = Vec2::new(new_size.width as f32, new_size.height as f32)
            }
            _ => (),
        }
    }
}

pub fn update() {
    unsafe {
        JUST_PRESSED_EVENTS.clear();
        UP_EVENTS.clear();
        MOUSE_DELTA = Vec2::ZERO;
    }
}

pub fn is_bind_pressed(requested_bind_name: &str) -> bool {
    let events = unsafe { &JUST_PRESSED_EVENTS };
    let binds = unsafe { &BINDS };

    for event in events {
        for (bind_name, bind) in binds.iter() {
            if requested_bind_name == bind_name {
                for input_event_type in bind {
                    if &input_event_type == &event {
                        return true;
                    }
                }
            }
        }
    }

    false
}

pub fn is_bind_down(requested_bind_name: &str) -> bool {
    let events = unsafe { &DOWN_EVENTS };
    let binds = unsafe { &BINDS };

    for event in events {
        for (bind_name, bind) in binds.iter() {
            if requested_bind_name == bind_name {
                for input_event_type in bind {
                    if &input_event_type == &event {
                        return true;
                    }
                }
            }
        }
    }

    false
}

pub fn is_bind_released(requested_bind_name: &str) -> bool {
    let events = unsafe { &UP_EVENTS };
    let binds = unsafe { &BINDS };

    for event in events {
        for (bind_name, bind) in binds.iter() {
            if requested_bind_name == bind_name {
                for input_event_type in bind {
                    if &input_event_type == &event {
                        return true;
                    }
                }
            }
        }
    }

    false
}

pub fn mouse_position() -> Vec2 {
    unsafe {
        let x = MOUSE_POSITION.x / WINDOW_RESOLUTION.x;
        let y = -(MOUSE_POSITION.y / WINDOW_RESOLUTION.y);
        Vec2::new(x, y)
    }
}

pub fn mouse_delta() -> Vec2 {
    unsafe {
        MOUSE_DELTA
    }
}

pub fn mouse_position_from_center() -> Vec2 {
    unsafe {
        let x = MOUSE_POSITION.x / WINDOW_RESOLUTION.x - 0.5;
        let y = -(MOUSE_POSITION.y / WINDOW_RESOLUTION.y - 0.5);
        Vec2::new(x, y)
    }
}

pub fn is_mouse_locked() -> bool {
    unsafe {
        IS_MOUSE_LOCKED
    }
}

pub fn set_mouse_locked(lock: bool) {
    unsafe {
        IS_MOUSE_LOCKED = lock
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InputEventType {
    Key(KeyCode),
    Mouse(MouseButton),
}

