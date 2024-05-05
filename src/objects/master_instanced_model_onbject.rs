use super::{gen_object_id, Object, ObjectGroup, Transform};
use crate::{
    assets::{
        model_asset::{self, Animation, AnimationChannel, AnimationChannelType, ModelAsset},
        shader_asset::ShaderAsset,
        texture_asset::TextureAsset,
    },
    managers::{
        debugger::{self, error, warn},
        physics::ObjectBodyParameters,
        render::{self, Cascades, ShadowTextures, Vertex},
    },
};
use egui_glium::egui_winit::egui::ComboBox;
use glam::{Mat4, Quat, Vec3};
use glium::{
    framebuffer::SimpleFrameBuffer,
    uniform,
    uniforms::{
        MagnifySamplerFilter, MinifySamplerFilter, Sampler, SamplerWrapFunction, UniformBuffer,
    },
    Display, IndexBuffer, Program, Surface, VertexBuffer,
};
use std::time::Instant;

#[derive(Debug)]
pub struct MasterInstancedModelObject {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    body: Option<ObjectBodyParameters>,
    id: u128,
    groups: Vec<ObjectGroup>,
    pub model_asset: ModelAsset,
    pub nodes_transforms: Vec<NodeTransform>,
    pub animation_settings: CurrentAnimationSettings,
    pub shader_asset: ShaderAsset,
    pub texture_asset: Option<TextureAsset>,
    texture: Option<glium::texture::Texture2d>,
    vertex_buffer: Vec<VertexBuffer<Vertex>>,
    program: Vec<Program>,
    shadow_program: Vec<Program>,
    started: bool,
    error: bool,
    inspector_anim_name: String,
}

impl MasterInstancedModelObject {
    pub fn new(
        name: &str,
        asset: ModelAsset,
        texture_asset: Option<TextureAsset>,
        shader_asset: ShaderAsset,
    ) -> Self {
        let mut nodes_transforms: Vec<NodeTransform> = vec![];
        for node in &asset.nodes {
            let node_local_transform_mat = Mat4::from_cols_array_2d(&node.transform);
            let node_scale_rotation_translation =
                node_local_transform_mat.to_scale_rotation_translation();
            let node_rotation = node_scale_rotation_translation
                .1
                .to_euler(glam::EulerRot::XYZ);

            nodes_transforms.push(NodeTransform {
                local_position: node_scale_rotation_translation.2,
                local_rotation: node_rotation.into(),
                local_scale: node_scale_rotation_translation.0,
                global_transform: None,
                node_id: node.node_index,
                parent_global_transform: None,
            });
        }

        MasterInstancedModelObject {
            transform: Transform::default(),
            nodes_transforms,
            children: vec![],
            name: name.to_string(),
            parent_transform: None,
            model_asset: asset,
            groups: vec![],
            texture_asset,
            shader_asset,
            texture: None,
            vertex_buffer: vec![],
            program: vec![],
            shadow_program: vec![],
            started: false,
            error: false,
            animation_settings: CurrentAnimationSettings {
                animation: None,
                looping: false,
                timer: None,
            },
            body: None,
            id: gen_object_id(),
            inspector_anim_name: "None".into(),
        }
    }
}

impl Object for MasterInstancedModelObject {
    fn start(&mut self) {}

    fn update(&mut self) {
        self.update_animation();
        for node in &self.model_asset.root_nodes {
            set_nodes_global_transform(
                &node,
                &self.model_asset.nodes,
                None,
                &mut self.nodes_transforms,
            );
        }
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
        "MasterInstancedModelObject"
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
        ui.heading("MasterInstancedModelObject parameters");
        ui.label(&format!("error: {}", self.error));
        ui.label(&format!("model asset: {}", self.model_asset.path));
        ui.label(&format!("texture asset: {}", self.texture_asset.is_some()));

        let anim_name = self.inspector_anim_name.clone();
        ComboBox::from_label("animation")
            .selected_text(&anim_name)
            .show_ui(ui, |ui| {
                for anim in self.model_asset.animations.clone() {
                    if ui.selectable_label(false, &anim.name).clicked() {
                        self.inspector_anim_name = anim.name;
                    }
                }
            });

        ui.horizontal(|ui| {
            if ui.button("play animation").clicked() {
                let _ = self.play_animation(&anim_name);
            }
            ui.checkbox(&mut self.animation_settings.looping, "loop");
            if ui.button("stop").clicked() {
                self.animation_settings.animation = None;
                self.animation_settings.timer = None;
            }
            if let Some(timer) = &self.animation_settings.timer {
                ui.label("time:");
                ui.label(format!("{} s", timer.elapsed().as_secs_f32()));
            }
        });
    }

    fn groups_list(&mut self) -> &mut Vec<ObjectGroup> {
        &mut self.groups
    }

    fn render(
        &mut self,
        display: &Display,
        target: &mut glium::Frame,
        cascades: &Cascades,
        shadow_texture: &ShadowTextures,
    ) {
        if self.error {
            return;
        }
        if !self.started {
            self.start_mesh(display);
        }

        let matrices = match render::get_instance_positions(&self.name) {
            Some(matrices) => matrices,
            None => return,
        };
        let mut model_matrices = Vec::new();
        let mut mvp_matrices = Vec::new();
        for matrix in matrices {
            model_matrices.push(matrix.model);
            mvp_matrices.push(matrix.mvp);
        }

        let closest_shadow_view_proj_cols = cascades.closest_view_proj.to_cols_array_2d();
        let furthest_shadow_view_proj_cols = cascades.furthest_view_proj.to_cols_array_2d();

        for i in 0..self.model_asset.objects.len() {
            let object = &self.model_asset.objects[i];

            let indices = IndexBuffer::new(
                display,
                glium::index::PrimitiveType::TrianglesList,
                &object.indices,
            );

            let mut transform: Option<&NodeTransform> = None;
            for tr in &self.nodes_transforms {
                if tr.node_id == self.model_asset.objects[i].node_index {
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

            //let setup_mat_result = self.setup_mat(transform.unwrap());
            //let mvp: Mat4 = setup_mat_result.mvp;
            //let model: Mat4 = setup_mat_result.model;

            let texture_option = self.texture.as_ref();

            let empty_texture = glium::texture::Texture2d::empty(display, 1, 1).unwrap();
            let texture: &glium::texture::Texture2d;
            match texture_option {
                Some(tx) => texture = tx,
                None => texture = &empty_texture,
            }
            let mut mvp_cols = Vec::new();
            for matrix in &mvp_matrices {
                mvp_cols.push(matrix.to_cols_array_2d());
            }
            let mut model_cols = Vec::new();
            for matrix in &model_matrices {
                model_cols.push(matrix.to_cols_array_2d());
            }
            let model_cols: [[[f32; 4]; 4]; 4096] = model_cols.try_into().unwrap();
            let mvp_cols: [[[f32; 4]; 4]; 4096] = mvp_cols.try_into().unwrap();

            let model_cols =
                UniformBuffer::new(display, model_cols).unwrap();
            let mvp_cols =
                UniformBuffer::new(display, mvp_cols).unwrap();

            let joints = UniformBuffer::new(display, self.get_joints_transforms()).unwrap();
            let inverse_bind_mats =
                UniformBuffer::new(display, self.model_asset.joints_inverse_bind_mats).unwrap();

            let camera_position: [f32; 3] = render::get_camera_position().into();


            let sampler_behaviour = glium::uniforms::SamplerBehavior {
                minify_filter: MinifySamplerFilter::Nearest,
                magnify_filter: MagnifySamplerFilter::Nearest,
                wrap_function: (
                    SamplerWrapFunction::Repeat,
                    SamplerWrapFunction::Repeat,
                    SamplerWrapFunction::Repeat,
                ),
                ..Default::default()
            };

            let uniforms = uniform! {
                jointsMats: &joints,
                jointsInverseBindMats: &inverse_bind_mats,
                mesh: object.transform,
                mvp: &mvp_cols,
                model: &model_cols, 
                tex: Sampler(texture, sampler_behaviour),
                lightPos: render::get_light_direction().to_array(),
                closestShadowTexture: &shadow_texture.closest,
                furthestShadowTexture: &shadow_texture.furthest,
                closestShadowViewProj: [
                    closest_shadow_view_proj_cols[0],
                    closest_shadow_view_proj_cols[1],
                    closest_shadow_view_proj_cols[2],
                    closest_shadow_view_proj_cols[3],
                ],
                furthestShadowViewProj: [
                    furthest_shadow_view_proj_cols[0],
                    furthest_shadow_view_proj_cols[1],
                    furthest_shadow_view_proj_cols[2],
                    furthest_shadow_view_proj_cols[3],
                ],
                cameraPosition: camera_position,
            };

            let draw_params = glium::DrawParameters {
                depth: glium::Depth {
                    test: glium::draw_parameters::DepthTest::IfLess,
                    write: true,
                    ..Default::default()
                },
                blend: glium::draw_parameters::Blend::alpha_blending(),
                backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                polygon_mode: glium::draw_parameters::PolygonMode::Fill,
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

    fn shadow_render(&mut self, _: &Mat4, _: &Display, _: &mut SimpleFrameBuffer) {
        // no shadow rendering(for now?)
    }
}

impl MasterInstancedModelObject {
    pub fn get_asset(&self) -> &ModelAsset {
        &self.model_asset
    }

    pub fn set_looping(&mut self, should_loop: bool) {
        self.animation_settings.looping = should_loop;
    }

    pub fn is_looping(&self) -> bool {
        self.animation_settings.looping
    }

    pub fn current_animation(&self) -> Option<&str> {
        match &self.animation_settings.animation {
            Some(animation) => Some(&animation.name),
            None => None,
        }
    }

    pub fn play_animation(&mut self, anim_name: &str) -> Result<(), ModelObjectError> {
        let anim_option = self.model_asset.find_animation(anim_name);

        match anim_option {
            Some(animation) => {
                self.animation_settings = CurrentAnimationSettings {
                    animation: Some(animation),
                    looping: self.animation_settings.looping,
                    timer: Some(Instant::now()),
                };

                Ok(())
            }
            None => Err(ModelObjectError::AnimationNotFound),
        }
    }

    fn get_joints_transforms(&self) -> [[[f32; 4]; 4]; 128] {
        let mut joints_vec: Vec<&NodeTransform> = Vec::new();
        for joint in &self.model_asset.joints {
            for node_transform in &self.nodes_transforms {
                if node_transform.node_id == joint.node_index {
                    joints_vec.push(node_transform);
                }
            }
        }

        let identity_mat: [[f32; 4]; 4] = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let mut joints_mat_options_vec: Vec<Option<[[f32; 4]; 4]>> = vec![None; 200];
        let mut joints_mat_vec: Vec<[[f32; 4]; 4]> = vec![identity_mat; 200];

        if joints_vec.len() > 128 {
            warn("model object warning! model contains more than 128 joints!\nonly 100 joints would be used");
        }
        joints_vec.into_iter().for_each(|joint| {
            match joint.global_transform {
                Some(global_tr) => joints_mat_options_vec.insert(joint.node_id, Some(global_tr.to_cols_array_2d())),
                None => warn("model object warning\njoints_transforms_vec_to_array(): joint's global transform is none")
            }
        });

        for joint_idx in 0..joints_mat_options_vec.len() {
            match joints_mat_options_vec[joint_idx] {
                Some(mat) => joints_mat_vec.insert(joint_idx, mat),
                None => joints_mat_vec.insert(joint_idx, identity_mat),
            }
        }
        joints_mat_vec.truncate(128);

        let joints_array: [[[f32; 4]; 4]; 128] = joints_mat_vec
            .try_into()
            .expect("joints_vec_to_array failed!");
        joints_array
    }

    fn update_animation(&mut self) {
        let anim_settings = &mut self.animation_settings;
        let animation_option = &mut anim_settings.animation;

        match animation_option {
            Some(ref mut animation) => {
                let timer = &anim_settings.timer;
                let time_elapsed = timer.expect("no timer(why)").elapsed().as_secs_f32();
                set_objects_anim_node_transform(
                    &mut animation.channels,
                    &mut self.nodes_transforms,
                    time_elapsed,
                );

                if time_elapsed >= animation.duration {
                    if anim_settings.looping {
                        set_objects_anim_node_transform(
                            &mut animation.channels,
                            &mut self.nodes_transforms,
                            time_elapsed,
                        );
                        anim_settings.timer = Some(Instant::now());
                    } else {
                        anim_settings.animation = None;
                        anim_settings.timer = None;
                        //return;
                        ()
                    }
                }
            }
            None => (),
        }
    }

    fn start_mesh(&mut self, display: &Display) {
        let shadow_shader = ShaderAsset::load_shadow_shader();
        let shadow_shader = if let Ok(shadow_shader) = shadow_shader {
            shadow_shader
        } else {
            error("failed to load shadow shader!");
            return;
        };

        for i in &self.model_asset.objects {
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

        let vertex_shadow_shader_src = &shadow_shader.vertex_shader_source;
        let fragment_shadow_shader_src = &shadow_shader.fragment_shader_source;

        for _ in &self.model_asset.objects {
            let program = Program::from_source(
                display,
                &vertex_shader_source,
                &fragment_shader_source,
                None,
            );

            let shadow_program = Program::from_source(
                display,
                &vertex_shadow_shader_src,
                &fragment_shadow_shader_src,
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

            match shadow_program {
                Ok(prog) => self.shadow_program.push(prog),
                Err(err) => {
                    error(&format!(
                        "Mesh object error:\nprogram creation error(shadow)!\nErr: {}",
                        err
                    ));
                    self.error = true;
                    return;
                }
            }
        }

        if self.texture_asset.is_some() {
            let asset = self.texture_asset.as_ref().unwrap();
            let image = glium::texture::RawImage2d::from_raw_rgba(
                asset.image_raw.clone(),
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
        } else {
            let asset_result = TextureAsset::default_texture();
            match asset_result {
                Ok(asset) => {
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
                Err(err) => {
                    debugger::error(&format!(
                        "model object({}) - failed to open a default texture.\nerror: {:?}",
                        self.name, err
                    ));
                    self.texture = None;
                }
            }
        };

        self.started = true;
    }
}

fn set_objects_anim_node_transform(
    channels: &mut Vec<AnimationChannel>,
    nodes_transforms: &mut Vec<NodeTransform>,
    time_elapsed: f32,
) {
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
            }
            AnimationChannelType::Rotation => {
                for node in &mut *nodes_transforms {
                    if node.node_id == channel.node_index {
                        let x_rot = channel.x_axis_spline.clamped_sample(time_elapsed).unwrap();
                        let y_rot = channel.y_axis_spline.clamped_sample(time_elapsed).unwrap();
                        let z_rot = channel.z_axis_spline.clamped_sample(time_elapsed).unwrap();
                        node.local_rotation = Vec3::new(x_rot, y_rot, z_rot);
                    }
                }
            }
            AnimationChannelType::Scale => {
                for node in &mut *nodes_transforms {
                    if node.node_id == channel.node_index {
                        let x_scale = channel.x_axis_spline.clamped_sample(time_elapsed).unwrap();
                        let y_scale = channel.y_axis_spline.clamped_sample(time_elapsed).unwrap();
                        let z_scale = channel.z_axis_spline.clamped_sample(time_elapsed).unwrap();
                        node.local_scale = Vec3::new(x_scale, y_scale, z_scale);
                    }
                }
            }
        }
    }
}

fn set_nodes_global_transform(
    node: &model_asset::Node,
    nodes_list: &Vec<model_asset::Node>,
    parent_transform_mat: Option<Mat4>,
    nodes_transforms: &mut Vec<NodeTransform>,
) {
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
    let local_rotation_quat = Quat::from_euler(
        glam::EulerRot::XYZ,
        local_rotation.x,
        local_rotation.y,
        local_rotation.z,
    );
    let local_transform = Mat4::from_scale_rotation_translation(
        node_transform.local_scale,
        local_rotation_quat,
        node_transform.local_position,
    );

    let global_transform_mat: Mat4;
    match parent_transform_mat {
        Some(parent_tr_mat) => {
            global_transform_mat = parent_tr_mat * local_transform;
            node_transform.parent_global_transform = Some(parent_tr_mat);
        }
        None => global_transform_mat = local_transform,
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
        set_nodes_global_transform(
            child,
            nodes_list,
            Some(global_transform_mat),
            nodes_transforms,
        );
    }
}

#[derive(Debug)]
pub struct CurrentAnimationSettings {
    pub animation: Option<Animation>,
    pub looping: bool,
    pub timer: Option<Instant>,
}

#[derive(Debug)]
pub struct NodeTransform {
    pub local_position: Vec3,
    pub local_rotation: Vec3,
    pub local_scale: Vec3,
    pub global_transform: Option<Mat4>,
    pub parent_global_transform: Option<Mat4>,
    pub node_id: usize,
}

#[derive(Debug)]
pub enum ModelObjectError {
    AnimationNotFound,
}
