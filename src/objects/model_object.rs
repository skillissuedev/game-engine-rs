use std::time::Instant;
use glam::{Mat4, Quat, Vec3};
use glium::{VertexBuffer, Program, IndexBuffer, Display, uniform, Surface, uniforms::UniformBuffer};
use crate::{assets::{model_asset::{ModelAsset, Animation, AnimationChannelType, AnimationChannel, self}, shader_asset::ShaderAsset, texture_asset::TextureAsset}, managers::{render::{Vertex, self}, debugger::error}, math_utils::deg_to_rad};
use super::{Object, Transform};

#[derive(Debug)]
pub struct ModelObject {
    pub name: String,
    pub transform: Transform,
    pub parent_transform: Option<Transform>,
    pub children: Vec<Box<dyn Object>>,
    pub asset: ModelAsset,
    pub nodes_transforms: Vec<NodeTransform>,
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
        let mut nodes_transforms: Vec<NodeTransform> = vec![];
        for node in &asset.nodes {
            let node_local_transform_mat = Mat4::from_cols_array_2d(&node.transform);
            let node_scale_rotation_translation = node_local_transform_mat.to_scale_rotation_translation();
            let node_rotation = node_scale_rotation_translation.1.to_euler(glam::EulerRot::XYZ);

            nodes_transforms.push(
                NodeTransform { 
                    local_position: node_scale_rotation_translation.2,
                    local_rotation: node_rotation.into(), 
                    local_scale: node_scale_rotation_translation.0,
                    global_transform: None, 
                    node_id: node.node_index
                }
            );
        }

        ModelObject {
            transform: Transform::default(),
            nodes_transforms,
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

    }

    fn update(&mut self) {
        self.update_animation();
        for node in &self.asset.root_nodes {
            set_nodes_global_transform(&node, &self.asset.nodes, None, &mut self.nodes_transforms);
        }
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
            let mut node_mat: Mat4 = Mat4::IDENTITY;
            for node_transform in &self.nodes_transforms {
                if node_transform.node_id == object.node_index {
                    match node_transform.global_transform {
                        Some(node_tr) => node_mat = node_tr,
                        None => error(
                            &format!("model object\ngot an error in render()\nnode transform's global_transform is None\node_index is {}", node_transform.node_id)),
                    }
                }
            }

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
            let mvp: Mat4 = self.setup_mat(transform.unwrap());

            let texture_option = self.texture.as_ref();

            let empty_texture = glium::texture::Texture2d::empty(display, 1, 1).unwrap();
            let texture: &glium::texture::Texture2d;
            match texture_option {
                Some(tx) => texture = tx,
                None => texture = &empty_texture,
            }
            let mvp_cols = mvp.to_cols_array_2d();

            let joints = UniformBuffer::new(display, self.asset.joints_mats).unwrap();
            let inverse_bind_mats = UniformBuffer::new(display, self.asset.joints_inverse_bind_mats).unwrap();

            let uniforms = uniform! {
                jointsMats: &joints,
                jointsInverseBindMats: &inverse_bind_mats,
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

    fn setup_mat(&self, node_transform: &NodeTransform) -> Mat4 {
        match node_transform.global_transform {
            Some(_) => (),
            None => {
                error("model object error\nerror in setup_mat()\nnode_transform's global_transform is None");
                return Mat4::IDENTITY;
            },
        }
        let node_global_transform = node_transform.global_transform.unwrap();
        let scale_rotation_translation = node_global_transform.to_scale_rotation_translation();
        let rotation_vector = scale_rotation_translation.1.to_euler(glam::EulerRot::XYZ);
        let translation_vector = scale_rotation_translation.2;
        let scale_vector = scale_rotation_translation.0;

        let scale_rotation_translation = node_transform.global_transform;
        let model_object_translation: [f32; 3] = self.transform.position.into();
        let model_object_rotation_vec: [f32; 3] = self.transform.rotation.into();
        let model_object_scale: [f32; 3] = self.transform.scale.into();
        let model_object_rotation_vec = 
            [deg_to_rad(model_object_rotation_vec[0]), deg_to_rad(model_object_rotation_vec[1]), deg_to_rad(model_object_rotation_vec[2])];

        let full_translation = 
            Vec3::new(
                model_object_translation[0] + translation_vector.x, 
                model_object_translation[1] + translation_vector.y, 
                model_object_translation[2] + translation_vector.z);
        let full_scale = 
            Vec3::new(
                model_object_scale[0] + scale_vector.x, 
                model_object_scale[1] + scale_vector.y, 
                model_object_scale[2] + scale_vector.z);
        let full_rotation = 
            [model_object_rotation_vec[0] + rotation_vector.0, model_object_rotation_vec[1] + rotation_vector.1, model_object_rotation_vec[2] + rotation_vector.2];
        let rotation_quat = 
            Quat::from_euler(glam::EulerRot::XYZ, full_rotation[0], full_rotation[1], full_rotation[2]);


        let transform = Mat4::from_scale_rotation_translation(full_scale, rotation_quat, full_translation);
        let view = glam::Mat4::from_cols_array_2d(&render::get_view_matrix().into());
        let proj = glam::Mat4::from_cols_array_2d(&render::get_projection_matrix().into());

        proj * view * transform
    }

    fn start_mesh(&mut self, display: &Display) {
        for i in &self.asset.objects {
            let vertex_buffer = VertexBuffer::new(display, &i.vertices);
            match vertex_buffer {
                Ok(buff) => self.vertex_buffer.push(buff),
                Err(err) => {
                    error(&format!(
                        "Mesh object error:\nvertex buffer creation error!\nErr: {}",
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
                        "Mesh object error:\nprogram creation error!\nErr: {}",
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
                        "Mesh object error:\ntexture creating error!\nErr: {}",
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
                        node.local_position = Vec3::new(x_pos, y_pos, z_pos);
                    }
                }
            },
            AnimationChannelType::Rotation => {
                for node in &mut *nodes_transforms {
                    if node.node_id == channel.node_index {
                        let x_rot = channel.x_axis_spline.clamped_sample(time_elapsed).unwrap();
                        let y_rot = channel.y_axis_spline.clamped_sample(time_elapsed).unwrap();
                        let z_rot = channel.z_axis_spline.clamped_sample(time_elapsed).unwrap();
                        node.local_rotation = Vec3::new(x_rot, y_rot, z_rot);
                    }
                }
            },
            AnimationChannelType::Scale => {
                for node in &mut *nodes_transforms {
                    if node.node_id == channel.node_index {
                        let x_scale = channel.x_axis_spline.clamped_sample(time_elapsed).unwrap();
                        let y_scale = channel.y_axis_spline.clamped_sample(time_elapsed).unwrap();
                        let z_scale = channel.z_axis_spline.clamped_sample(time_elapsed).unwrap();
                        node.local_scale = Vec3::new(x_scale, y_scale, z_scale);
                    }
                }
            },
        }
    }
}

fn set_nodes_global_transform
    (node: &model_asset::Node, nodes_list: &Vec<model_asset::Node>, parent_transform_mat: Option<Mat4>, nodes_transforms: &mut Vec<NodeTransform>) {

    let node_index = node.node_index;
    let mut node_transform: Option<&mut NodeTransform> = None;

    for transform in nodes_transforms.into_iter() {
        if transform.node_id == node_index {
            node_transform = Some(transform);
        }
    }

    if node_transform.is_none() {
        error("model object error\ngot an error in set_nodes_global_transform()\nnode transform not found");
        return;
    }
    let node_transform = node_transform.expect("node transform was None(why)");

    let local_rotation = node_transform.local_rotation;
    let local_rotation_quat = Quat::from_euler(glam::EulerRot::XYZ, local_rotation.x, local_rotation.y, local_rotation.z);
    let local_transform = Mat4::from_scale_rotation_translation(node_transform.local_scale, local_rotation_quat, node_transform.local_position);


    let global_transform_mat: Mat4;
    match parent_transform_mat {
        Some(parent_tr_mat) => global_transform_mat = parent_tr_mat * local_transform,
        None => global_transform_mat = local_transform
    }

    node_transform.global_transform = Some(global_transform_mat);

    
    let mut children_nodes: Vec<&model_asset::Node> = vec![];
    for child_id in &node.children_id {
        for current_node in nodes_list {
            if &current_node.node_index == child_id {
                children_nodes.push(current_node);
            }
        }
    }

    for child in children_nodes {
        set_nodes_global_transform(child, nodes_list, Some(global_transform_mat), nodes_transforms);
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
    pub local_position: Vec3,
    pub local_rotation: Vec3,
    pub local_scale: Vec3,
    pub global_transform: Option<Mat4>,
    pub node_id: usize
}

#[derive(Debug)]
pub enum ModelObjectError {
    AnimationNotFound
}

