use glam::Vec2;
use glium::{glutin::surface::WindowSurface, Display};
//use recast_rs::{util, Heightfield, CompactHeightfield, NoRegions, PolyMesh, ContourBuildFlags, ContourSet};
use super::{gen_object_id, Object, ObjectGroup, Transform};
use crate::{framework::Framework, managers::{
    debugger,
    navigation::{self, NavMeshDimensions},
    physics::ObjectBodyParameters,
}};

//#[derive(Debug)]
pub struct NavigationGround {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    id: u128,
    groups: Vec<ObjectGroup>,
    dimensions: NavMeshDimensions,
    //grid: Grid<Option<()>>
}

impl NavigationGround {
    pub fn new(name: &str, area_size: Vec2) -> Self {
        Self {
            name: name.into(),
            transform: Transform::default(),
            parent_transform: None,
            children: vec![],
            id: gen_object_id(),
            groups: vec![],
            dimensions: NavMeshDimensions::new(Vec2::new(0.0, 0.0), area_size),
            //grid: Grid::new(x_cells_count, z_cells_count, Some(())),
        }
    }
}

impl Object for NavigationGround {
    fn start(&mut self) {}

    fn update(&mut self, framework: &mut Framework) {
        let pos = self.global_transform().position;
        self.dimensions.set_position(Vec2::new(pos.x, pos.z));

        framework.navigation.add_navmesh(*self.object_id(), self.dimensions.clone());
    }

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
        debugger::error(
            "NavMesh object error!\ncan't use set_body_parameters in this type of objects",
        );
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

    fn inspector_ui(&mut self, ui: &mut egui_glium::egui_winit::egui::Ui) {
        ui.heading("NavigationGround parameters");
        ui.label("this object type is made specifically for servers so there's noting to change here ._.");
    }

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Object")
            .field("name", &self.name())
            .field("object_type", &self.object_type())
            .field("children", &self.children_list())
            .finish()
    }

    fn render(
        &mut self,
        _display: &Display<WindowSurface>,
        _target: &mut glium::Frame,
        _cascades: &crate::managers::render::Cascades,
        _shadow_textures: &crate::managers::render::ShadowTextures,
    ) {
    }

    fn shadow_render(
        &mut self,
        _view_proj: &glam::Mat4,
        _display: &Display<WindowSurface>,
        _target: &mut glium::framebuffer::SimpleFrameBuffer,
    ) {
    }

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

    fn update_transform(&mut self) {
        if let Some(parameters) = self.body_parameters() {
            if let None = parameters.rigid_body_handle {
                return;
            }

            let position_and_rotation_option =
                crate::managers::physics::get_body_transformations(parameters);

            if let Some((pos, rot)) = position_and_rotation_option {
                self.set_position(pos, false);
                self.set_rotation(rot, false);
            }
        }
    }

    fn debug_render(&self) {
        // Adding collider to render manager's render colliders list if debug mode != None
        match crate::framework::get_debug_mode() {
            crate::framework::DebugMode::Full => {
                if let Some(body) = self.body_parameters() {
                    if let Some(mut render_collider) = body.render_collider_type {
                        let transform = self.global_transform();
                        render_collider.set_transform(transform.position, transform.rotation);
                        crate::managers::render::add_collider_to_draw(render_collider);
                    }
                }

                self.children_list()
                    .iter()
                    .for_each(|child| child.debug_render());
            }
            _ => (),
        }
    }

    fn set_position(&mut self, position: glam::Vec3, set_rigid_body_position: bool) {
        let mut transform = self.local_transform();
        transform.position = position;
        self.set_local_transform(transform);

        if let Some(parameters) = self.body_parameters() {
            if set_rigid_body_position == true {
                crate::managers::physics::set_body_position(parameters, position);
            }
        }
    }

    fn set_rotation(&mut self, rotation: glam::Vec3, set_rigid_body_rotation: bool) {
        let mut transform = self.local_transform();
        transform.rotation = rotation;
        self.set_local_transform(transform);

        if let Some(parameters) = self.body_parameters() {
            if set_rigid_body_rotation == true {
                crate::managers::physics::set_body_rotation(parameters, rotation);
            }
        }
    }

    fn set_scale(&mut self, scale: glam::Vec3) {
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
        body_type: Option<crate::managers::physics::BodyType>,
        custom_render_collider: Option<crate::managers::physics::RenderColliderType>,
        mass: f32,
        membership_groups: Option<crate::managers::physics::CollisionGroups>,
        filter_groups: Option<crate::managers::physics::CollisionGroups>,
    ) {
        match body_type {
            Some(body_type) => {
                let mut body_parameters = crate::managers::physics::new_rigid_body(
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
                    crate::managers::physics::remove_rigid_body(&mut body);
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
        crate::managers::systems::register_object_id_groups(*self.object_id(), self.groups_list());
    }

    fn remove_from_group(&mut self, group_name: &str) {
        self.groups_list().retain(|group| group_name != group.0);
        crate::managers::systems::register_object_id_groups(*self.object_id(), self.groups_list());
    }
}

impl std::fmt::Debug for NavigationGround {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NavigationGround")
            .field("name", &self.name())
            .field("object_type", &self.object_type())
            .field("children", &self.children_list())
            .finish()
    }
}

enum CurrentAxis {
    X,
    Y,
    Z,
}

#[derive(Debug)]
pub enum NavMeshError {
    HeightmapError,
    RasterizeError,
    PolyMeshError,
}
