use glam::Vec2;
use glium::{glutin::surface::WindowSurface, Display};
//use recast_rs::{util, Heightfield, CompactHeightfield, NoRegions, PolyMesh, ContourBuildFlags, ContourSet};
use super::{gen_object_id, Object, ObjectGroup, Transform};
use crate::{framework::Framework, managers::{
    debugger,
    navigation::NavMeshDimensions,
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
        _: &Display<WindowSurface>,
        _: &mut glium::Frame,
        _: &crate::managers::render::Cascades,
        _: &crate::managers::render::ShadowTextures,
    ) {
    }

    fn shadow_render(
        &mut self,
        _: &glam::Mat4,
        _: &Display<WindowSurface>,
        _: &mut glium::framebuffer::SimpleFrameBuffer,
    ) {}

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

    fn add_child(&mut self, mut object: Box<dyn Object>) {
        object.set_parent_transform(self.global_transform());
        dbg!(object.object_id());
        self.children_list_mut().push(object);
        self.children_list_mut().last_mut().unwrap().start();
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
