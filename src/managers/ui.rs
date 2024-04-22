use egui_glium::egui_winit::egui::{epaint::ahash::HashMap, Ui};

use super::systems;

pub fn draw_inspector(ui: &mut Ui, fps: &usize, ui_state: &mut UiState) {
    dbg!(&ui_state);
    ui.label(format!("fps: {}", fps));
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
                        });
                }
            }
        });
    }
    ui.separator();

    let selected_object = &ui_state.selected_inspector_object;
    if let Some(selected_object) = selected_object {
        match systems::get_system_mut_with_id(&selected_object.current_selected_object_system) {
            Some(system) => {
                match system.find_object_mut(&selected_object.current_selected_object_name) {
                    Some(object) => {
                        ui.heading(format!("Object {}", object.name()));
                        ui.label(format!("Type: {}", object.object_type()));
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
    selected_inspector_object: Option<SelectedInspectorObject>
}

#[derive(Default, Debug)]
pub struct SelectedInspectorObject {
    current_selected_object_system: String,
    current_selected_object_name: String,
}
