use egui_glium::egui_winit::egui::{self, TextEdit, Ui};
use glam::Vec3;

use crate::framework::{set_debug_mode, DebugMode};

use super::systems;

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
                            input_postition: None,
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
                        let transform = object.local_transform();

                        ui.heading(format!("object '{}'", object.name()));
                        ui.label(format!("type: {}", object.object_type()));

                        ui.label("local position:");
                        ui.horizontal(|ui| {
                            match selected_object.input_postition {
                                Some(ref mut input_pos) => {
                                    ui.label("x:");
                                    ui.add_sized(egui::vec2(70.0, 20.0), TextEdit::singleline(&mut input_pos[0]));
                                    ui.label("y:");
                                    ui.add_sized(egui::vec2(70.0, 20.0), TextEdit::singleline(&mut input_pos[1]));
                                    ui.label("z:");
                                    ui.add_sized(egui::vec2(70.0, 20.0), TextEdit::singleline(&mut input_pos[2]));

                                    if ui.button("done").clicked() {
                                        let x = input_pos[0].parse::<f32>();
                                        if let Ok(x) = x {
                                            let y = input_pos[1].parse::<f32>();
                                            if let Ok(y) = y {
                                                let z = input_pos[2].parse::<f32>();
                                                if let Ok(z) = z {
                                                    object.set_position(Vec3::new(x, y, z), true);
                                                }
                                            }
                                        }
                                    };
                                    if ui.button("cancel").clicked() {
                                        selected_object.input_postition = None;
                                    };
                                },
                                None => {
                                    let position = transform.position;
                                    let mut pos_array = [position.x.to_string(), position.y.to_string(), position.z.to_string()];
                                    ui.label("x:");
                                    ui.add_sized(egui::vec2(70.0, 20.0), TextEdit::singleline(&mut pos_array[0]));
                                    ui.label("y:");
                                    ui.add_sized(egui::vec2(70.0, 20.0), TextEdit::singleline(&mut pos_array[1]));
                                    ui.label("z:");
                                    ui.add_sized(egui::vec2(70.0, 20.0), TextEdit::singleline(&mut pos_array[2]));

                                    if ui.button("edit").clicked() {
                                        selected_object.input_postition = Some(pos_array.clone());
                                    };
                                },
                            }

                        });
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
    selected_inspector_object: Option<SelectedInspectorObject>
}

#[derive(Default, Debug)]
pub struct SelectedInspectorObject {
    current_selected_object_system: String,
    current_selected_object_name: String,
    input_postition: Option<[String; 3]>,
}

fn handle_full_debug_checkbox_value(full_debug: bool) {
    if full_debug {
        set_debug_mode(DebugMode::Full)
    } else {
        set_debug_mode(DebugMode::ShowFps)
    }
}
