use crate::{
    assets::sound_asset::SoundAsset,
    components::sound_emitter::{SoundEmitter, SoundEmitterType},
    managers::render,
    object::new_object,
};
use ultraviolet::Vec3;

pub fn start() {
    let sound = SoundAsset::load_from_wav("sounds/test.wav").unwrap();
    let obj = new_object("1");
    let mut emitter = Box::new(SoundEmitter::new(&sound, SoundEmitterType::Positional).unwrap());
    emitter.play_sound();
    emitter.set_max_distance(90.0);
    emitter.set_looping(true);
    obj.add_component(emitter);
}

pub fn update() {
    let pos = render::get_camera_position();
    let rot = render::get_camera_rotation();

    render::set_camera_position(Vec3 { x: 10.0, y: 0.0, z: 0.0 });
    println!("{:?}", rot);
    /*render::set_camera_position(Vec3 {
        x: pos.x,
        y: 0.0,
        z: pos.z + 0.2,
    });*/
    render::set_camera_rotation(Vec3 { x: rot.x, y: rot.y + 0.5, z: rot.z });
}

pub fn render() {}
