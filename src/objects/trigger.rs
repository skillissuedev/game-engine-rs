use crate::{framework::Framework, managers::{
    debugger,
    physics::{self, BodyColliderType, CollisionGroups, ObjectBodyParameters, PhysicsManager, RenderColliderType},
    render, systems,
}};
use glam::Vec3;
use rapier3d::{
    dynamics::{RigidBodyBuilder, RigidBodyHandle, RigidBodyType},
    geometry::{ActiveCollisionTypes, ColliderHandle, ColliderSet, CollisionEvent},
    pipeline::{ActiveEvents, PhysicsPipeline},
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
        physics: &mut PhysicsManager,
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

        let body_handle = physics.rigid_body_set.insert(body);
        let collider_handle = 
            physics.collider_set.insert_with_parent(
                collider,
                body_handle,
                &mut physics.rigid_body_set,
            );

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
    pub fn is_intersecting(&self, physics: &PhysicsManager) -> bool {
        let intersections_count = physics.narrow_phase
            .intersection_pairs_with(self.collider_handle)
            .count();

        match intersections_count {
            0 => {
                let contact_count = physics.narrow_phase
                    .contact_pairs_with(self.collider_handle)
                    .count();
                if contact_count > 0 {
                    true
                } else {
                    false
                }
            }
            _ => true,
        }
    }

    pub fn is_intersecting_with_group(&self, physics: &PhysicsManager, group: ObjectGroup) -> bool {
        let intersections_iter =
            physics.narrow_phase.intersection_pairs_with(self.collider_handle);

        let collider_set = &physics.collider_set;

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
