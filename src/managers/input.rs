use super::debugger;
use glam::Vec2;
use std::collections::HashMap;
use glium::winit::{
    event::{DeviceEvent, ElementState, MouseButton, WindowEvent},
    keyboard::KeyCode,
};

#[derive(Default)]
pub struct InputManager {
    binds: HashMap<String, Vec<InputEventType>>,
    down_events: Vec<InputEventType>,
    just_pressed_events: Vec<InputEventType>,
    up_events: Vec<InputEventType>,
    mouse_position: Vec2,
    mouse_delta: Vec2,
    window_resolution: Vec2,
    is_mouse_locked: bool,
}

impl InputManager {
    pub fn new_bind(&mut self, name: &str, input_events: Vec<InputEventType>) {
        match self.binds.get_mut(name) {
            Some(bind_evs) => {
                debugger::warn("Binds '{}' already exist, adding new key in list");
                input_events.iter().for_each(|ev| bind_evs.push(*ev));
            }
            None => {
                self.binds.insert(name.into(), input_events);
            }
        }
    }

    pub fn get_bind_keys(&self, name: String) -> Option<Vec<InputEventType>> {
        match self.binds.get(&name) {
            Some(bind) => Some(bind.to_owned()),
            None => None,
        }
    }

    pub fn reg_device_event(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.mouse_delta = Vec2::new(delta.0 as f32, delta.1 as f32)
            }
            _ => (),
        }
    }

    pub fn reg_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                //input,
                is_synthetic: _,
                event: input,
            } => {
                self.binds.iter().for_each(|bind_events| {
                    for bind_ev in bind_events.1 {
                        if let InputEventType::Key(keycode) = bind_ev {
                            if keycode == &input.physical_key {
                                let input_ev_type = InputEventType::Key(*keycode);
                                match input.state {
                                    ElementState::Pressed => {
                                        if self.down_events.contains(&input_ev_type) == false {
                                            self.down_events.push(input_ev_type);
                                            self.just_pressed_events.push(input_ev_type);
                                        }
                                    }
                                    ElementState::Released => {
                                        if self.up_events.contains(&input_ev_type) == false {
                                            self.up_events.push(input_ev_type);
                                        }
                                        self.down_events.retain(|value| value != &input_ev_type);
                                    }
                                }
                            }
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
                        if self.down_events.contains(&input_ev_type) == false {
                            self.down_events.push(input_ev_type);
                            self.just_pressed_events.push(input_ev_type);
                        }
                    }
                    ElementState::Released => {
                        if self.up_events.contains(&input_ev_type) == false {
                            self.up_events.push(input_ev_type);
                        }
                        self.down_events.retain(|value| value != &input_ev_type);
                    }
                }
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.mouse_position = Vec2::new(position.x as f32, position.y as f32);
            }
            WindowEvent::Resized(new_size) => {
                self.window_resolution = Vec2::new(new_size.width as f32, new_size.height as f32)
            }
            _ => (),
        }
    }

    pub fn update(&mut self) {
        self.just_pressed_events.clear();
        self.up_events.clear();
        self.mouse_delta = Vec2::ZERO;
    }

    pub fn is_bind_pressed(&self, requested_bind_name: &str) -> bool {
        for event in &self.just_pressed_events {
            for (bind_name, bind) in self.binds.iter() {
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

    pub fn is_bind_down(&self, requested_bind_name: &str) -> bool {
        for event in &self.down_events {
            for (bind_name, bind) in self.binds.iter() {
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

    pub fn is_bind_released(&self, requested_bind_name: &str) -> bool {
        for event in &self.up_events {
            for (bind_name, bind) in self.binds.iter() {
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

    pub fn mouse_position(&self) -> Vec2 {
        let x = self.mouse_position.x / self.window_resolution.x;
        let y = -(self.mouse_position.y / self.window_resolution.y);
        Vec2::new(x, y)
    }

    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    pub fn mouse_position_from_center(&self) -> Vec2 {
        let x = self.mouse_position.x / self.window_resolution.x - 0.5;
        let y = -(self.mouse_position.y / self.window_resolution.y - 0.5);
        Vec2::new(x, y)
    }

    pub fn is_mouse_locked(&self) -> bool {
        self.is_mouse_locked
    }

    pub fn set_mouse_locked(&mut self, lock: bool) {
        self.is_mouse_locked = lock
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InputEventType {
    Key(KeyCode),
    Mouse(MouseButton),
}
