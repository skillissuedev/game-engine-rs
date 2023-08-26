use crate::{managers::systems::add_system, systems::test_system::TestSystem};

pub fn start() {
    add_system(Box::new(TestSystem { is_destroyed: false, objects: vec![]}));
}

pub fn update() {}

pub fn render() {}
