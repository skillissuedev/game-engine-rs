use crate::{managers::systems::add_system, systems::test_system::TestSystem};


pub fn start() {
    add_system(Box::new(TestSystem { is_destroyed: false, objects: vec![]}));
    /*let dyon_sys = DyonSystem::new("dyon system", "scripts/dyon/test.dyon");
    match dyon_sys {
        Ok(system) => {
            add_system(Box::new(system));
            println!("{:?}", get_system_with_id("dyon system").unwrap().get_call_list());
        },
        Err(err) => panic!("got an error when trying to create a dyon system!\nerr: {:?}", err)
    }*/

    /*let mut lua = LuaSystem::new("lua_sys".into(), "scripts/lua/test.lua".into()).unwrap();
    lua.add_object(Box::new(EmptyObject::new("cool")));
    add_system(Box::new(lua));*/
}

pub fn update() {}

pub fn render() {}
