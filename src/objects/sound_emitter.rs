use super::{gen_object_id, Object, ObjectGroup, Transform};
use crate::{
    assets::sound_asset::SoundAsset,
    managers::{debugger::warn, physics::ObjectBodyParameters},
};
use core::f32;
use ez_al::{SoundError, SoundSource, SoundSourceType};
use glam::Vec3;
use std::fmt::Debug;

pub struct SoundEmitter {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    body: Option<ObjectBodyParameters>,
    id: u128,
    groups: Vec<ObjectGroup>,
    pub source_type: SoundSourceType,
    pub source: SoundSource,
    max_distance_inspector: Option<String>,
}

impl SoundEmitter {
    pub fn new(
        name: &str,
        asset: &SoundAsset,
        emitter_type: SoundSourceType,
    ) -> Result<SoundEmitter, SoundError> {
        let source = SoundSource::new(&asset.wav, emitter_type.clone());
        match source {
            Ok(source) => Ok(SoundEmitter {
                name: name.to_string(),
                transform: Transform::default(),
                parent_transform: None,
                children: vec![],
                body: None,
                id: gen_object_id(),
                groups: vec![],
                source_type: emitter_type,
                source,
                max_distance_inspector: None,
            }),
            Err(err) => Err(err),
        }
    }

    pub fn set_looping(&mut self, should_loop: bool) {
        self.source.set_looping(should_loop);
    }

    pub fn is_looping(&self) -> bool {
        self.source.is_looping()
    }

    pub fn play_sound(&mut self) {
        self.source.play_sound();
    }

    pub fn set_max_distance(&mut self, distance: f32) -> Result<(), SoundError> {
        match self.source_type {
            SoundSourceType::Simple => {
                warn("tried to set max distance when emitter type is simple");
                Err(SoundError::WrongSoundSourceType)
            }
            SoundSourceType::Positional => {
                let _ = self.source.set_max_distance(distance);
                Ok(())
            }
        }
    }

    pub fn get_max_distance(&mut self) -> Result<f32, SoundError> {
        match self.source_type {
            SoundSourceType::Simple => {
                warn("tried to get max distance when emitter type is simple");
                Err(SoundError::WrongSoundSourceType)
            }
            SoundSourceType::Positional => Ok(self.source.get_max_distance().unwrap()),
        }
    }

    pub fn update_sound_transforms(&mut self, sound_position: Vec3) {
        let _ = self.source.update(sound_position.into());
    }
}

impl Object for SoundEmitter {
    fn start(&mut self) {}

    fn update(&mut self) {
        self.update_sound_transforms(self.global_transform().position);
    }

    fn children_list(&self) -> &Vec<Box<dyn Object>> {
        &self.children
    }

    fn children_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.children
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }
    fn object_type(&self) -> &str {
        "SoundEmitter"
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn local_transform(&self) -> Transform {
        self.transform
    }

    fn set_local_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    fn parent_transform(&self) -> Option<Transform> {
        self.parent_transform
    }

    fn set_parent_transform(&mut self, transform: Transform) {
        self.parent_transform = Some(transform);
    }

    fn set_body_parameters(&mut self, rigid_body: Option<ObjectBodyParameters>) {
        self.body = rigid_body
    }

    fn body_parameters(&self) -> Option<ObjectBodyParameters> {
        self.body
    }

    fn object_id(&self) -> &u128 {
        &self.id
    }

    fn inspector_ui(&mut self, ui: &mut egui_glium::egui_winit::egui::Ui) {
        ui.heading("SoundEmitter parameters");

        let mut looping = self.source.is_looping();
        ui.checkbox(&mut looping, "looping");
        self.source.set_looping(looping);

        let mut set_distance_string: Option<String> = None;
        let mut cancel = false;
        ui.label(format!("source type is {:?}", self.source_type));
        if let SoundSourceType::Positional = self.source_type {
            match &mut self.max_distance_inspector {
                Some(distance) => {
                    ui.label("max distance:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(distance);
                        if ui.button("done").clicked() {
                            set_distance_string = Some(distance.to_string());
                        }
                        if ui.button("cancel").clicked() {
                            cancel = true;
                        }
                    });
                }
                None => {
                    ui.horizontal(|ui| {
                        let distance_str = self.get_max_distance().unwrap().to_string();

                        ui.label("max distance:");
                        ui.label(&distance_str);
                        if ui.button("change").clicked() {
                            self.max_distance_inspector = Some(distance_str);
                        }
                    });
                }
            }
        }
        if let Some(distance) = set_distance_string {
            if let Ok(distance) = distance.parse::<f32>() {
                let _ = self.set_max_distance(distance);
                self.max_distance_inspector = None;
            }
        }
        if cancel {
            self.max_distance_inspector = None;
        }

        if ui.button("play").clicked() {
            self.play_sound();
        }
    }

    fn groups_list(&mut self) -> &mut Vec<super::ObjectGroup> {
        &mut self.groups
    }

    fn call(&mut self, name: &str, _args: Vec<&str>) -> Option<String> {
        if name == "play" {
            self.play_sound();
        }
        None
    }
}

impl Debug for SoundEmitter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SoundEmitter")
            .field("name", &self.name)
            .field("object_type", &self.object_type())
            .field("transform", &self.transform)
            .field("parent_transform", &self.parent_transform)
            .field("children", &self.children)
            .field("emitter_type", &self.source_type)
            .field("looping", &self.is_looping())
            .finish()
    }
}
