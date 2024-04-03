pub mod test_system;

use crate::{
    managers::{
        networking::{self, Message, MessageReliability, NetworkError},
        systems::{register_object_id_name, register_object_id_system, CallList},
    },
    objects::Object,
};
use glam::Mat4;
use glium::{framebuffer::SimpleFrameBuffer, texture::DepthTexture2d, Display, Frame};

pub trait System {
    fn client_start(&mut self);
    fn server_start(&mut self);
    fn client_update(&mut self);
    fn server_update(&mut self);
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

        return None;
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

        return None;
    }

    fn update_objects(&mut self) {
        //println!("update objects!");
        self.objects_list_mut()
            .into_iter()
            .for_each(|object| object.update_transform());
        self.objects_list_mut()
            .into_iter()
            .for_each(|object| object.update());
        self.objects_list_mut()
            .into_iter()
            .for_each(|object| object.update_children());
    }

    fn render_objects(&mut self, display: &Display, target: &mut Frame, shadow_view_proj: &Mat4, shadow_texture: &DepthTexture2d) {
        self.objects_list_mut()
            .into_iter()
            .for_each(|object| object.render(display, target, shadow_view_proj, shadow_texture));
        self.objects_list_mut()
            .into_iter()
            .for_each(|object| object.render_children(display, target, shadow_view_proj, shadow_texture));
        self.objects_list_mut()
            .into_iter()
            .for_each(|object| object.debug_render());
    }

    fn shadow_render_objects(
        &mut self,
        view_proj: &Mat4,
        display: &Display,
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

    fn add_object(&mut self, object: Box<dyn Object>) {
        register_object_id_system(*object.object_id(), self.system_id());
        register_object_id_name(*object.object_id(), object.name());
        self.objects_list_mut().push(object);
        self.objects_list_mut()
            .last_mut()
            .expect("the last object does not exist(why?..)")
            .start();
    }

    fn call_with_args(&mut self, call_id: &str, args: Vec<String>) -> Option<String> {
        None
    }
}
