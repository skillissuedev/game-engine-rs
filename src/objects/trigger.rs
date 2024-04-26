use crate::managers::{
    debugger,
    physics::{self, BodyColliderType, CollisionGroups, ObjectBodyParameters, RenderColliderType},
    render, systems,
    ui::{draw_vec3_editor_inspector, Vec3Inspector},
};
use glam::Vec3;
use rapier3d::{
    dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodyType},
    geometry::{ActiveCollisionTypes, ColliderHandle, ColliderSet, CollisionEvent},
    pipeline::ActiveEvents,
};

use super::{gen_object_id, Object, ObjectGroup, Transform};

pub struct Trigger {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    id: u128,
    mask: CollisionGroups,
    membership_group: CollisionGroups,
    body_handle: RigidBodyHandle,
    collider_handle: ColliderHandle,
    render_collider: Option<RenderColliderType>,
    current_events: Vec<CollisionEvent>,
}

impl Trigger {
    pub fn new(
        name: &str,
        membership_group: Option<CollisionGroups>,
        mask: Option<CollisionGroups>,
        collider: BodyColliderType,
    ) -> Self {
        let id = gen_object_id();
        let mask = match mask {
            Some(mask) => mask,
            None => CollisionGroups::full(), // maybe use all if this won't work?
        };
        let membership_group = match membership_group {
            Some(group) => group,
            None => CollisionGroups::Group1,
        };
        let render_collider = physics::collider_type_to_render_collider(&collider, true);

        let mut collider =
            physics::collider_type_to_collider_builder(collider, membership_group, mask)
                .sensor(true)
                .active_events(ActiveEvents::COLLISION_EVENTS)
                .active_collision_types(
                    ActiveCollisionTypes::default()
                        | ActiveCollisionTypes::FIXED_FIXED
                        | ActiveCollisionTypes::DYNAMIC_FIXED
                        | ActiveCollisionTypes::DYNAMIC_DYNAMIC
                        | ActiveCollisionTypes::DYNAMIC_KINEMATIC
                        | ActiveCollisionTypes::DYNAMIC_FIXED
                        | ActiveCollisionTypes::KINEMATIC_FIXED,
                )
                .build();
        collider.user_data = id;

        let body = RigidBodyBuilder::new(RigidBodyType::Fixed).build();

        let body_handle = unsafe { physics::RIGID_BODY_SET.insert(body) };
        let collider_handle = unsafe {
            physics::COLLIDER_SET.insert_with_parent(
                collider,
                body_handle,
                &mut physics::RIGID_BODY_SET,
            )
        };

        Trigger {
            name: name.to_string(),
            transform: Transform::default(),
            parent_transform: None,
            children: vec![],
            id,
            mask,
            membership_group,
            body_handle,
            collider_handle,
            render_collider,
            current_events: Vec::new(),
        }
    }
}

impl Object for Trigger {
    fn start(&mut self) {}

    fn update(&mut self) {}

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
        debugger::error("failed to call set_body_parameters!\nTrigger objects can't have bodies");
    }

    fn body_parameters(&self) -> Option<ObjectBodyParameters> {
        None
    }

    fn object_id(&self) -> &u128 {
        &self.id
    }

    fn inspector_ui(&mut self, ui: &mut egui_glium::egui_winit::egui::Ui) {
        ui.heading("Inspector parameters");
        ui.label("this object type is made specifically for servers so there's noting to change here ._.");
    }

    fn groups_list(&mut self) -> &mut Vec<super::ObjectGroup> {
        todo!()
    }

    fn find_object(&self, object_name: &str) -> Option<&Box<dyn Object>> {
        for object in self.children_list() {
            if object.name() == object_name {
                return Some(object);
            }

            match object.find_object(object_name) {
                Some(found_obj) => return Some(found_obj),
                None => (),
            }
        }

        None
    }

    fn find_object_mut(&mut self, object_name: &str) -> Option<&mut Box<dyn Object>> {
        for object in self.children_list_mut() {
            if object.name() == object_name {
                return Some(object);
            }

            match object.find_object_mut(object_name) {
                Some(found_obj) => return Some(found_obj),
                None => (),
            }
        }

        None
    }

    fn update_transform(&mut self) {
        if let Some(parameters) = self.body_parameters() {
            if let None = parameters.rigid_body_handle {
                return;
            }

            let position_and_rotation_option = physics::get_body_transformations(parameters);

            if let Some((pos, rot)) = position_and_rotation_option {
                self.set_position(pos, false);
                self.set_rotation(rot, false);
            }
        }
    }

    fn update_children(&mut self) {
        let global_transform = self.global_transform();

        self.children_list_mut().iter_mut().for_each(|child| {
            child.set_parent_transform(global_transform);
            child.update();
            child.update_children();
        });
    }

    fn debug_render(&self) {
        // Adding collider to render manager's render colliders list if debug mode != None
        match crate::framework::get_debug_mode() {
            crate::framework::DebugMode::Full => {
                if let Some(mut render_collider) = self.render_collider {
                    let transform = self.global_transform();
                    render_collider.set_transform(transform.position, transform.rotation);
                    render::add_collider_to_draw(render_collider);
                }

                self.children_list()
                    .iter()
                    .for_each(|child| child.debug_render());
            }
            _ => (),
        }
    }

    fn set_position(&mut self, position: Vec3, _set_rigid_body_position: bool) {
        let mut transform = self.local_transform();
        transform.position = position;
        self.set_local_transform(transform);

        physics::set_rigidbody_position(self.body_handle, position);
    }

    fn set_rotation(&mut self, rotation: Vec3, set_rigid_body_rotation: bool) {
        let mut transform = self.local_transform();
        transform.rotation = rotation;
        self.set_local_transform(transform);

        if let Some(parameters) = self.body_parameters() {
            if set_rigid_body_rotation == true {
                physics::set_body_rotation(parameters, rotation);
            }
        }
    }

    fn set_scale(&mut self, scale: Vec3) {
        let mut transform = self.local_transform();
        transform.scale = scale;
        self.set_local_transform(transform);
    }
}

impl std::fmt::Debug for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Trigger")
            .field("name", &self.name())
            .field("object_type", &self.object_type())
            .field("children", &self.children_list())
            .finish()
    }
}

impl Trigger {
    pub fn is_colliding(&self) -> bool {
        let intersections_count = unsafe {
            physics::NARROW_PHASE
                .intersection_pairs_with(self.collider_handle)
                .count()
        };

        match intersections_count {
            0 => {
                let contact_count = unsafe {
                    physics::NARROW_PHASE
                        .contact_pairs_with(self.collider_handle)
                        .count()
                };
                if contact_count > 0 {
                    true
                } else {
                    false
                }
            }
            _ => true,
        }
    }

    pub fn is_intersecting_with_group(&self, group: ObjectGroup) -> bool {
        let intersections_iter =
            unsafe { physics::NARROW_PHASE.intersection_pairs_with(self.collider_handle) };

        let collider_set = unsafe { &physics::COLLIDER_SET };

        for (collider1, collider2, intersecting) in intersections_iter {
            if intersecting {
                if collider1 == self.collider_handle {
                    if is_collider_in_group(collider_set, collider2, &group) == true {
                        return true;
                    }
                } else {
                    if is_collider_in_group(collider_set, collider1, &group) == true {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn mask(&self) -> &CollisionGroups {
        &self.mask
    }

    pub fn set_mask(&mut self, mask: CollisionGroups) {
        self.mask = mask
    }

    pub fn group(&self) -> &CollisionGroups {
        &self.membership_group
    }

    pub fn set_group(&mut self, group: CollisionGroups) {
        self.membership_group = group
    }
}

fn is_collider_in_group(
    collider_set: &ColliderSet,
    collider_handle: ColliderHandle,
    group: &ObjectGroup,
) -> bool {
    let collider = collider_set.get(collider_handle);

    if let Some(collider) = collider {
        //dbg!(systems::get_object_name_with_id(collider.user_data));
        if let Some(groups_list) = systems::get_object_groups_with_id(collider.user_data) {
            //dbg!(&groups_list);
            if groups_list
                .iter()
                .filter(|group_in_list| *group_in_list == group)
                .count()
                > 0
            {
                return true;
            }
        }
    }

    false
}
