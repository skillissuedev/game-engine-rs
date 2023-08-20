use crate::components::object::Object;

trait System {
    fn call();
    fn find_object(name: &str) -> Option<&Box<Object>>;
    fn mut_find_object(name: &str) -> Option<&mut Box<Object>>;
    fn start();
    fn update();
    fn render();
}
