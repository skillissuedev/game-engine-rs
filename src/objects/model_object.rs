use std::time::Instant;
use glam::Mat4;
use ultraviolet::Vec3;
use crate::assets::model_asset::{ModelAsset, Animation, AnimationChannelType};
use super::{Object, Transform};

#[derive(Debug)]
pub struct ModelObject {
    pub name: String,
    pub transform: Transform,
    pub nodes_transforms: Vec<NodeTransform>,
    pub parent_transform: Option<Transform>,
    pub children: Vec<Box<dyn Object>>,
    pub asset: ModelAsset,
    pub animation_settings: CurrentAnimationSettings
}

impl ModelObject {
    pub fn new(name: &str, asset: ModelAsset) -> Self {
        ModelObject {
            transform: Transform::default(),
            nodes_transforms: vec![],
            children: vec![],
            name: name.to_string(),
            parent_transform: None, asset,
            animation_settings: CurrentAnimationSettings { animation: None, looping: false, timer: None }
        }
    }
}


impl Object for ModelObject {
    fn get_object_type(&self) -> &str {
        "ModelObject"
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }


    fn start(&mut self) {
        for obj in &self.asset.objects {
            let matrix = Mat4::from_cols_array_2d(&obj.transform);
            let scale_rot_pos = matrix.to_scale_rotation_translation();
            let rotation = scale_rot_pos.1.to_euler(glam::EulerRot::XYZ);
            let position = scale_rot_pos.2;
            let scale = scale_rot_pos.0;

            let obj_transform = Transform { 
                position: Vec3 { x: position.x, y: position.y, z: position.z },
                rotation: Vec3 { x: rotation.0, y: rotation.1, z: rotation.2 },
                scale: Vec3 { x: scale.x, y: scale.y, z: scale.z }
            };

            self.nodes_transforms.push(NodeTransform { transform: obj_transform, node_id: obj.node_index });
        }
    }

    fn update(&mut self) {
        match &self.animation_settings.animation {
            None => (),
            Some(animation) => {
                let anim_settings = &self.animation_settings;
                let timer = &self.animation_settings.timer;
                let time_elapsed = timer.expect("no timer(why)").elapsed().as_secs_f32();

                for channel in &animation.channels {
                    match channel.channel_type {
                        AnimationChannelType::Translation => {
                            for node in &mut self.nodes_transforms {
                                if node.node_id == channel.node_index {
                                    let x_pos = channel.x_axis_spline.clamped_sample(time_elapsed).unwrap();
                                    let y_pos = channel.y_axis_spline.clamped_sample(time_elapsed).unwrap();
                                    let z_pos = channel.z_axis_spline.clamped_sample(time_elapsed).unwrap();
                                    node.transform.position = Vec3::new(x_pos, y_pos, z_pos);
                                }
                            }
                        },
                        AnimationChannelType::Rotation => {
                            for node in &mut self.nodes_transforms {
                                if node.node_id == channel.node_index {
                                    let x_rot = channel.x_axis_spline.clamped_sample(time_elapsed).unwrap();
                                    let y_rot = channel.y_axis_spline.clamped_sample(time_elapsed).unwrap();
                                    let z_rot = channel.z_axis_spline.clamped_sample(time_elapsed).unwrap();
                                    node.transform.rotation = Vec3::new(x_rot, y_rot, z_rot);
                                }
                            }
                        },
                        AnimationChannelType::Scale => {
                            for node in &mut self.nodes_transforms {
                                if node.node_id == channel.node_index {
                                    let x_scale = channel.x_axis_spline.clamped_sample(time_elapsed).unwrap();
                                    let y_scale = channel.y_axis_spline.clamped_sample(time_elapsed).unwrap();
                                    let z_scale = channel.z_axis_spline.clamped_sample(time_elapsed).unwrap();
                                    node.transform.scale = Vec3::new(x_scale, y_scale, z_scale);
                                }
                            }
                        },
                    }
                }
            }
        };
    }

    fn render(&mut self, _display: &mut glium::Display, _target: &mut glium::Frame) { }



    fn get_local_transform(&self) -> Transform {
        self.transform
    }

    fn set_local_transform(&mut self, transform: Transform) {
        self.transform = transform
    }

    fn get_parent_transform(&self) -> Option<Transform> {
        self.parent_transform
    }

    fn set_parent_transform(&mut self, transform: Transform) {
        self.parent_transform = Some(transform);
    }


    fn get_children_list(&self) -> &Vec<Box<dyn Object>> {
        &self.children
    }

    fn get_children_list_mut(&mut self) -> &mut Vec<Box<dyn Object>> {
        &mut self.children
    }


    fn call(&mut self, name: &str, args: Vec<&str>) {
        if name == "play_animation" && !args.is_empty() {
            let _ = self.play_animation(args[0]);
        }
    }
}

impl ModelObject {
    pub fn get_asset(&self) -> &ModelAsset {
        &self.asset
    }

    pub fn play_animation(&mut self, anim_name: &str) -> Result<(), ModelObjectError> {
        let anim_option = self.asset.find_animation(anim_name);

        match anim_option {
            Some(animation) => {
                self.animation_settings = CurrentAnimationSettings {
                    animation: Some(animation),
                    looping: self.animation_settings.looping,
                    timer: Some(Instant::now())
                };

                return Ok(());
            },
            None => return Err(ModelObjectError::AnimationNotFound)
        }
    }
}

#[derive(Debug)]
pub struct CurrentAnimationSettings {
    pub animation: Option<Animation>, 
    pub looping: bool,
    pub timer: Option<Instant>
}

#[derive(Debug)]
pub struct NodeTransform {
    pub transform: Transform,
    pub node_id: usize
}

pub enum ModelObjectError {
    AnimationNotFound
}

