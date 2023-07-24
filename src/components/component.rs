use std::collections::HashMap;
use glium::{Display, Frame};
use rcrefcell::RcCell;
use crate::object::Object;

pub trait Component {
    fn start(&mut self) {}
    fn update(&mut self) {}
    fn opaque_render(&mut self, display: &Display, target: &mut Frame) {}
    fn transparent_render(&mut self, display: &Display, target: &mut Frame) {}

    fn get_component_type(&self) -> &str; 

    fn set_owner(&mut self, owner: RcCell<Object>);
    fn get_owner(&self) -> &Option<RcCell<Object>>;

    fn get_data(&self) -> Option<HashMap<&str, String>> { None }
}
