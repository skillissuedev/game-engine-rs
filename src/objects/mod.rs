use std::collections::HashMap;

use crate::{
    framework::{self, Framework},
    managers::{
        self,
        assets::AssetManager,
        physics::{BodyType, CollisionGroups, ObjectBodyParameters, RenderColliderType},
        render::RenderManager, systems::SystemValue,
    },
};
use downcast_rs::{impl_downcast, Downcast};
use egui_glium::egui_winit::egui::Ui;
use glam::Vec3;
use serde::{Deserialize, Serialize};

pub mod character_controller;
pub mod empty_object;
pub mod instanced_model_object;
pub mod instanced_model_transform_holder;
pub mod master_instanced_model_object;
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
    fn update(&mut self, framework: &mut Framework);
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
    fn inspector_ui(&mut self, framework: &mut Framework, ui: &mut Ui);
    /// DO NOT MODIFY THIS DIRECTLY!
    fn groups_list(&mut self) -> &mut Vec<ObjectGroup>;
    fn set_object_properties(&mut self, properties: HashMap<String, Vec<SystemValue>>);
    fn object_properties(&self) -> &HashMap<String, Vec<SystemValue>>;

    fn call(&mut self, _name: &str, _args: Vec<&str>) -> Option<String> {
        println!("call function is not implemented in this object.");
        None
    }

    fn render(&mut self, _framework: &mut Framework) {}

    fn shadow_render(
        &mut self,
        /*_framework: &mut Framework,
        _target: &mut SimpleFrameBuffer*/
        _render: &mut RenderManager,
        _assets: &AssetManager,
        //_current_cascade: &CurrentCascade,
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

    fn update_transform(&mut self, framework: &mut Framework) {
        if let Some(parameters) = self.body_parameters() {
            if let None = parameters.rigid_body_handle {
                return;
            }

            let position_and_rotation_option =
                framework.physics.get_body_transformations(parameters);

            if let Some((pos, rot)) = position_and_rotation_option {
                self.set_position(framework, pos, false);
                self.set_rotation(framework, rot, false);
            }
        }
    }

    fn update_children(&mut self, framework: &mut Framework) {
        let global_transform = self.global_transform();

        self.children_list_mut().iter_mut().for_each(|child| {
            child.set_parent_transform(global_transform);
            child.update(framework);
            child.update_children(framework);
        });
    }

    fn render_children(&mut self, framework: &mut Framework) {
        self.children_list_mut().iter_mut().for_each(|child| {
            child.render(framework);
            child.render_children(framework);
        });
    }

    fn shadow_render_children(
        &mut self,
        render: &mut RenderManager,
        assets: &AssetManager,
        //current_cascade: &CurrentCascade,
    ) {
        /*self.children_list_mut().iter_mut().for_each(|child| {
            child.shadow_render(render, assets, &current_cascade);
            child.shadow_render_children(render, assets, &current_cascade);
        });*/
    }

    fn debug_render(&self, framework: &mut Framework) {
        /*
        // Adding collider to render manager's render colliders list if debug mode != None
        match framework.debug_mode() {
            framework::DebugMode::Full => {
                if let Some(body) = self.body_parameters() {
                    if let Some(mut render_collider) = body.render_collider_type {
                        let transform = self.global_transform();
                        render_collider.set_transform(transform.position, transform.rotation);
                        if let Some(render) = &mut framework.render {
                            render.add_collider_to_draw(render_collider)
                        };
                    }
                }

                self.children_list()
                    .iter()
                    .for_each(|child| child.debug_render(framework));
            }
            _ => (),
        }*/
    }

    fn set_position(
        &mut self,
        framework: &mut Framework,
        position: Vec3,
        set_rigid_body_position: bool,
    ) {
        let mut transform = self.local_transform();
        transform.position = position;
        self.set_local_transform(transform);

        if let Some(parameters) = self.body_parameters() {
            if set_rigid_body_position == true {
                framework.physics.set_body_position(parameters, position);
            }
        }
    }

    fn set_rotation(
        &mut self,
        framework: &mut Framework,
        rotation: Vec3,
        set_rigid_body_rotation: bool,
    ) {
        let mut transform = self.local_transform();
        transform.rotation = rotation;
        self.set_local_transform(transform);

        if let Some(parameters) = self.body_parameters() {
            if set_rigid_body_rotation == true {
                framework.physics.set_body_rotation(parameters, rotation);
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
        self.children_list_mut().push(object);
        self.children_list_mut().last_mut().unwrap().start();
    }

    fn delete_child(&mut self, framework: &mut Framework, name: &str) -> bool {
        for (idx, object) in self.children_list_mut().iter_mut().enumerate() {
            if object.name() == name {
                if let Some(body_parameters) = object.body_parameters() {
                    if let Some(handle) = body_parameters.rigid_body_handle {
                        framework.physics.remove_rigid_body_by_handle(handle);
                    }
                    if let Some(handle) = body_parameters.collider_handle {
                        framework.physics.remove_collider_by_handle(handle);
                    }
                }
                self.children_list_mut().remove(idx);
                return true;
            }
            return object.delete_child(framework, name);
        }
        false
    }

    fn build_object_rigid_body(
        &mut self,
        framework: &mut Framework,
        body_type: Option<BodyType>,
        custom_render_collider: Option<RenderColliderType>,
        mass: f32,
        membership_groups: Option<CollisionGroups>,
        filter_groups: Option<CollisionGroups>,
    ) {
        match body_type {
            Some(body_type) => {
                let mut body_parameters = framework.physics.new_rigid_body(
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
                    framework.physics.remove_rigid_body(&mut body);
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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

impl Into<String> for ObjectGroup {
    fn into(self) -> String {
        self.0
    }
}

impl From<String> for ObjectGroup {
    fn from(value: String) -> Self {
        ObjectGroup(value)
    }
}

impl ObjectGroup {
    pub fn as_raw(&self) -> &str {
        &self.0
    }
}
