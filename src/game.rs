use ultraviolet::Vec3;

use crate::{assets::{mesh_asset::MeshAsset, shader_asset, texture_asset::TextureAsset}, components::{mesh::Mesh, transform::BasicTransform}, object::new_object}; 

pub fn start() {
    let mesh_asset = MeshAsset::from_gltf("models/test/clin_crash_test.gltf").unwrap();
    let shader_asset = shader_asset::load_default_shader().unwrap();
    let texture_asset = TextureAsset::from_file("textures/checker.png");

    let obj = new_object("1");
    obj.add_component(Box::new(BasicTransform::new(Vec3::new(10.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 45.0), Vec3::new(0.2, 0.2, 0.2))));
    obj.add_component(Box::new(Mesh::new(mesh_asset, Some(texture_asset.unwrap()), shader_asset)));
}

pub fn update() {
}

pub fn render() {

}
