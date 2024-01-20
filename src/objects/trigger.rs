use glam::Vec3;
use glium::Display;
use rapier3d::{geometry::{ColliderHandle, CollisionEvent}, pipeline::ActiveEvents, dynamics::{RigidBodyBuilder, RigidBodyType, RigidBodyHandle}};
use crate::managers::{physics::{ObjectBodyParameters, CollisionGroups, BodyColliderType, self, RenderColliderType}, debugger, render};

use super::{Object, Transform, gen_object_id};

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
    current_events: Vec<CollisionEvent>
    // collider,  mask(CollisionGroups)
}

impl Trigger {
    pub fn new(name: &str, membership_group: Option<CollisionGroups>, mask: Option<CollisionGroups>, collider: BodyColliderType) -> Self {
        let mask = match mask {
            Some(mask) => mask,
            None => CollisionGroups::full(), // maybe use all if this won't work?
        };
        let membership_group = match membership_group {
            Some(group) => group,
            None => CollisionGroups::Group1,
        };
        let render_collider = physics::collider_type_to_render_collider(&collider, true);

        let collider = physics::collider_type_to_collider_builder(collider, membership_group, mask).sensor(true).active_events(ActiveEvents::COLLISION_EVENTS).build();
        let body = RigidBodyBuilder::new(RigidBodyType::Fixed).build();

        let body_handle = unsafe { physics::RIGID_BODY_SET.insert(body) };
        let collider_handle = unsafe { physics::COLLIDER_SET.insert_with_parent(collider, body_handle, &mut physics::RIGID_BODY_SET) };

        Trigger { 
            name: name.to_string(),
            transform: Transform::default(),
            parent_transform: None,
            children: vec![],
            id: gen_object_id(),
            mask,
            membership_group,
            body_handle, 
            collider_handle,
            render_collider,
            current_events: Vec::new()
        }
    }
}


impl Object for Trigger {
    fn start(&mut self) { }

    fn update(&mut self) { }

    fn render(&mut self, _display: &mut Display, _target: &mut glium::Frame) { }

    fn get_children_list(&self) -> &Vec<Box<dyn Object>> {
        &self.children
    }

    fn get_children_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.children
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_object_type(&self) -> &str {
        "Ray"
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn get_local_transform(&self) -> Transform {
        self.transform
    }



    fn set_local_transform(&mut self, transform: Transform) {
        self.transform = transform
    }

    fn get_parent_transform(&self) -> Option<Transform> {
        self.parent_transform
    }

    fn set_parent_transform(&mut self, transform: Transform) {
        self.parent_transform = Some(transform);
    }

    fn set_body_parameters(&mut self, _rigid_body: Option<ObjectBodyParameters>) {
        debugger::error("failed to call set_body_parameters!\nTrigger objects can't have bodies");
    }

    fn get_body_parameters(&self) -> Option<ObjectBodyParameters> {
        None
    }

    fn get_object_id(&self) -> &u128 {
        &self.id
    }

    fn call(&mut self, name: &str, args: Vec<&str>) -> Option<std::string::String> {
        if name == "is_colliding" {
            return match self.is_colliding() {
                true => Some("true".into()),
                false => Some("false".into()),
            }
        }

        None
    }

    fn get_global_transform(&self) -> Transform {
        let base_transformations = self.get_local_transform();
        let additional_transformations: Transform;
        match self.get_parent_transform() {
            Some(transform) => additional_transformations = transform,
            None => additional_transformations = Transform::default(),
        }

        Transform {
            position: base_transformations.position + additional_transformations.position,
            rotation: base_transformations.rotation + additional_transformations.rotation,
            scale: base_transformations.scale + additional_transformations.scale,
        }
    }

    fn find_object(&self, object_name: &str) -> Option<&Box<dyn Object>> {
        for object in self.get_children_list() {
            if object.get_name() == object_name {
                return Some(object);
            }

            match object.find_object(object_name) {
                Some(found_obj) => return Some(found_obj),
                None => ()
            }
        }

        return None;
    }

    fn find_object_mut(&mut self, object_name: &str) -> Option<&mut Box<dyn Object>> {
        for object in self.get_children_list_mut() {
            if object.get_name() == object_name {
                return Some(object);
            }

            match object.find_object_mut(object_name) {
                Some(found_obj) => return Some(found_obj),
                None => ()
            }
        }

        return None;
    }

    fn update_transform(&mut self) {
        if let Some(parameters) = self.get_body_parameters() {
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
        let global_transform = self.get_global_transform();

        self.get_children_list_mut().iter_mut().for_each(|child| {
            child.set_parent_transform(global_transform);
            child.update(); 
            child.update_children(); 
        });
    }

    fn render_children(&mut self, display: &mut Display, target: &mut glium::Frame) {
        self.get_children_list_mut().iter_mut().for_each(|child| child.render(display, target));
    }

    fn debug_render(&self) {
        // Adding collider to render manager's render colliders list if debug mode != None
        match crate::framework::get_debug_mode() {
            crate::framework::DebugMode::Full => {
                if let Some(render_collider) = self.render_collider {
                    render::add_collider_to_draw(render_collider);
                }

                self.get_children_list().iter().for_each(|child| child.debug_render());
            },
            _ => ()
        }
    }

    fn set_position(&mut self, position: Vec3, set_rigid_body_position: bool) {
        let mut transform = self.get_local_transform();
        transform.position = position;
        self.set_local_transform(transform);

        physics::set_rigidbody_position(self.body_handle, position);
    }

    fn set_rotation(&mut self, rotation: Vec3, set_rigid_body_rotation: bool) {
        let mut transform = self.get_local_transform();
        transform.rotation = rotation;
        self.set_local_transform(transform);

        if let Some(parameters) = self.get_body_parameters() {
            if set_rigid_body_rotation == true {
                physics::set_body_rotation(parameters, rotation);
            }
        }
    }

    fn set_scale(&mut self, scale: Vec3) {
        let mut transform = self.get_local_transform();
        transform.scale = scale;
        self.set_local_transform(transform);
    }

    fn add_child(&mut self, mut object: Box<dyn Object>) {
        object.set_parent_transform(self.get_global_transform());
        self.get_children_list_mut().push(object);
    }

    fn build_object_rigid_body(&mut self, body_type: Option<physics::BodyType>,
        custom_render_collider: Option<physics::RenderColliderType>, mass: f32, membership_groups: Option<CollisionGroups>, filter_groups: Option<CollisionGroups>) {

        match body_type {
            Some(body_type) => {
                let mut body_parameters = 
                    physics::new_rigid_body(body_type, Some(self.get_global_transform()), mass, *self.get_object_id(), membership_groups, filter_groups);
                if let Some(render_collider) = custom_render_collider {
                    body_parameters.set_render_collider(Some(render_collider));
                }
                self.set_body_parameters(Some(body_parameters));
            },
            None => {
                if let Some(mut body) = self.get_body_parameters() {
                    physics::remove_rigid_body(&mut body);
                }
                if let Some(render_collider) = custom_render_collider {
                    let mut params = ObjectBodyParameters::empty();
                    params.set_render_collider(Some(render_collider));
                    self.set_body_parameters(Some(params));
                }
            },
        }
    }
}

impl std::fmt::Debug for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Trigger")
            .field("name", &self.get_name())
            .field("object_type", &self.get_object_type())
            .field("children", &self.get_children_list())
            .finish()
    }
}

impl Trigger {
    pub fn is_colliding(&self) -> bool {
        let intersections_count = unsafe {
            physics::NARROW_PHASE.intersections_with(self.collider_handle).count()
        };

        match intersections_count {
            0 => {
                let contact_count = unsafe {
                    physics::NARROW_PHASE.contacts_with(self.collider_handle).count()
                };
                if contact_count > 0 { 
                    true
                } else {
                    false
                }
            }
            _ => true
        }
    }
    
    pub fn get_mask(&self) -> &CollisionGroups {
        &self.mask
    }

    pub fn set_mask(&mut self, mask: CollisionGroups) {
        self.mask = mask
    }

    pub fn get_group(&self) -> &CollisionGroups {
        &self.membership_group
    }

    pub fn set_group(&mut self, group: CollisionGroups) {
        self.membership_group = group
    }
}
