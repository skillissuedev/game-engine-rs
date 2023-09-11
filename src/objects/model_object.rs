use std::time::Instant;
use glam::{Mat4, Quat};
use glium::{VertexBuffer, Program, IndexBuffer, Display, uniform, Surface};
use ultraviolet::Vec3;
use crate::{assets::{model_asset::{ModelAsset, Animation, AnimationChannelType, AnimationChannel}, shader_asset::ShaderAsset, texture_asset::TextureAsset}, managers::{render::{Vertex, self}, debugger::error}, math_utils::deg_to_rad};
use super::{Object, Transform};

#[derive(Debug)]
pub struct ModelObject {
    pub name: String,
    pub transform: Transform,
    pub nodes_transforms: Vec<NodeTransform>,
    pub parent_transform: Option<Transform>,
    pub children: Vec<Box<dyn Object>>,
    pub asset: ModelAsset,
    pub animation_settings: CurrentAnimationSettings,
    pub shader_asset: ShaderAsset,
    pub texture_asset: Option<TextureAsset>,
    texture: Option<glium::texture::Texture2d>,
    vertex_buffer: Vec<VertexBuffer<Vertex>>,
    program: Vec<Program>,
    started: bool,
    error: bool,
}

impl ModelObject {
    pub fn new(name: &str, asset: ModelAsset, texture_asset: Option<TextureAsset>, shader_asset: ShaderAsset) -> Self {
        ModelObject {
            transform: Transform::default(),
            nodes_transforms: vec![],
            children: vec![],
            name: name.to_string(),
            parent_transform: None, asset,
            texture_asset,
            shader_asset,
            texture: None,
            vertex_buffer: vec![], program: vec![],
            started: false, error: false,
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
        self.update_animation();
        //println!("{:?}", self.nodes_transforms[0].transform.rotation);
    }

    fn render(&mut self, display: &mut glium::Display, target: &mut glium::Frame) {
        if self.error {
            return;
        }
        if !self.started {
            self.start_mesh(display);
        }

        for i in 0..self.asset.objects.len() {
            let object = &self.asset.objects[i];
            let indices = IndexBuffer::new(
                display,
                glium::index::PrimitiveType::TrianglesList,
                &object.indices,
            );

            let mut transform: Option<&NodeTransform> = None;
            for tr in &self.nodes_transforms {
                if tr.node_id == self.asset.objects[i].node_index {
                    transform = Some(tr);
                    break;
                } 
            }

            match transform {
                Some(_) => (), 
                None => {
                    error("no node transform found!");
                    return;
                }
            }
            let transform = transform.unwrap();
            let mvp: Mat4 = self.setup_mat(transform);

            let texture_option = self.texture.as_ref();

            let empty_texture = glium::texture::Texture2d::empty(display, 1, 1).unwrap();
            let texture: &glium::texture::Texture2d;
            match texture_option {
                Some(tx) => texture = tx,
                None => texture = &empty_texture,
            }
            let mvp_cols = mvp.to_cols_array_2d();

            let uniforms = uniform! {
                mesh: object.transform,
                mvp: [
                    mvp_cols[0],
                    mvp_cols[1],
                    mvp_cols[2],
                    mvp_cols[3],
                ],
                tex: texture,
            };

            let draw_params = glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                ..Default::default()
            };

            target
                .draw(
                    &self.vertex_buffer[i],
                    &indices.unwrap(),
                    &self.program[i],
                    &uniforms,
                    &draw_params,
                )
                .unwrap();
        } 
    }



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


    fn call(&mut self, name: &str, args: Vec<&str>) -> Option<&str> {
        if name == "play_animation" && !args.is_empty() {
            let _ = self.play_animation(args[0]);
            return None;
        }

        if name == "set_looping" && !args.is_empty() {
            let looping = match args[0] {
                "true" => true,
                "false" => false,
                _ => {
                    error("set_looping model object call failed - wrong args(only 'true' and 'false' avaliable)");
                    return None;
                }
            };

            self.animation_settings.looping = looping;

            return None;
        }
        return None;
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
                println!("playing animation!!!");
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

    fn update_animation(&mut self) {
        let anim_settings = &mut self.animation_settings;
        let animation_option = &mut anim_settings.animation;

        match animation_option {
            Some(ref mut animation) => {
                let timer = &anim_settings.timer;
                let time_elapsed = timer.expect("no timer(why)").elapsed().as_secs_f32();
                set_objects_anim_node_transform(&mut animation.channels, &mut self.nodes_transforms, time_elapsed);
                println!("animation is playing!");

                if time_elapsed >= animation.duration {
                    if anim_settings.looping {
                        set_objects_anim_node_transform(&mut animation.channels, &mut self.nodes_transforms, time_elapsed);
                        anim_settings.timer = Some(Instant::now());
                    } else {
                        anim_settings.animation = None;
                        anim_settings.timer = None;
                        return;
                    }
                }
            },
            None => ()
        }
    }

    fn setup_mat(&self, node_transform_data: &NodeTransform) -> Mat4 {
        let transform_data = node_transform_data.transform;

        let object_translation: [f32; 3] = self.transform.position.into();
        let object_rotation_vec: [f32; 3] = self.transform.rotation.into();
        let object_scale: [f32; 3] = self.transform.scale.into();
        let object_rotation_vec = [deg_to_rad(object_rotation_vec[0]), deg_to_rad(object_rotation_vec[1]), deg_to_rad(object_rotation_vec[2])];

        let node_rotation_vec: [f32; 3] = transform_data.rotation.into();
        // LINE BELOW!!!
        let rotation_vector = [object_rotation_vec[0] + node_rotation_vec[0], object_rotation_vec[1] + node_rotation_vec[1], object_rotation_vec[2] + node_rotation_vec[2]];
        let rotation = Quat::from_euler(glam::EulerRot::XYZ, rotation_vector[0], rotation_vector[1], rotation_vector[2]);

        let node_translation = glam::Vec3::new(transform_data.position.x, transform_data.position.y, transform_data.position.z);
        let translation: glam::Vec3 = glam::Vec3::from_array(object_translation) + node_translation;

        let node_scale = glam::Vec3::new(transform_data.scale.x, transform_data.scale.y, transform_data.scale.z);
        let scale: glam::Vec3 = glam::Vec3::from_array(object_scale) + node_scale;


        let transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);
        let view = glam::Mat4::from_cols_array_2d(&render::get_view_matrix().into());
        let proj = glam::Mat4::from_cols_array_2d(&render::get_projection_matrix().into());


        //println!("{:?}", transform.to_scale_rotation_translation().1);
        println!("{:?}", rotation.to_euler(glam::EulerRot::XYZ));

        proj * view * transform
    }

    fn start_mesh(&mut self, display: &Display) {
        for i in &self.asset.objects {
            let vertex_buffer = VertexBuffer::new(display, &i.vertices);
            match vertex_buffer {
                Ok(buff) => self.vertex_buffer.push(buff),
                Err(err) => {
                    error(&format!(
                        "Mesh component error:\nvertex buffer creation error!\nErr: {}",
                        err
                    ));
                    self.error = true;
                    return;
                }
            }
        }

        let vertex_shader_source = &self.shader_asset.vertex_shader_source;
        let fragment_shader_source = &self.shader_asset.fragment_shader_source;

        for _i in &self.asset.objects {
            let program = Program::from_source(
                display,
                &vertex_shader_source,
                &fragment_shader_source,
                None,
            );
            match program {
                Ok(prog) => self.program.push(prog),
                Err(err) => {
                    error(&format!(
                        "Mesh component error:\nprogram creation error!\nErr: {}",
                        err
                    ));
                    self.error = true;
                    return;
                }
            }
        }

        if self.texture_asset.is_some() {
            let asset = self.texture_asset.as_ref().unwrap();
            let image = glium::texture::RawImage2d::from_raw_rgba_reversed(
                &asset.image_raw,
                asset.image_dimensions,
            );
            let texture = glium::texture::texture2d::Texture2d::new(display, image);

            match texture {
                Ok(tx) => self.texture = Some(tx),
                Err(err) => {
                    error(&format!(
                        "Mesh component error:\ntexture creating error!\nErr: {}",
                        err
                    ));
                    self.texture = None;
                }
            }
        }

        self.started = true;
    }
}

fn set_objects_anim_node_transform(channels: &mut Vec<AnimationChannel>, nodes_transforms: &mut Vec<NodeTransform>, time_elapsed: f32) {
    for channel in channels {
        match channel.channel_type {
            AnimationChannelType::Translation => {
                for node in &mut *nodes_transforms {
                    if node.node_id == channel.node_index {
                        let x_pos = channel.x_axis_spline.clamped_sample(time_elapsed).unwrap();
                        let y_pos = channel.y_axis_spline.clamped_sample(time_elapsed).unwrap();
                        let z_pos = channel.z_axis_spline.clamped_sample(time_elapsed).unwrap();
                        node.transform.position = Vec3::new(x_pos, y_pos, z_pos);
                    }
                }
            },
            AnimationChannelType::Rotation => {
                for node in &mut *nodes_transforms {
                    if node.node_id == channel.node_index {
                        let x_rot = channel.x_axis_spline.clamped_sample(time_elapsed).unwrap();
                        let y_rot = channel.y_axis_spline.clamped_sample(time_elapsed).unwrap();
                        let z_rot = channel.z_axis_spline.clamped_sample(time_elapsed).unwrap();
                        node.transform.rotation = Vec3::new(x_rot, y_rot, z_rot);
                    }
                }
            },
            AnimationChannelType::Scale => {
                for node in &mut *nodes_transforms {
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

