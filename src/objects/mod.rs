use crate::{
    framework,
    managers::{
        self,
        physics::{self, BodyType, CollisionGroups, ObjectBodyParameters, RenderColliderType},
        render::{self, Cascades, ShadowTextures},
    },
};
use downcast_rs::{impl_downcast, Downcast};
use glam::{Mat4, Vec3};
use glium::{framebuffer::SimpleFrameBuffer, texture::DepthTexture2d, Display, Frame};
use serde::{Deserialize, Serialize};

pub mod camera_position;
pub mod character_controller;
pub mod empty_object;
pub mod model_object;
pub mod nav_obstacle;
pub mod navmesh;
pub mod ray;
pub mod sound_emitter;
pub mod trigger;

static mut LAST_OBJECT_ID: u128 = 0;

pub fn gen_object_id() -> u128 {
    unsafe {
        LAST_OBJECT_ID += 1;
        LAST_OBJECT_ID
    }
}

pub trait Object: std::fmt::Debug + Downcast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Object")
            .field("name", &self.name())
            .field("object_type", &self.object_type())
            .field("children", &self.children_list())
            .finish()
    }

    fn start(&mut self);
    fn update(&mut self);
    fn children_list(&self) -> &Vec<Box<dyn Object>>;
    fn children_list_mut(&mut self) -> &mut Vec<Box<dyn Object>>;
    fn name(&self) -> &str;
    fn object_type(&self) -> &str;
    fn set_name(&mut self, name: &str);
    fn local_transform(&self) -> Transform;
    fn set_local_transform(&mut self, transform: Transform);
    fn parent_transform(&self) -> Option<Transform>;
    fn set_parent_transform(&mut self, transform: Transform);
    fn set_body_parameters(&mut self, rigid_body: Option<ObjectBodyParameters>);
    fn body_parameters(&self) -> Option<ObjectBodyParameters>;
    fn object_id(&self) -> &u128;

    fn groups_list(&mut self) -> &mut Vec<ObjectGroup>;

    fn call(&mut self, _name: &str, _args: Vec<&str>) -> Option<String> {
        println!("call function is not implemented in this object.");
        None
    }

    fn render(&mut self, _display: &Display, _target: &mut Frame, _cascades: &Cascades, _shadow_textures: &ShadowTextures) {}

    fn shadow_render(
        &mut self,
        _view_proj: &Mat4,
        _display: &Display,
        _target: &mut SimpleFrameBuffer,
    ) {
    }

    // premade fns:
    fn global_transform(&self) -> Transform {
        let base_transformations = self.local_transform();
        match self.parent_transform() {
            Some(transform) => Transform {
                position: base_transformations.position + transform.position,
                rotation: base_transformations.rotation + transform.rotation,
                scale: base_transformations.scale + transform.scale,
            },
            None => base_transformations,
        }
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

        return None;
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

        return None;
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

    fn render_children(&mut self, display: &Display, target: &mut Frame, cascades: &Cascades, shadow_texture: &ShadowTextures) {
        self.children_list_mut().iter_mut().for_each(|child| {
            child.render(display, target, cascades, shadow_texture);
            child.render_children(display, target, cascades, shadow_texture);
        });
    }

    fn shadow_render_children(
        &mut self,
        view_proj: &Mat4,
        display: &Display,
        target: &mut SimpleFrameBuffer,
    ) {
        self.children_list_mut().iter_mut().for_each(|child| {
            child.shadow_render(&view_proj, display, target);
            child.shadow_render_children(&view_proj, display, target);
        });
    }

    fn debug_render(&self) {
        // Adding collider to render manager's render colliders list if debug mode != None
        match framework::get_debug_mode() {
            framework::DebugMode::Full => {
                if let Some(body) = self.body_parameters() {
                    if let Some(mut render_collider) = body.render_collider_type {
                        let transform = self.global_transform();
                        render_collider.set_transform(transform.position, transform.rotation);
                        render::add_collider_to_draw(render_collider);
                    }
                }

                self.children_list()
                    .iter()
                    .for_each(|child| child.debug_render());
            }
            _ => (),
        }
    }

    fn set_position(&mut self, position: Vec3, set_rigid_body_position: bool) {
        let mut transform = self.local_transform();
        transform.position = position;
        self.set_local_transform(transform);

        if let Some(parameters) = self.body_parameters() {
            if set_rigid_body_position == true {
                physics::set_body_position(parameters, position);
            }
        }
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

    fn add_child(&mut self, mut object: Box<dyn Object>) {
        object.set_parent_transform(self.global_transform());
        dbg!(object.object_id());
        self.children_list_mut().push(object);
        self.children_list_mut().last_mut().unwrap().start();
    }

    fn build_object_rigid_body(
        &mut self,
        body_type: Option<BodyType>,
        custom_render_collider: Option<RenderColliderType>,
        mass: f32,
        membership_groups: Option<CollisionGroups>,
        filter_groups: Option<CollisionGroups>,
    ) {
        match body_type {
            Some(body_type) => {
                let mut body_parameters = physics::new_rigid_body(
                    body_type,
                    Some(self.global_transform()),
                    mass,
                    *self.object_id(),
                    membership_groups,
                    filter_groups,
                );
                if let Some(render_collider) = custom_render_collider {
                    body_parameters.set_render_collider(Some(render_collider));
                }
                self.set_body_parameters(Some(body_parameters));
            }
            None => {
                if let Some(mut body) = self.body_parameters() {
                    physics::remove_rigid_body(&mut body);
                }
                if let Some(render_collider) = custom_render_collider {
                    let mut params = ObjectBodyParameters::empty();
                    params.set_render_collider(Some(render_collider));
                    self.set_body_parameters(Some(params));
                }
            }
        }
    }

    fn add_to_group(&mut self, group_name: &str) {
        self.groups_list().push(ObjectGroup(group_name.into()));
        managers::systems::register_object_id_groups(*self.object_id(), self.groups_list());
    }

    fn remove_from_group(&mut self, group_name: &str) {
        self.groups_list().retain(|group| group_name != group.0);
        managers::systems::register_object_id_groups(*self.object_id(), self.groups_list());
    }
}

impl_downcast!(Object);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectGroup(pub String);

impl ObjectGroup {
    pub fn as_raw(&self) -> &String {
        &self.0
    }
}
