pub mod main_system;
pub mod world_generator;
pub mod player_manager;

use crate::{
    framework::Framework, managers::{
        debugger, networking::{self, Message, MessageReliability, NetworkError}, render::{Cascades, ShadowTextures}, systems::{register_object_id_name, register_object_id_system, CallList, SystemValue}
    }, objects::Object
};
use egui_glium::egui_winit::egui::Context;
use glam::Mat4;
use glium::{framebuffer::SimpleFrameBuffer, glutin::surface::WindowSurface, Display, Frame};

pub trait System {
    fn client_start(&mut self, framework: &mut Framework);
    fn server_start(&mut self, framework: &mut Framework);
    fn client_update(&mut self, framework: &mut Framework);
    fn server_update(&mut self, framework: &mut Framework);
    fn server_render(&mut self);
    fn client_render(&mut self);
    fn call(&self, call_id: &str);
    fn call_mut(&mut self, call_id: &str);
    fn objects_list(&self) -> &Vec<Box<dyn Object>>;
    fn objects_list_mut(&mut self) -> &mut Vec<Box<dyn Object>>;
    fn call_list(&self) -> CallList;
    fn system_id(&self) -> &str;
    fn is_destroyed(&self) -> bool;
    fn set_destroyed(&mut self, is_destroyed: bool);
    fn reg_message(&mut self, message: Message);
    fn get_value(&mut self, value_name: String) -> Option<SystemValue>;

    fn send_message(
        &mut self,
        reliability: MessageReliability,
        message: Message,
    ) -> Result<(), NetworkError> {
        networking::send_message(reliability, message)
    }

    fn find_object(&self, object_name: &str) -> Option<&Box<dyn Object>> {
        for object in self.objects_list() {
            if object.name() == object_name {
                return Some(object);
            }

            match object.find_object(object_name) {
                Some(found_obj) => return Some(found_obj),
                None => (),
            }
        }

        None
    }

    fn find_object_mut(&mut self, object_name: &str) -> Option<&mut Box<dyn Object>> {
        for object in self.objects_list_mut() {
            if object.name() == object_name {
                return Some(object);
            }

            match object.find_object_mut(object_name) {
                Some(found_obj) => return Some(found_obj),
                None => (),
            }
        }

        None
    }

    fn update_objects(&mut self, framework: &mut Framework) {
        //println!("update objects!");
        self.objects_list_mut()
            .into_iter()
            .for_each(|object| object.update_transform(framework));
        self.objects_list_mut()
            .into_iter()
            .for_each(|object| object.update(framework));
        self.objects_list_mut()
            .into_iter()
            .for_each(|object| object.update_children(framework));
    }

    fn render_objects(
        &mut self,
        framework: &mut Framework
    ) {
        self.objects_list_mut()
            .into_iter()
            .for_each(|object| object.render(framework));
        self.objects_list_mut()
            .into_iter()
            .for_each(|object| object.render_children(framework));
        self.objects_list_mut()
            .into_iter()
            .for_each(|object| object.debug_render(framework));
    }

    fn shadow_render_objects(
        &mut self,
        view_proj: &Mat4,
        display: &Display<WindowSurface>,
        target: &mut SimpleFrameBuffer,
    ) {
        self.objects_list_mut()
            .into_iter()
            .for_each(|object| object.shadow_render(view_proj, display, target));
        self.objects_list_mut()
            .into_iter()
            .for_each(|object| object.shadow_render_children(view_proj, display, target));
    }

    fn destroy_system(&mut self) {
        self.set_destroyed(true);
    }

    fn delete_object(&mut self, framework: &mut Framework, name: &str) {
        for (idx, object) in self.objects_list_mut().iter_mut().enumerate() {
            if object.name() == name {
                if let Some(body_parameters) = object.body_parameters() {
                    if let Some(handle) = body_parameters.collider_handle {
                        framework.physics.remove_collider_by_handle(handle);
                    }
                    if let Some(handle) = body_parameters.rigid_body_handle {
                        framework.physics.remove_rigid_body_by_handle(handle);
                    }
                }

                self.objects_list_mut().remove(idx);
                return;
            }

            if object.delete_child(framework, name) == true {
                return
            }
        }
        debugger::warn(&format!("Failed to delete object \"{}\". An object with such name wasn't found", name));
    }

    fn add_object(&mut self, object: Box<dyn Object>) {
        register_object_id_system(*object.object_id(), self.system_id());
        register_object_id_name(*object.object_id(), object.name());
        self.objects_list_mut().push(object);
        self.objects_list_mut()
            .last_mut()
            .expect("the last object does not exist(why?..)")
            .start();
    }

    fn ui_render(&mut self, _ctx: &Context) {}
}
