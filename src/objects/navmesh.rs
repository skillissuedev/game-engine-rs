use glam::{Mat4, Vec3, Vec2};
use glium::Display;
//use recast_rs::{util, Heightfield, CompactHeightfield, NoRegions, PolyMesh, ContourBuildFlags, ContourSet};
use crate::{managers::{physics::ObjectBodyParameters, debugger, navigation}, assets::model_asset::ModelAsset};
use super::{Object, Transform, ObjectGroup, gen_object_id};

#[derive(Debug)]
pub struct NavMesh {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    id: u128,
    groups: Vec<ObjectGroup>,
    area_size: Vec2,
}

impl NavMesh {
    pub fn new(name: &str, area_size: Vec2) -> Self {
        Self { 
            name: name.into(),
            transform: Transform::default(),
            parent_transform: None,
            children: vec![],
            id: gen_object_id(),
            groups: vec![],
            area_size
        }
    }
}


impl Object for NavMesh {
    fn start(&mut self) {
        let global_pos = self.global_transform().position;
        let x1 = global_pos.x - self.area_size.x / 2.0;
        let x2 = global_pos.x + self.area_size.x / 2.0;
        let z1 = global_pos.z - self.area_size.y / 2.0;
        let z2 = global_pos.z + self.area_size.y / 2.0;
        dbg!(x1, x2, z1, z2);

        for x in x1.round() as i32..x2.round() as i32 {
            let map_x = navigation::world_x_to_map_x(x as f32);
            println!("{}", x);
            for z in z1.round() as i32..z2.round() as i32 {
                let map_z = navigation::world_z_to_map_z(z as f32);
                navigation::set_map_val_by_map_coords(Vec2::new(map_x as f32, map_z as f32), true);
            }
        }

        dbg!(unsafe {
            &navigation::MAP
        });
    }

    fn update(&mut self) { }

    fn render(&mut self, _display: &mut Display, _target: &mut glium::Frame) { }

    fn children_list(&self) -> &Vec<Box<dyn Object>> {
        &self.children
    }

    fn children_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.children
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn object_type(&self) -> &str {
        "EmptyObject"
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn local_transform(&self) -> Transform {
        self.transform
    }



    fn set_local_transform(&mut self, transform: Transform) {
        self.transform = transform
    }

    fn parent_transform(&self) -> Option<Transform> {
        self.parent_transform
    }

    fn set_parent_transform(&mut self, transform: Transform) {
        self.parent_transform = Some(transform);
    }

    fn set_body_parameters(&mut self, _rigid_body: Option<ObjectBodyParameters>) {
        debugger::error("NavMesh object error!\ncan't use set_body_parameters in this type of objects");
    }

    fn body_parameters(&self) -> Option<ObjectBodyParameters> {
        None
    }

    fn object_id(&self) -> &u128 {
        &self.id
    }

    fn groups_list(&mut self) -> &mut Vec<super::ObjectGroup> {
        &mut self.groups
    }

    fn call(&mut self, name: &str, args: Vec<&str>) -> Option<String> {
        if name == "test" {
            println!("test message {}", args[0])
        }
        None
    }
}

enum CurrentAxis {
    X,
    Y,
    Z
}

#[derive(Debug)]
pub enum NavMeshError {
    HeightmapError,
    RasterizeError,
    PolyMeshError
}

