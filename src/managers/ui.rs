use std::collections::HashMap;
use egui_glium::egui_winit::egui::{self, Button, ComboBox, Context, Label, TextEdit, Ui, Window};
use glam::{Vec2, Vec3};
use crate::framework::{DebugMode, Framework};
use super::{debugger, physics::RenderColliderType, systems};

#[derive(Default)]
pub struct UiManager {
    windows: HashMap<String, UiManagerWindow>,
}

#[derive(Clone)]
pub enum WidgetData {
    Button(String),
    Label(String),
    Horizontal,
    Vertical,
}

impl Default for WidgetData {
    fn default() -> Self {
        Self::Label(String::new())
    }
}

pub struct UiManagerWindow {
    position: Vec2,
    size: Option<Vec2>,
    widgets: Vec<Widget>,
    transparent: bool
}

#[derive(Default, Clone)]
pub struct Widget {
    id: String,
    size: Vec2,
    widget_data: WidgetData,
    children: Vec<Widget>,
    left_clicked: bool,
    right_clicked: bool,
    double_clicked: bool,
    hovered: bool,
    dragged: bool,
    changed: bool,
}

impl UiManager {
    pub fn render(&mut self, ctx: &Context) {
        for (id, manager_window) in self.windows.iter_mut() {
            let window = match manager_window.transparent {
                true => {
                    Window::new(id)
                        .frame(egui::Frame::none())
                        .title_bar(false)
                        .resizable(false)
                        .scroll(false)
                },
                false => Window::new(id),
            };
            window.show(ctx, |ui| {
                for widget in &mut manager_window.widgets {
                    Self::render_widget(ctx, ui, widget);
                }
            });
        }
    }

    fn render_widget(ctx: &Context, ui: &mut Ui, widget: &mut Widget) {
        let size = egui::Vec2::new(widget.size.x, widget.size.y);
        let egui_widget = match &widget.widget_data {
            WidgetData::Button(contents) => ui.add_sized(size, Button::new(contents)),
            WidgetData::Label(contents) => ui.add_sized(size, Label::new(contents)),
            WidgetData::Horizontal => ui.horizontal(|ui| {
                for widget in &mut widget.children {
                    Self::render_widget(ctx, ui, widget);
                }
            }).response,
            WidgetData::Vertical => ui.vertical(|ui| {
                for widget in &mut widget.children {
                    Self::render_widget(ctx, ui, widget);
                }
            }).response,
        };

        widget.left_clicked = egui_widget.clicked();
        widget.right_clicked = egui_widget.secondary_clicked();
        widget.hovered = egui_widget.hovered();
        widget.dragged = egui_widget.dragged();
        widget.changed = egui_widget.changed();
        widget.double_clicked = egui_widget.double_clicked();
    }

    fn add_child(function_name: &str, target_widget: &str, current_widget: &mut Widget, widget_to_add: Widget, widget_to_add_id: &str) -> bool {
        for widget in &mut current_widget.children {
            let id = &widget.id;
            if id == target_widget {
                for i in &widget.children {
                    if i.id == widget_to_add_id {
                        debugger::error(
                            &format!(
                                "{} error!\nChild with id '{}' already exists in the children list of the widget with id '{}'", function_name, widget_to_add_id, id
                            )
                        );

                        return false
                    }
                }

                widget.children.push(widget_to_add);
                return true
            } 

            if Self::add_child(function_name, target_widget, widget, widget_to_add.clone(), widget_to_add_id) == true {
                return true
            }
        }

        false
    }
    // exists to write less in functions like add_button, add_label, ...
    fn add_widget(&mut self, function_name: &str, window_id: &str, widget_id: &str, widget: Widget, parent: Option<&str>) {
        match self.windows.get_mut(window_id) {
            Some(window) => {
                match parent {
                    Some(parent) => {
                        for child in &mut window.widgets {
                            let child_id = &child.id;
                            println!("{}", child_id);
                            if child_id == parent {
                                for i in &widget.children {
                                    if i.id == widget_id {
                                        debugger::error(
                                            &format!(
                                                "{} error!\nChild with id '{}' already exists in the children list of the widget with id '{}'", 
                                                function_name, widget_id, child_id
                                            )
                                        );
                                        return
                                    }
                                }

                                println!("added!");
                                child.children.push(widget.clone());
                                return
                            }

                            Self::add_child(function_name, parent, child, widget.clone(), widget_id);
                        }
                    },
                    None => {
                        for i in &window.widgets {
                            if i.id == widget_id {
                                debugger::error(
                                    &format!(
                                        "{} error!\nWidget with id '{}' already exists in the window with id '{}'", function_name, widget_id, window_id
                                    )
                                );
                                return
                            }
                        }

                        window.widgets.push(widget);
                    },
                }
            },
            None => {
                debugger::error(
                    &format!(
                        "{} error!\nFailed to get the window with id '{}' to add a widget with id '{}'", function_name, window_id, widget_id
                    )
                )
            },
        }
    }

    pub fn new_window(&mut self, id: &str, transparent: bool) {
        if self.windows.contains_key(id) == true {
            debugger::error(&format!("new_window error!\nWindow with id '{}' already exists!", id));
            return;
        }
        self.windows.insert(id.into(), UiManagerWindow {
            position: Vec2::ZERO,
            size: None,
            widgets: Vec::new(),
            transparent,
        });
    }

    pub fn add_button(&mut self, window_id: &str, widget_id: &str, contents: &str, size: Vec2, parent: Option<&str>) {
        let widget = Widget {
            id: widget_id.into(),
            size,
            widget_data: WidgetData::Button(contents.into()),
            children: Vec::new(),
            ..Default::default()
        };

        self.add_widget("add_button", window_id, widget_id, widget, parent)
    }

    pub fn add_label(&mut self, window_id: &str, widget_id: &str, contents: &str, parent: Option<&str>) {
        let widget = Widget {
            id: widget_id.into(),
            widget_data: WidgetData::Label(contents.into()),
            children: Vec::new(),
            ..Default::default()
        };

        self.add_widget("add_label", window_id, widget_id, widget, parent)
    }

    pub fn add_horizontal(&mut self, window_id: &str, widget_id: &str, parent: Option<&str>) {
        let widget = Widget {
            id: widget_id.into(),
            widget_data: WidgetData::Horizontal,
            children: Vec::new(),
            ..Default::default()
        };

        self.add_widget("add_horizontal", window_id, widget_id, widget, parent)
    }

    pub fn add_vertical(&mut self, window_id: &str, widget_id: &str, size: Vec2, parent: Option<&str>) {
        let widget = Widget {
            id: widget_id.into(),
            size,
            widget_data: WidgetData::Vertical,
            children: Vec::new(),
            ..Default::default()
        };

        self.add_widget("add_vertical", window_id, widget_id, widget, parent)
    }
}

// inspector
pub fn draw_inspector(framework: &mut Framework, ui: &mut Ui, fps: &usize, ui_state: &mut UiState) {
    ui.label(format!("fps: {}", fps));
    ui.checkbox(&mut ui_state.full_debug_checkbox_val, "full debug");
    handle_full_debug_checkbox_value(framework, ui_state.full_debug_checkbox_val);

    ui.separator();

    ui.heading("systems:");
    for system in systems::get_systems_iter() {
        let system_id = system.system_id();

        ui.collapsing(system_id, |ui| {
            for object in system.objects_list() {
                let object_name = object.name();
                if ui.small_button(object_name).clicked() {
                    ui_state.selected_inspector_object = Some(SelectedInspectorObject {
                        current_selected_object_system: system_id.into(),
                        current_selected_object_name: object_name.into(),
                        //input_postition: None,
                        position: Vec3Inspector::default(),
                        rotation: Vec3Inspector::default(),
                        scale: Vec3Inspector::default(),
                        render_collider: None,
                        render_collider_size: Vec3Inspector {
                            input_value: Some(["".into(), "".into(), "".into()]),
                        },
                        cancel_collider: false,
                        new_group_name: String::new(),
                    });
                }
            }
        });
    }

    ui.separator();

    let selected_object = &mut ui_state.selected_inspector_object;
    if let Some(selected_object) = selected_object {
        match systems::get_system_mut_with_id(&selected_object.current_selected_object_system) {
            Some(system) => {
                match system.find_object_mut(&selected_object.current_selected_object_name) {
                    Some(object) => {
                        ui.heading(format!("object '{}'", object.name()));
                        ui.label(format!("type: {}", object.object_type()));
                        ui.separator();
                        ui.collapsing("children", |ui| {
                            for object in object.children_list() {
                                if ui.button(object.name()).clicked() {
                                    selected_object.current_selected_object_name =
                                        object.name().into();
                                    return;
                                }
                            }
                        });

                        ui.collapsing("groups", |ui| {
                            for group in object.groups_list().clone() {
                                ui.horizontal(|ui| {
                                    ui.label(group.as_raw());
                                    if ui.button("remove").clicked() {
                                        object.remove_from_group(group.as_raw());
                                    }
                                });
                            }
                            ui.horizontal(|ui| {
                                ui.label("group name:");
                                ui.text_edit_singleline(&mut selected_object.new_group_name);
                                if ui.button("add group").clicked() {
                                    object.add_to_group(&selected_object.new_group_name);
                                }
                            });
                        });

                        ui.label("local position:");
                        if let Some(pos) = draw_vec3_editor_inspector(
                            ui,
                            &mut selected_object.position,
                            &object.local_transform().position,
                            true,
                        ) {
                            object.set_position(framework, pos, true);
                        }
                        ui.label("local rotation:");
                        if let Some(rot) = draw_vec3_editor_inspector(
                            ui,
                            &mut selected_object.rotation,
                            &object.local_transform().rotation,
                            true,
                        ) {
                            object.set_rotation(framework, rot, true);
                        }
                        ui.label("local scale:");
                        if let Some(sc) = draw_vec3_editor_inspector(
                            ui,
                            &mut selected_object.scale,
                            &object.local_transform().scale,
                            true,
                        ) {
                            object.set_scale(sc);
                        }

                        // collider
                        if selected_object.cancel_collider {
                            selected_object.render_collider = None;
                            selected_object.cancel_collider = false;
                            return;
                        }
                        //dbg!(&selected_object.render_collider, &selected_object.cancel_collider);
                        if let Some(collider) = &mut selected_object.render_collider {
                            ui.separator();
                            ui.label("render collider settings:");
                            ComboBox::from_label("collider type")
                                .selected_text(
                                    format!("{:?}", collider.collider_type).to_lowercase(),
                                )
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut collider.collider_type,
                                        InspectorRenderColliderType::Cuboid,
                                        "cuboid",
                                    );
                                    ui.selectable_value(
                                        &mut collider.collider_type,
                                        InspectorRenderColliderType::Ball,
                                        "ball",
                                    );
                                    ui.selectable_value(
                                        &mut collider.collider_type,
                                        InspectorRenderColliderType::Capsule,
                                        "capsule",
                                    );
                                    ui.selectable_value(
                                        &mut collider.collider_type,
                                        InspectorRenderColliderType::Cylinder,
                                        "cylinder",
                                    );
                                });
                            if let Some(ref mut input_value) =
                                &mut selected_object.render_collider_size.input_value
                            {
                                match &collider.collider_type {
                                    InspectorRenderColliderType::Ball => {
                                        ui.horizontal(|ui| {
                                            ui.label("radius:");
                                            ui.text_edit_singleline(&mut input_value[0]);
                                        });
                                    }
                                    InspectorRenderColliderType::Cuboid => {
                                        draw_vec3_editor_inspector(
                                            ui,
                                            &mut selected_object.render_collider_size,
                                            &Vec3::new(1.0, 1.0, 1.0),
                                            false,
                                        );
                                    }
                                    InspectorRenderColliderType::Capsule => {
                                        ui.horizontal(|ui| {
                                            ui.label("radius:");
                                            ui.text_edit_singleline(&mut input_value[0]);
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("height:");
                                            ui.text_edit_singleline(&mut input_value[1]);
                                        });
                                    }
                                    InspectorRenderColliderType::Cylinder => {
                                        ui.horizontal(|ui| {
                                            ui.label("radius:");
                                            ui.text_edit_singleline(&mut input_value[0]);
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("height:");
                                            ui.text_edit_singleline(&mut input_value[1]);
                                        });
                                    }
                                }
                            }
                            let render_collider_size = &mut selected_object.render_collider_size;
                            ui.horizontal(|ui| {
                                if ui.button("cancel").clicked() {
                                    selected_object.cancel_collider = true;
                                }

                                if ui.button("done").clicked() {
                                    let mut trigger = false;
                                    if object.object_type() == "Trigger" {
                                        trigger = true;
                                    };

                                    if let Some(ref input_value) = render_collider_size.input_value
                                    {
                                        let x = input_value[0].parse::<f32>();
                                        if let Ok(x) = x {
                                            let y = input_value[1].parse::<f32>();
                                            if let Ok(y) = y {
                                                let z = input_value[2].parse::<f32>();
                                                if let Ok(z) = z {
                                                    match collider.clone().collider_type {
                                                        InspectorRenderColliderType::Ball => {
                                                            object.build_object_rigid_body(
                                                                framework,
                                                                None,
                                                                Some(RenderColliderType::Ball(
                                                                    None, None, x, trigger,
                                                                )),
                                                                1.0,
                                                                None,
                                                                None,
                                                            );
                                                        }
                                                        InspectorRenderColliderType::Cuboid => {
                                                            object.build_object_rigid_body(
                                                                framework,
                                                                None,
                                                                Some(RenderColliderType::Cuboid(
                                                                    None, None, x, y, z, trigger,
                                                                )),
                                                                1.0,
                                                                None,
                                                                None,
                                                            );
                                                        }
                                                        InspectorRenderColliderType::Capsule => {
                                                            object.build_object_rigid_body(
                                                                framework,
                                                                None,
                                                                Some(RenderColliderType::Capsule(
                                                                    None, None, x, y, trigger,
                                                                )),
                                                                1.0,
                                                                None,
                                                                None,
                                                            );
                                                        }
                                                        InspectorRenderColliderType::Cylinder => {
                                                            object.build_object_rigid_body(
                                                                framework,
                                                                None,
                                                                Some(RenderColliderType::Cylinder(
                                                                    None, None, x, y, trigger,
                                                                )),
                                                                1.0,
                                                                None,
                                                                None,
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                        } else {
                            if ui.button("render collider settings").clicked() {
                                match selected_object.render_collider {
                                    Some(_) => selected_object.render_collider = None,
                                    None => {
                                        selected_object.render_collider =
                                            Some(InspectorRenderCollider {
                                                collider_type: InspectorRenderColliderType::Cuboid,
                                            })
                                    }
                                }
                            }
                        }

                        ui.separator();
                        object.inspector_ui(framework, ui);
                    }
                    None => {
                        ui.heading(format!(
                            "Failed to get an object with name {} in the system with id {}",
                            selected_object.current_selected_object_name,
                            selected_object.current_selected_object_system
                        ));
                    }
                }
            }
            None => {
                ui.heading(format!(
                    "Failed to get a system with id {}",
                    selected_object.current_selected_object_system
                ));
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct UiState {
    full_debug_checkbox_val: bool,
    selected_inspector_object: Option<SelectedInspectorObject>,
}

#[derive(Default, Debug)]
pub struct SelectedInspectorObject {
    current_selected_object_system: String,
    current_selected_object_name: String,
    position: Vec3Inspector,
    rotation: Vec3Inspector,
    scale: Vec3Inspector,
    render_collider: Option<InspectorRenderCollider>,
    render_collider_size: Vec3Inspector,
    cancel_collider: bool,
    new_group_name: String, //input_postition: Option<[String; 3]>,
}

#[derive(Debug, Default)]
pub struct Vec3Inspector {
    input_value: Option<[String; 3]>,
}

#[derive(Debug, Clone)]
struct InspectorRenderCollider {
    collider_type: InspectorRenderColliderType,
}

#[derive(Debug, PartialEq, Clone)]
enum InspectorRenderColliderType {
    /// radius
    Ball, //(f32),
    /// half-size
    Cuboid, //(Vec3),
    /// radius, height
    Capsule, //(f32, f32),
    /// radius, height
    Cylinder, //(f32, f32),
}

fn handle_full_debug_checkbox_value(framework: &mut Framework, full_debug: bool) {
    if full_debug {
        framework.set_debug_mode(DebugMode::Full)
    } else {
        framework.set_debug_mode(DebugMode::ShowFps)
    }
}

pub fn draw_vec3_editor_inspector(
    ui: &mut Ui,
    vec3: &mut Vec3Inspector,
    object_val: &Vec3,
    show_default_buttons: bool,
) -> Option<Vec3> {
    let mut return_val: Option<Vec3> = None;
    ui.horizontal(|ui| {
        match vec3.input_value {
            Some(ref mut input_val) => {
                ui.label("x:");
                ui.add_sized(
                    egui::vec2(70.0, 20.0),
                    TextEdit::singleline(&mut input_val[0]),
                );
                ui.label("y:");
                ui.add_sized(
                    egui::vec2(70.0, 20.0),
                    TextEdit::singleline(&mut input_val[1]),
                );
                ui.label("z:");
                ui.add_sized(
                    egui::vec2(70.0, 20.0),
                    TextEdit::singleline(&mut input_val[2]),
                );

                if show_default_buttons {
                    if ui.button("done").clicked() {
                        let x = input_val[0].parse::<f32>();
                        if let Ok(x) = x {
                            let y = input_val[1].parse::<f32>();
                            if let Ok(y) = y {
                                let z = input_val[2].parse::<f32>();
                                if let Ok(z) = z {
                                    return_val = Some(Vec3::new(x, y, z));
                                    vec3.input_value = None;
                                    return;
                                }
                            }
                        }
                    };
                    if ui.button("cancel").clicked() {
                        vec3.input_value = None;
                    };
                }
            }
            None => {
                let val = object_val.clone();
                let mut val_array = [val.x.to_string(), val.y.to_string(), val.z.to_string()];
                ui.label("x:");
                ui.add_sized(
                    egui::vec2(70.0, 20.0),
                    TextEdit::singleline(&mut val_array[0]),
                );
                ui.label("y:");
                ui.add_sized(
                    egui::vec2(70.0, 20.0),
                    TextEdit::singleline(&mut val_array[1]),
                );
                ui.label("z:");
                ui.add_sized(
                    egui::vec2(70.0, 20.0),
                    TextEdit::singleline(&mut val_array[2]),
                );

                if show_default_buttons && ui.button("edit").clicked() {
                    vec3.input_value = Some(val_array.clone());
                };
            }
        }
        return_val = None;
    });

    return_val
}

