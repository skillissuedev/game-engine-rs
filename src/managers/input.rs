use std::collections::HashMap;
use glium::glutin::event::{VirtualKeyCode, WindowEvent, MouseButton, ElementState};
use once_cell::sync::Lazy;
use super::debugger;

static mut BINDS: Lazy<HashMap<String, Vec<InputEventType>>> = Lazy::new(|| HashMap::new());
static mut DOWN_EVENTS: Vec<InputEventType> = vec![];
static mut JUST_PRESSED_EVENTS: Vec<InputEventType> = vec![];
static mut UP_EVENTS: Vec<InputEventType> = vec![];

pub fn new_bind(name: &str, input_events: Vec<InputEventType>) {
    unsafe { 
        match BINDS.get_mut(name) {
            Some(bind_evs) => {
                debugger::warn("Binds '{}' already exist, adding new key in list");
                input_events.iter().for_each(|ev| bind_evs.push(*ev));
            },
            None => {
                BINDS.insert(name.into(), input_events);
            },
        }
    }
}

pub fn get_bind_keys(name: String) -> Option<Vec<InputEventType>> {
    unsafe {
        match BINDS.get(&name) {
            Some(bind) => Some(bind.to_owned()),
            None => None
        }
    }
}

pub fn reg_event(event: &WindowEvent) { 
    unsafe {
        match event {
            WindowEvent::KeyboardInput { device_id: _, input, is_synthetic: _ } => {
                BINDS.iter().for_each(|bind_events| {
                    for bind_ev in bind_events.1 {
                        match bind_ev {
                            InputEventType::Key(keycode) => {
                                if Some(keycode) == input.virtual_keycode.as_ref() {
                                    let input_ev_type = InputEventType::Key(*keycode);
                                    match input.state {
                                        ElementState::Pressed => {
                                            if DOWN_EVENTS.contains(&input_ev_type) == false {
                                                DOWN_EVENTS.push(input_ev_type);
                                                JUST_PRESSED_EVENTS.push(input_ev_type);
                                            }
                                        },
                                        ElementState::Released => {
                                            if UP_EVENTS.contains(&input_ev_type) == false {
                                                UP_EVENTS.push(input_ev_type);
                                                for i in 0..DOWN_EVENTS.len() - 1 {
                                                    let val = DOWN_EVENTS[i];
                                                    if val == input_ev_type {
                                                        DOWN_EVENTS.remove(i);
                                                    }
                                                }
                                            }
                                        },
                                    };
                                }
                            },
                            InputEventType::Mouse(_) => (),
                        }
                    }
                });
            },
            WindowEvent::MouseInput { device_id: _, state, button, modifiers: _ } => {
                let input_ev_type = InputEventType::Mouse(*button);
                match state {
                    ElementState::Pressed => {
                        if DOWN_EVENTS.contains(&input_ev_type) == false {
                            DOWN_EVENTS.push(input_ev_type);
                            JUST_PRESSED_EVENTS.push(input_ev_type);
                        }
                    },
                    ElementState::Released => {
                        if UP_EVENTS.contains(&input_ev_type) == false {
                            UP_EVENTS.push(input_ev_type);
                            for i in 0..DOWN_EVENTS.len() {
                                let val = DOWN_EVENTS[i];
                                if val == input_ev_type {
                                    DOWN_EVENTS.remove(i);
                                }
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

pub fn update() {
    unsafe {
        JUST_PRESSED_EVENTS.clear();
        UP_EVENTS.clear();
    }
}

pub fn is_bind_pressed(requested_bind_name: &str) -> bool {
    let events = unsafe {
        &JUST_PRESSED_EVENTS
    };
    let binds = unsafe {
        &BINDS
    };

    for event in events {
        for (bind_name, bind) in binds.iter() {
            if requested_bind_name == bind_name {
                for input_event_type in bind {
                    if &input_event_type == &event {
                        return true;
                    }
                };
            }
        }
    }

    return false;
}

pub fn is_bind_down(requested_bind_name: &str) -> bool {
    let events = unsafe {
        &DOWN_EVENTS
    };
    let binds = unsafe {
        &BINDS
    };

    for event in events {
        for (bind_name, bind) in binds.iter() {
            if requested_bind_name == bind_name {
                for input_event_type in bind {
                    if &input_event_type == &event {
                        return true;
                    }
                };
            }
        }
    }

    return false;
}

pub fn is_bind_up(requested_bind_name: &str) -> bool {
    let events = unsafe {
        &UP_EVENTS
    };
    let binds = unsafe {
        &BINDS
    };

    for event in events {
        for (bind_name, bind) in binds.iter() {
            if requested_bind_name == bind_name {
                for input_event_type in bind {
                    if &input_event_type == &event {
                        return true;
                    }
                };
            }
        }
    }

    return false;
}

#[derive(Clone, Copy, PartialEq)]
pub enum InputEventType {
    Key(VirtualKeyCode),
    Mouse(MouseButton)
}

#[derive(Clone, Copy, PartialEq)]
pub enum InputEventPressType {
    JustPressed, 
    Down,
    Up
}

