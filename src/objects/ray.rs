use glam::{Vec3, Quat};
use glium::Display;
use rapier3d::{pipeline::QueryFilter, geometry::InteractionGroups};
use crate::{managers::{physics::{ObjectBodyParameters, CollisionGroups, is_ray_intersecting, RenderRay, get_ray_intersaction_position}, debugger, render}, math_utils::deg_vec_to_rad, framework::{self, DebugMode}};

use super::{Object, Transform, gen_object_id};

#[derive(Debug)]
pub struct Ray {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    id: u128,
    direction: Vec3,
    mask: CollisionGroups
    // TODO: direction, point(using object's position), mask(CollisionGroups)
}

impl Ray {
    pub fn new(name: &str, direction: Vec3, mask: Option<CollisionGroups>) -> Self {
        let mask = match mask {
            Some(mask) => mask,
            None => CollisionGroups::full(), // maybe use all if this won't work?
        };

        Ray { transform: Transform::default(), children: vec![], name: name.to_string(), parent_transform: None, id: gen_object_id(), direction, mask }
    }
}


impl Object for Ray {
    fn start(&mut self) { }

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
        "Ray"
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
        debugger::error("failed to call set_body_parameters!\nRay objects can't have bodies");
    }

    fn body_parameters(&self) -> Option<ObjectBodyParameters> {
        None
    }

    fn object_id(&self) -> &u128 {
        &self.id
    }

    fn groups_list(&self) -> Vec<super::ObjectGroup> {
        todo!()
    }

    fn call(&mut self, name: &str, args: Vec<&str>) -> Option<std::string::String> {
        if name == "is_intersecting" {
            if self.is_intersecting() == true {
                return Some("true".into())
            } else {
                return Some("false".into())
            }
        }
        if name == "get_intersection_position" {
            match self.intersection_position() {
                Some(pos) => {
                    let x_string = pos.x.to_string();
                    let y_string = &pos.y.to_string();
                    let z_string = &pos.z.to_string();
                    let array_string = x_string + ";" + y_string + ";" + z_string;
                    return Some(array_string);
                },
                None => return None
            }
        }
        None
    }

    fn debug_render(&self) {
        if let DebugMode::Full = framework::get_debug_mode() {
            render::add_ray_to_draw(RenderRay { origin: self.global_transform().position, direction: self.rotated_direction() });
        }
    }
}

impl Ray {
    pub fn is_intersecting(&self) -> bool {
        let global_transform = self.global_transform();
        let rotated_direction = self.rotated_direction();

        let toi = rotated_direction.distance(global_transform.position);
        let query_filter = QueryFilter::new().groups(InteractionGroups::new(CollisionGroups::Group1.bits().into(), self.mask.bits().into()));

        let ray = rapier3d::geometry::Ray::new(global_transform.position.into(), rotated_direction.into());

        //dbg!(toi);
        //dbg!(rotated_direction.normalize());
        //dbg!(query_filter.groups);

        is_ray_intersecting(ray, toi, query_filter)
    }

    pub fn intersection_position(&self) -> Option<Vec3> {
        let global_transform = self.global_transform();
        let rotated_direction = self.rotated_direction();

        let toi = rotated_direction.distance(global_transform.position);
        let query_filter = QueryFilter::new().groups(InteractionGroups::new(CollisionGroups::Group1.bits().into(), self.mask.bits().into()));

        let ray = rapier3d::geometry::Ray::new(global_transform.position.into(), rotated_direction.into());

        get_ray_intersaction_position(ray, toi, query_filter)
    }

    fn rotated_direction(&self) -> Vec3 {
        let global_transform = self.global_transform();
        let rotation = deg_vec_to_rad(global_transform.rotation);
        let rotation_quat = Quat::from_euler(glam::EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
        //let direction_quat = Quat::from_euler(glam::EulerRot::XYZ, self.direction.x, self.direction.y, self.direction.z);
        //let rotated_direction = direction_quat
        //    .mul_vec3(global_transform.rotation);
        let rotated_direction = rotation_quat
            .mul_vec3(self.direction);

        rotated_direction
    }
}
