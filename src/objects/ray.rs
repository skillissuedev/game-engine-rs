use std::collections::HashMap;

use crate::{
    framework::{DebugMode, Framework},
    managers::{
        self, debugger, physics::{CollisionGroups, ObjectBodyParameters, PhysicsManager, RenderRay}, systems::{self, SystemValue}, ui::Vec3Inspector
    },
};
use glam::Vec3;
use rapier3d::{geometry::InteractionGroups, pipeline::QueryFilter, prelude::QueryFilterFlags};

use super::{gen_object_id, Object, ObjectGroup, Transform};

#[derive(Debug)]
pub struct Ray {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    id: u128,
    groups: Vec<ObjectGroup>,
    direction: Vec3,
    mask: CollisionGroups,
    inspector: Vec3Inspector,
    object_properties: HashMap<String, Vec<crate::managers::systems::SystemValue>>
}

impl Ray {
    pub fn new(name: &str, direction: Vec3, mask: Option<CollisionGroups>) -> Self {
        let mask = match mask {
            Some(mask) => mask,
            None => CollisionGroups::full(), // maybe use all if this won't work?
        };

        Ray {
            transform: Transform::default(),
            children: vec![],
            name: name.to_string(),
            parent_transform: None,
            id: gen_object_id(),
            groups: vec![],
            direction,
            mask,
            inspector: Vec3Inspector::default(),
            object_properties: HashMap::new()
        }
    }
}

impl Object for Ray {
    fn set_object_properties(&mut self, properties: HashMap<String, Vec<crate::managers::systems::SystemValue>>) {
        self.object_properties = properties.clone();
        crate::managers::systems::register_object_id_properties(self.object_id().to_owned(), properties);
    }

    fn object_properties(&self) -> &HashMap<String, Vec<crate::managers::systems::SystemValue>> {
        &self.object_properties
    }

    fn start(&mut self) {}

    fn update(&mut self, _: &mut Framework) {}

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

    fn inspector_ui(&mut self, _: &mut Framework, ui: &mut egui_glium::egui_winit::egui::Ui) {
        ui.heading("Ray parameters");
        ui.label("direction:");
        if let Some(new_dir) =
            managers::ui::draw_vec3_editor_inspector(ui, &mut self.inspector, &self.direction, true)
        {
            self.direction = new_dir;
        }
    }

    fn groups_list(&mut self) -> &mut Vec<super::ObjectGroup> {
        &mut self.groups
    }

    /*
    fn debug_render(&self, framework: &mut Framework) {
        if let DebugMode::Full = framework.debug_mode() {
            framework
                .render
                .as_mut()
                .expect("No render but debug_render is still being called. pls fix")
                .add_ray_to_draw(RenderRay {
                    origin: self.global_transform().position,
                    direction: self.direction,
                });
        }
    }*/
}

impl Ray {
    pub fn is_intersecting(&self, physics: &PhysicsManager) -> bool {
        let global_transform = self.global_transform();
        let toi = global_transform
            .position
            .distance(global_transform.position + self.direction);

        let query_filter = QueryFilter::new().groups(InteractionGroups::new(
            CollisionGroups::Group1.bits().into(),
            self.mask.bits().into(),
        ));

        let ray =
            rapier3d::geometry::Ray::new(global_transform.position.into(), self.direction.into());

        physics.is_ray_intersecting(ray, toi, query_filter)
    }

    pub fn intersection_object_name(&self, physics: &PhysicsManager) -> Option<String> {
        let global_transform = self.global_transform();
        let toi = global_transform
            .position
            .distance(global_transform.position + self.direction);

        let mut query_filter = QueryFilter::new().groups(InteractionGroups::new(
            CollisionGroups::Group1.bits().into(),
            self.mask.bits().into(),
        ));
        query_filter.flags = QueryFilterFlags::empty();

        let ray =
            rapier3d::geometry::Ray::new(global_transform.position.into(), self.direction.into());

        let object_id = physics.get_ray_intersaction_object_id(ray, toi, query_filter);

        match object_id {
            Some(object_id) => systems::get_object_name_with_id(object_id),
            None => None,
        }
    }

    pub fn intersection_object_groups(&self, physics: &PhysicsManager) -> Option<Vec<ObjectGroup>> {
        let global_transform = self.global_transform();
        let toi = global_transform
            .position
            .distance(global_transform.position + self.direction);

        let query_filter = QueryFilter::new().groups(InteractionGroups::new(
            CollisionGroups::Group1.bits().into(),
            self.mask.bits().into(),
        ));

        let ray =
            rapier3d::geometry::Ray::new(global_transform.position.into(), self.direction.into());

        let object_id = physics.get_ray_intersaction_object_id(ray, toi, query_filter);

        match object_id {
            Some(object_id) => systems::get_object_groups_with_id(object_id),
            None => None,
        }
    }

    pub fn intersection_object_properties(&self, physics: &PhysicsManager) -> Option<HashMap<String, Vec<SystemValue>>> {
        let global_transform = self.global_transform();
        let toi = global_transform
            .position
            .distance(global_transform.position + self.direction);

        let query_filter = QueryFilter::new().groups(InteractionGroups::new(
            CollisionGroups::Group1.bits().into(),
            self.mask.bits().into(),
        ));

        let ray =
            rapier3d::geometry::Ray::new(global_transform.position.into(), self.direction.into());

        let object_id = physics.get_ray_intersaction_object_id(ray, toi, query_filter);

        match object_id {
            Some(object_id) => systems::get_object_properties_with_id(object_id),
            None => None,
        }
    }

    pub fn intersection_position(&self, physics: &PhysicsManager) -> Option<Vec3> {
        let global_transform = self.global_transform();
        let toi = global_transform
            .position
            .distance(global_transform.position + self.direction);

        let query_filter = QueryFilter::new().groups(InteractionGroups::new(
            CollisionGroups::Group1.bits().into(),
            self.mask.bits().into(),
        ));

        let ray =
            rapier3d::geometry::Ray::new(global_transform.position.into(), self.direction.into());

        match physics.get_ray_intersaction_position(ray, toi, query_filter) {
            Some(pos) => Some(Vec3::new(pos.x, pos.y, pos.z)),
            None => None,
        }
    }

    pub fn set_direction(&mut self, dir: Vec3) {
        self.direction = dir;
    }
}
