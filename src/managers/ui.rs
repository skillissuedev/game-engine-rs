use egui_glium::egui_winit::egui::{self, TextEdit, Ui};
use glam::Vec3;

use crate::{framework::{set_debug_mode, DebugMode}, objects::Object};

use super::systems;


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
                        if let Some(pos) = draw_vec3_editor_inspector(ui, &mut selected_object.position, &object.local_transform().position) {
                            object.set_position(pos, true);
                        }
                        ui.label("local rotation:");
                        if let Some(rot) = draw_vec3_editor_inspector(ui, &mut selected_object.rotation, &object.local_transform().rotation) {
                            object.set_rotation(rot, true);
                        }
                        ui.label("local scale:");
                        if let Some(sc) = draw_vec3_editor_inspector(ui, &mut selected_object.scale, &object.local_transform().scale) {
                            object.set_scale(sc);
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
    //input_postition: Option<[String; 3]>,
}

#[derive(Debug, Default)]
struct Vec3Inspector {
    input_value: Option<[String; 3]>,
}

fn handle_full_debug_checkbox_value(full_debug: bool) {
    if full_debug {
        set_debug_mode(DebugMode::Full)
    } else {
        set_debug_mode(DebugMode::ShowFps)
    }
}

fn draw_vec3_editor_inspector(ui: &mut Ui, vec3: &mut Vec3Inspector, object_val: &Vec3) -> Option<Vec3> {
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

                if ui.button("edit").clicked() {
                    vec3.input_value = Some(val_array.clone());
                };
            },
        }
        return_val = None;
    });

    return_val
}
