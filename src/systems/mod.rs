use glium::{Display, Frame};
use crate::{objects::Object, managers::systems::CallList};

pub mod test_system;



pub trait System {
    fn start(&mut self) {}
    fn update(&mut self) {}
    fn render(&mut self) {}
    fn call(&self, call_id: &str);    
    fn call_mut(&mut self, call_id: &str);    
    fn get_objects_list(&self) -> &Vec<Box<dyn Object>>;
    fn get_objects_list_mut(&mut self) -> &mut Vec<Box<dyn Object>>;
    fn get_call_list(&self) -> CallList;
    fn system_id(&self) -> &str;
    fn is_destroyed(&self) -> bool;
    fn set_destroyed(&mut self, is_destroyed: bool);

    fn find_object(&self, object_name: &str) -> Option<&Box<dyn Object>> {
        for object in self.get_objects_list() {
            if object.get_name() == object_name {
                return Some(object);
            }


            match object.find_object(object_name) {
                Some(found_obj) => return Some(found_obj),
                None => ()
            }
        }

        return None;
    }

    fn find_object_mut(&mut self, object_name: &str) -> Option<&mut Box<dyn Object>> {
        for object in self.get_objects_list_mut() {
            if object.get_name() == object_name {
                return Some(object);
            }

            match object.find_object_mut(object_name) {
                Some(found_obj) => return Some(found_obj),
                None => ()
            }
        }

        return None;
    }

    fn update_objects(&mut self) {
        self.get_objects_list_mut().into_iter().for_each(|object| object.update());
        self.get_objects_list_mut().into_iter().for_each(|object| object.update_children());
    }

    fn render_objects(&mut self, display: &mut Display, target: &mut Frame) {
        self.get_objects_list_mut().into_iter().for_each(|object| object.render(display, target));
        self.get_objects_list_mut().into_iter().for_each(|object| object.render_children(display, target));
    }

    fn destroy_system(&mut self) {
        self.set_destroyed(true);
    }

    fn add_object(&mut self, object: Box<dyn Object>) {
        self.get_objects_list_mut().push(object);
        self.get_objects_list_mut().last_mut().expect("the last object does not exist(why?..)").start();
    }
}

