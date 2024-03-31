use crate::{managers::systems::add_system, systems::test_system::TestSystem};

pub fn start() {
    add_system(Box::new(TestSystem::new()));

    //let lua = LuaSystem::new("lua_sys".into(), "scripts/lua/test.lua".into()).unwrap();
    //add_system(Box::new(lua));
}

pub fn update() {}

pub fn render() {}
