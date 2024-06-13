use super::System;
use crate::{
    framework::{get_delta_time, set_global_system_value}, managers::{
        input::{self, is_mouse_locked, set_mouse_locked, InputEventType},
        networking::Message,
        render::{get_camera_front, get_camera_position, get_camera_right, get_camera_rotation, set_camera_position, set_camera_rotation, set_light_direction},
        systems::{CallList, SystemValue},
    }, objects::Object
};
use glam::Vec3;
use winit::keyboard::KeyCode;

pub struct PlayerManager {
    pub is_destroyed: bool,
    pub objects: Vec<Box<dyn Object>>,
}

impl PlayerManager {
    pub fn new() -> PlayerManager {
        PlayerManager {
            is_destroyed: false,
            objects: vec![],
        }
    }
}

impl System for PlayerManager {
    fn client_start(&mut self) {
        input::new_bind(
            "lock_mouse",
            vec![InputEventType::Key(KeyCode::KeyL)],
        );
        input::new_bind(
            "forward",
            vec![InputEventType::Key(KeyCode::KeyW)],
        );
        input::new_bind(
            "left",
            vec![InputEventType::Key(KeyCode::KeyA)],
        );
        input::new_bind(
            "backwards",
            vec![InputEventType::Key(KeyCode::KeyS)],
        );
        input::new_bind(
            "right",
            vec![InputEventType::Key(KeyCode::KeyD)],
        );
        input::new_bind(
            "cam_up",
            vec![InputEventType::Key(KeyCode::KeyQ)],
        );
        input::new_bind(
            "cam_down",
            vec![InputEventType::Key(KeyCode::KeyE)],
        );
    }

    fn server_start(&mut self) {
    }

    fn client_update(&mut self) {
        //dbg!(serde_json::from_str::<VirtualKeyCode>("\"Grave\""));
        set_light_direction(Vec3::new(-0.2, 0.0, 0.0));
        let camera_position = get_camera_position();
        set_global_system_value("PlayerPosition", vec![SystemValue::Vec3(-camera_position.x, camera_position.y, camera_position.z)]);

        set_light_direction(Vec3::new(-0.2, 0.0, 0.0));

        //locking mouse
        if input::is_bind_pressed("lock_mouse") {
            set_mouse_locked(!is_mouse_locked());
        }

        // movement
        let delta_time = get_delta_time().as_secs_f32();
        let delta = input::mouse_delta();
        let camera_rotation = get_camera_rotation();

        set_camera_rotation(Vec3::new(camera_rotation.x - delta.y * 50.0 * delta_time, camera_rotation.y + delta.x * 50.0 * delta_time, camera_rotation.z));

        let speed = 420.0 * delta_time;

        let camera_front = get_camera_front();
        let camera_right = get_camera_right();
        let mut camera_position = get_camera_position();

        if input::is_bind_down("cam_up") {
            set_camera_position(Vec3::new(
                camera_position.x,
                camera_position.y + speed,
                camera_position.z,
            ));
            camera_position = get_camera_position();
        }

        if input::is_bind_down("cam_down") {
            set_camera_position(Vec3::new(
                camera_position.x,
                camera_position.y - speed,
                camera_position.z,
            ));
            camera_position = get_camera_position();
        }

        if input::is_bind_down("forward") {
            set_camera_position(camera_position + camera_front * speed);
            camera_position = get_camera_position();
        }

        if input::is_bind_down("backwards") {
            set_camera_position(camera_position - camera_front * speed);
            camera_position = get_camera_position();
        }

        if input::is_bind_down("left") {
            set_camera_position(camera_position - camera_right * speed);
            camera_position = get_camera_position();
        }

        if input::is_bind_down("right") {
            set_camera_position(camera_position + camera_right * speed);
        }

        if get_camera_rotation().x > 89.0 {
            let rot = get_camera_rotation();
            set_camera_rotation(Vec3::new(89.0, rot.y, rot.z));
        } else if get_camera_rotation().x < -89.0 {
            let rot = get_camera_rotation();
            set_camera_rotation(Vec3::new(-89.0, rot.y, rot.z));
        }
        //println!("{}", get_camera_position());
    }

    fn server_update(&mut self) {
    }

    fn server_render(&mut self) {}
    fn client_render(&mut self) {}

    fn call(&self, _call_id: &str) {}
    fn call_mut(&mut self, _call_id: &str) {}

    fn objects_list(&self) -> &Vec<Box<dyn Object>> {
        &self.objects
    }

    fn objects_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.objects
    }

    fn call_list(&self) -> CallList {
        CallList {
            immut_call: vec![],
            mut_call: vec![],
        }
    }

    fn system_id(&self) -> &str {
        "PlayerManager"
    }

    fn is_destroyed(&self) -> bool {
        self.is_destroyed
    }
    fn set_destroyed(&mut self, is_destroyed: bool) {
        self.is_destroyed = is_destroyed;
    }

    fn reg_message(&mut self, message: Message) {
    }

    fn get_value(&mut self, value_name: String) -> Option<SystemValue> {
        None
    }
}


