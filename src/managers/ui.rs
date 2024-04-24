use egui_glium::egui_winit::egui::{self, ComboBox, TextEdit, Ui};
use glam::Vec3;

use crate::framework::{set_debug_mode, DebugMode};

use super::{physics::RenderColliderType, systems};


// inspector
pub fn draw_inspector(ui: &mut Ui, fps: &usize, ui_state: &mut UiState) {
    ui.label(format!("fps: {}", fps));
    ui.checkbox(&mut ui_state.full_debug_checkbox_val, "full debug");
    handle_full_debug_checkbox_value(ui_state.full_debug_checkbox_val);

    ui.separator();

    ui.heading("systems:");
    for system in systems::get_systems_iter() {
        let system_id = system.system_id();

        ui.collapsing(system_id, |ui| {
            for object in system.objects_list() {
                let object_name = object.name();
                if ui.small_button(object_name).clicked() {
                    ui_state.selected_inspector_object = 
                        Some(SelectedInspectorObject {
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
                        //let transform = object.local_transform();

                        ui.heading(format!("object '{}'", object.name()));
                        ui.label(format!("type: {}", object.object_type()));

                        ui.label("local position:");
                        if let Some(pos) = draw_vec3_editor_inspector(ui, &mut selected_object.position, &object.local_transform().position, true) {
                            object.set_position(pos, true);
                        }
                        ui.label("local rotation:");
                        if let Some(rot) = draw_vec3_editor_inspector(ui, &mut selected_object.rotation, &object.local_transform().rotation, true) {
                            object.set_rotation(rot, true);
                        }
                        ui.label("local scale:");
                        if let Some(sc) = draw_vec3_editor_inspector(ui, &mut selected_object.scale, &object.local_transform().scale, true) {
                            object.set_scale(sc);
                        }


                        // collider
                        if selected_object.cancel_collider {
                            selected_object.render_collider = None;
                            selected_object.cancel_collider = false;
                            return;
                        }
                        dbg!(&selected_object.render_collider, &selected_object.cancel_collider);
                        if let Some(collider) = &mut selected_object.render_collider {
                            ui.separator();
                            ui.label("render collider settings:");
                            ComboBox::from_label("collider type")
                                .selected_text(format!("{:?}", collider.collider_type).to_lowercase())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut collider.collider_type, InspectorRenderColliderType::Cuboid, "cuboid");
                                    ui.selectable_value(&mut collider.collider_type, InspectorRenderColliderType::Ball, "ball");
                                    ui.selectable_value(&mut collider.collider_type, InspectorRenderColliderType::Capsule, "capsule");
                                    ui.selectable_value(&mut collider.collider_type, InspectorRenderColliderType::Cylinder, "cylinder");
                                }
                            );
                            if let Some(ref mut input_value) = &mut selected_object.render_collider_size.input_value {
                                match &collider.collider_type {
                                    InspectorRenderColliderType::Ball => {
                                        ui.horizontal(|ui| {
                                            ui.label("radius:");
                                            ui.text_edit_singleline(&mut input_value[0]);
                                        });
                                    },
                                    InspectorRenderColliderType::Cuboid => {
                                        draw_vec3_editor_inspector(ui, &mut selected_object.render_collider_size, &Vec3::new(1.0, 1.0, 1.0), false);
                                    },
                                    InspectorRenderColliderType::Capsule => {
                                        ui.horizontal(|ui| {
                                            ui.label("radius:");
                                            ui.text_edit_singleline(&mut input_value[0]);
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("height:");
                                            ui.text_edit_singleline(&mut input_value[1]);
                                        });
                                    },
                                    InspectorRenderColliderType::Cylinder => {
                                        ui.horizontal(|ui| {
                                            ui.label("radius:");
                                            ui.text_edit_singleline(&mut input_value[0]);
                                        });
                                        ui.horizontal(|ui| {
                                            ui.label("height:");
                                            ui.text_edit_singleline(&mut input_value[1]);
                                        });
                                    },
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

                                    if let Some(ref input_value) = render_collider_size.input_value {
                                        let x = input_value[0].parse::<f32>();
                                        if let Ok(x) = x {
                                            let y = input_value[1].parse::<f32>();
                                            if let Ok(y) = y {
                                                let z = input_value[2].parse::<f32>();
                                                if let Ok(z) = z {
                                                    match collider.clone().collider_type {
                                                        InspectorRenderColliderType::Ball => {
                                                            object.build_object_rigid_body(
                                                                None, 
                                                                Some(RenderColliderType::Ball(None, None, x, trigger)),
                                                                1.0, None, None
                                                            );
                                                        },
                                                        InspectorRenderColliderType::Cuboid => {
                                                            object.build_object_rigid_body(
                                                                None, 
                                                                Some(RenderColliderType::Cuboid(None, None, x, y, z, trigger)),
                                                                1.0, None, None
                                                            );
                                                        },
                                                        InspectorRenderColliderType::Capsule => {
                                                            object.build_object_rigid_body(
                                                                None, 
                                                                Some(RenderColliderType::Capsule(None, None, x, y, trigger)),
                                                                1.0, None, None
                                                            );
                                                        },
                                                        InspectorRenderColliderType::Cylinder => {
                                                            object.build_object_rigid_body(
                                                                None, 
                                                                Some(RenderColliderType::Cylinder(None, None, x, y, trigger)),
                                                                1.0, None, None
                                                            );
                                                        },
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
                                    None => selected_object.render_collider = 
                                        Some(InspectorRenderCollider {
                                            collider_type: InspectorRenderColliderType::Cuboid,
                                        }),
                                }
                            }
                        }

                        ui.separator();
                        object.inspector_ui(ui);
                    },
                    None => {
                        ui.heading(format!("Failed to get an object with name {} in the system with id {}", 
                                selected_object.current_selected_object_name,
                                selected_object.current_selected_object_system));
                    },
                }
            },
            None => {
                ui.heading(format!("Failed to get a system with id {}", selected_object.current_selected_object_system));
            },
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
    cancel_collider: bool
    //input_postition: Option<[String; 3]>,
}

#[derive(Debug, Default)]
struct Vec3Inspector {
    input_value: Option<[String; 3]>,
}

#[derive(Debug, Clone)]
struct InspectorRenderCollider {
    collider_type: InspectorRenderColliderType,
}

#[derive(Debug, PartialEq, Clone)]
enum InspectorRenderColliderType {
    /// radius
    Ball,//(f32),
    /// half-size
    Cuboid,//(Vec3),
    /// radius, height
    Capsule,//(f32, f32),
    /// radius, height
    Cylinder,//(f32, f32),
}

fn handle_full_debug_checkbox_value(full_debug: bool) {
    if full_debug {
        set_debug_mode(DebugMode::Full)
    } else {
        set_debug_mode(DebugMode::ShowFps)
    }
}

fn draw_vec3_editor_inspector(ui: &mut Ui, vec3: &mut Vec3Inspector, object_val: &Vec3, show_default_buttons: bool) -> Option<Vec3> {
    let mut return_val: Option<Vec3> = None;
    ui.horizontal(|ui| {
        match vec3.input_value {
            Some(ref mut input_val) => {
                ui.label("x:");
                ui.add_sized(egui::vec2(70.0, 20.0), TextEdit::singleline(&mut input_val[0]));
                ui.label("y:");
                ui.add_sized(egui::vec2(70.0, 20.0), TextEdit::singleline(&mut input_val[1]));
                ui.label("z:");
                ui.add_sized(egui::vec2(70.0, 20.0), TextEdit::singleline(&mut input_val[2]));

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
            },
            None => {
                let val = object_val.clone();
                let mut val_array = [val.x.to_string(), val.y.to_string(), val.z.to_string()];
                ui.label("x:");
                ui.add_sized(egui::vec2(70.0, 20.0), TextEdit::singleline(&mut val_array[0]));
                ui.label("y:");
                ui.add_sized(egui::vec2(70.0, 20.0), TextEdit::singleline(&mut val_array[1]));
                ui.label("z:");
                ui.add_sized(egui::vec2(70.0, 20.0), TextEdit::singleline(&mut val_array[2]));

                if show_default_buttons && ui.button("edit").clicked() {
                    vec3.input_value = Some(val_array.clone());
                };
            },
        }
        return_val = None;
    });

    return_val
}
