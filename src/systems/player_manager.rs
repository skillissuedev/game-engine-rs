use super::System;
use crate::{
    framework::Framework,
    managers::{
        //framework.input.{self, is_mouse_locked, set_mouse_locked, InputEventType},
        input::InputEventType,
        networking::Message,
        systems::{CallList, SystemValue},
    },
    objects::Object,
};
use glam::Vec3;
use glium::winit::keyboard::KeyCode;

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
    fn client_start(&mut self, framework: &mut Framework) {
        framework
            .input
            .new_bind("lock_mouse", vec![InputEventType::Key(KeyCode::KeyL)]);
        framework
            .input
            .new_bind("forward", vec![InputEventType::Key(KeyCode::KeyW)]);
        framework
            .input
            .new_bind("left", vec![InputEventType::Key(KeyCode::KeyA)]);
        framework
            .input
            .new_bind("backwards", vec![InputEventType::Key(KeyCode::KeyS)]);
        framework
            .input
            .new_bind("right", vec![InputEventType::Key(KeyCode::KeyD)]);
        framework
            .input
            .new_bind("cam_up", vec![InputEventType::Key(KeyCode::KeyQ)]);
        framework
            .input
            .new_bind("cam_down", vec![InputEventType::Key(KeyCode::KeyE)]);
    }

    fn server_start(&mut self, _: &mut Framework) {}

    fn client_update(&mut self, framework: &mut Framework) {
        //dbg!(serde_json::from_str::<VirtualKeyCode>("\"Grave\""));
        let camera_position;
        let delta_time = framework.delta_time();
        {
            let render = framework.render.as_mut().unwrap();
            render.set_light_direction(Vec3::new(-0.2, 0.0, 0.0));
            camera_position = render.get_camera_position();
        }
        
        {
            framework.set_global_system_value("PlayerPosition", vec![SystemValue::Vec(
                vec![
                    SystemValue::Float(-camera_position.x),
                    SystemValue::Float(camera_position.y), 
                    SystemValue::Float(camera_position.z)
                ])]
            );
        }

        let render = framework.render.as_mut().unwrap();
        render.set_light_direction(Vec3::new(-0.2, 0.0, 0.0));

        //locking mouse
        if framework.input.is_bind_pressed("lock_mouse") {
            framework.input.set_mouse_locked(!framework.input.is_mouse_locked());
        }

        // movement
        let delta_time = delta_time.as_secs_f32();
        let delta = framework.input.mouse_delta();
        let camera_rotation = render.get_camera_rotation();

        render.set_camera_rotation(Vec3::new(camera_rotation.x - delta.y * 50.0 * delta_time, camera_rotation.y + delta.x * 50.0 * delta_time, camera_rotation.z));

        let speed = 420.0 * delta_time;

        let camera_front = render.get_camera_front();
        let camera_right = render.get_camera_left();
        let mut camera_position = render.get_camera_position();

        if framework.input.is_bind_down("cam_up") {
            render.set_camera_position(Vec3::new(
                camera_position.x,
                camera_position.y + speed,
                camera_position.z,
            ));
            camera_position = render.get_camera_position();
        }

        if framework.input.is_bind_down("cam_down") {
            render.set_camera_position(Vec3::new(
                camera_position.x,
                camera_position.y - speed,
                camera_position.z,
            ));
            camera_position = render.get_camera_position();
        }

        if framework.input.is_bind_down("forward") {
            render.set_camera_position(camera_position + camera_front * speed);
            camera_position = render.get_camera_position();
        }

        if framework.input.is_bind_down("backwards") {
            render.set_camera_position(camera_position - camera_front * speed);
            camera_position = render.get_camera_position();
        }

        if framework.input.is_bind_down("left") {
            render.set_camera_position(camera_position - camera_right * speed);
            camera_position = render.get_camera_position();
        }

        if framework.input.is_bind_down("right") {
            render.set_camera_position(camera_position + camera_right * speed);
        }

        if render.get_camera_rotation().x > 89.0 {
            let rot = render.get_camera_rotation();
            render.set_camera_rotation(Vec3::new(89.0, rot.y, rot.z));
        } else if render.get_camera_rotation().x < -89.0 {
            let rot = render.get_camera_rotation();
            render.set_camera_rotation(Vec3::new(-89.0, rot.y, rot.z));
        }
    }

    fn server_update(&mut self, _: &mut Framework) {}

    fn server_render(&mut self) {}
    fn client_render(&mut self, _: &mut Framework) {}

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

    fn reg_message(&mut self, message: Message) {}

    fn get_value(&mut self, value_name: String) -> Option<SystemValue> {
        None
    }
}
