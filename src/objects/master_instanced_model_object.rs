use super::{gen_object_id, model_object::{CurrentAnimationSettings, NodeTransform}, Object, ObjectGroup, Transform};
use crate::{
    assets::{
        model_asset::{self, Animation, AnimationChannel, AnimationChannelType},
        shader_asset::ShaderAsset,
    },
    framework::Framework,
    managers::{
        assets::{AssetManager, ModelAssetId, TextureAssetId},
        debugger::{error, warn},
        physics::ObjectBodyParameters,
        render::{CurrentCascade, Instance, ModelData, RenderLayers, RenderManager},
    },
};
use egui_glium::egui_winit::egui::ComboBox;
use glam::{Mat4, Quat, Vec3};
use glium::{
    glutin::surface::WindowSurface,
    implement_vertex, uniform,
    uniforms::{
        MagnifySamplerFilter, MinifySamplerFilter, Sampler, SamplerWrapFunction, UniformBuffer,
    },
    Display, IndexBuffer, Program, Surface,
};
use std::{collections::HashMap, time::Instant};

#[derive(Debug)]
pub struct MasterInstancedModelObject {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    body: Option<ObjectBodyParameters>,
    id: u128,
    groups: Vec<ObjectGroup>,
    pub model_asset_id: ModelAssetId,
    pub nodes_transforms: Vec<NodeTransform>,
    pub animation_settings: CurrentAnimationSettings,
    pub shader_asset: ShaderAsset,
    pub texture_asset_id: Option<TextureAssetId>,
    programs: Vec<Program>,
    shadow_programs: Vec<Program>,
    started: bool,
    error: bool,
    inspector_anim_name: String,
    object_properties: HashMap<String, Vec<crate::managers::systems::SystemValue>>,
    layer: RenderLayers,
    is_transparent: bool,
}

impl MasterInstancedModelObject {
    pub fn new(
        name: &str,
        framework: &mut Framework,
        model_asset_id: ModelAssetId,
        texture_asset_id: Option<TextureAssetId>,
        shader_asset: ShaderAsset,
        is_transparent: bool,
        layer: RenderLayers,
    ) -> Self {
        let mut nodes_transforms: Vec<NodeTransform> = vec![];

        let asset = framework.assets.get_model_asset(&model_asset_id);
        match asset {
            Some(asset) => {
                for node in &asset.nodes {
                    let node_local_transform_mat = Mat4::from_cols_array_2d(&node.transform);
                    let node_scale_rotation_translation =
                        node_local_transform_mat.to_scale_rotation_translation();

                    nodes_transforms.push(NodeTransform {
                        local_position: node_scale_rotation_translation.2,
                        local_rotation: node_scale_rotation_translation.1,
                        local_scale: node_scale_rotation_translation.0,
                        global_transform: Mat4::IDENTITY,
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
                    model_asset_id,
                    groups: vec![],
                    texture_asset_id,
                    shader_asset,
                    programs: vec![],
                    shadow_programs: vec![],
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
                    object_properties: HashMap::new(),
                    layer,
                    is_transparent,
                }
            }
            None => {
                error(&format!(
                        "Failed to create a new MasterInstancedModelObject\nFailed to get ModelAsset!\nModelAsset id = {}",
                        model_asset_id.get_id()
                    )
                );

                MasterInstancedModelObject {
                    transform: Transform::default(),
                    nodes_transforms,
                    children: vec![],
                    name: name.to_string(),
                    parent_transform: None,
                    model_asset_id,
                    groups: vec![],
                    texture_asset_id,
                    shader_asset,
                    programs: vec![],
                    shadow_programs: vec![],
                    started: true,
                    error: true,
                    animation_settings: CurrentAnimationSettings {
                        animation: None,
                        looping: false,
                        timer: None,
                    },
                    body: None,
                    id: gen_object_id(),
                    inspector_anim_name: "None".into(),
                    object_properties: HashMap::new(),
                    layer,
                    is_transparent,
                }
            }
        }
    }
}

impl Object for MasterInstancedModelObject {
    fn set_object_properties(&mut self, properties: HashMap<String, Vec<crate::managers::systems::SystemValue>>) {
        self.object_properties = properties.clone();
        crate::managers::systems::register_object_id_properties(self.object_id().to_owned(), properties);
    }

    fn object_properties(&self) -> &HashMap<String, Vec<crate::managers::systems::SystemValue>> {
        &self.object_properties
    }

    fn start(&mut self) {}

    fn update(&mut self, _: &mut Framework) {}

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

    fn inspector_ui(
        &mut self,
        framework: &mut Framework,
        ui: &mut egui_glium::egui_winit::egui::Ui,
    ) {
        ui.heading("MasterInstancedModelObject parameters");
        ui.label(&format!("error: {}", self.error));
        ui.label(&format!(
            "model asset's id: {}",
            self.model_asset_id.get_id()
        ));
        ui.label(&format!(
            "texture asset: {}",
            self.texture_asset_id.is_some()
        ));

        let anim_name = self.inspector_anim_name.clone();
        ComboBox::from_label("animation")
            .selected_text(&anim_name)
            .show_ui(ui, |ui| {
                if let Some(asset) = framework.assets.get_model_asset(&self.model_asset_id) {
                    for anim in asset.animations.clone() {
                        if ui.selectable_label(false, &anim.name).clicked() {
                            self.inspector_anim_name = anim.name;
                        }
                    }
                }
            });

        ui.horizontal(|ui| {
            if ui.button("play animation").clicked() {
                let _ = self.play_animation(framework, &anim_name);
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

    fn render(&mut self, framework: &mut Framework) {
        let render = framework.render.as_mut().expect("MasterInstancedModelObject: render(): There's no RenderManager!");

        self.update_animation();
        if let Some(asset) = framework.assets.get_model_asset(&self.model_asset_id) {
            for node in &asset.root_nodes {
                set_nodes_global_transform(&node, &asset.nodes, None, &mut self.nodes_transforms);
            }
        }

        match self.is_transparent {
            false => render.set_opaque_model_transform(*self.object_id(), self.transform, self.nodes_transforms.clone()),
            true => render.set_transparent_model_transform(*self.object_id(), self.transform, self.nodes_transforms.clone()),
        }
    }

    fn shadow_render(
        &mut self,
        render: &mut RenderManager,
        assets: &AssetManager,
        current_cascade: &CurrentCascade,
    ) {
        if !self.started {
            self.start_mesh(assets, render);
        }

        if self.error {
            return;
        }

        let matrices = match render.get_instance_positions(&self.name) {
            Some(matrices) => matrices,
            None => return,
        };
        let mut per_instance_data = Vec::new();

        for matrix in matrices {
            per_instance_data.push(Instance {
                model: matrix.to_cols_array_2d(),
            });
        }
        let per_instance_buffer =
            glium::vertex::VertexBuffer::dynamic(&render.display, &per_instance_data).unwrap();

        if let Some(asset) = assets.get_model_asset(&self.model_asset_id) {
            let vertex_buffers = &asset.vertex_buffers.as_ref().unwrap();
            for i in 0..asset.objects.len() {
                let vertex_buffer = &vertex_buffers[i];
                let object = &asset.objects[i];

                let indices = IndexBuffer::new(
                    &render.display,
                    glium::index::PrimitiveType::TrianglesList,
                    &object.indices,
                );

                let mut transform: Option<&NodeTransform> = None;
                for tr in &self.nodes_transforms {
                    if tr.node_id == asset.objects[i].node_index {
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

                let view_proj_cols = match current_cascade {
                    CurrentCascade::Closest => render.cascades.closest_view_proj.to_cols_array_2d(),
                    CurrentCascade::Furthest => {
                        render.cascades.furthest_view_proj.to_cols_array_2d()
                    }
                };

                let uniforms = uniform! {
                    view_proj: [
                        view_proj_cols[0],
                        view_proj_cols[1],
                        view_proj_cols[2],
                        view_proj_cols[3],
                    ],
                    lightPos: render.get_light_direction().to_array(),
                };

                let draw_params = glium::DrawParameters {
                    depth: glium::Depth {
                        test: glium::draw_parameters::DepthTest::IfLessOrEqual, // set to IfLess if it
                        write: true,
                        ..Default::default()
                    },
                    backface_culling:
                        glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise,
                    ..Default::default()
                };

                let mut target = match current_cascade {
                    CurrentCascade::Closest => render.closest_shadow_fbo(),
                    CurrentCascade::Furthest => render.furthest_shadow_fbo(),
                };

                target
                    .draw(
                        (vertex_buffer, per_instance_buffer.per_instance().unwrap()),
                        &indices.unwrap(),
                        &self.shadow_programs[i],
                        &uniforms,
                        &draw_params,
                    )
                    .unwrap();
            }
        }
    }
}

impl MasterInstancedModelObject {
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

    pub fn stop_animation(&mut self) {
        self.animation_settings.animation = None;
        self.animation_settings.timer = None;
    }

    pub fn play_animation(
        &mut self,
        framework: &mut Framework,
        anim_name: &str,
    ) -> Result<(), ModelObjectError> {
        let asset = framework
            .assets
            .get_model_asset(&self.model_asset_id)
            .expect("Failed to play the animation! Failed to get the asset.");
        let anim_option = asset.find_animation(anim_name);

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

    fn get_joints_transforms(&self, assets: &AssetManager) -> [[[f32; 4]; 4]; 128] {
        let asset = assets
            .get_model_asset(&self.model_asset_id)
            .expect("Failed to play the animation! Failed to get the asset.");

        let mut joints_vec: Vec<&NodeTransform> = Vec::new();
        for joint in &asset.joints {
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
            joints_mat_options_vec.insert(joint.node_id, Some(joint.global_transform.to_cols_array_2d()));
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

    fn start_mesh(&mut self, assets: &AssetManager, render: &mut RenderManager) {
        let display = &render.display;
        let shadow_shader = ShaderAsset::load_instanced_shadow_shader();

        let shadow_shader = if let Ok(shadow_shader) = shadow_shader {
            shadow_shader
        } else {
            error("failed to load shadow shader!");
            self.error = true;
            return;
        };

        let vertex_shader_source = &self.shader_asset.vertex_shader_source;
        let fragment_shader_source = &self.shader_asset.fragment_shader_source;

        let vertex_shadow_shader_src = &shadow_shader.vertex_shader_source;
        let fragment_shadow_shader_src = &shadow_shader.fragment_shader_source;

        let asset = assets.get_model_asset(&self.model_asset_id);
        let mut programs = Vec::new();
        if let Some(asset) = asset {
            for _ in &asset.objects {
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
                    Ok(prog) => programs.push(prog),
                    Err(err) => {
                        error(&format!(
                            "MasterInstancedModelObject error:\nprogram creation error!\nErr: {}",
                            err
                        ));
                        self.error = true;
                        return;
                    }
                }

                match shadow_program {
                    Ok(prog) => self.shadow_programs.push(prog),
                    Err(err) => {
                        error(&format!(
                            "MasterInstancedModelObject error:\nprogram creation error(shadow)!\nErr: {}",
                            err
                        ));
                        self.error = true;
                        return;
                    }
                }
            }

            match self.is_transparent {
                true => {
                    render.add_transparent_model(*self.object_id(), ModelData {
                        transform: self.global_transform(),
                        model_asset_id: self.model_asset_id.clone(),
                        nodes_transforms: self.nodes_transforms.clone(),
                        animation_settings: self.animation_settings.clone(),
                        shader_asset: self.shader_asset.clone(),
                        texture_asset_id: self.texture_asset_id.clone(),
                        programs,
                        layer: self.layer.clone(),
                        started: self.started,
                        error: self.error,
                        master_object_id: Some(self.name.clone()),
                    });
                },
                false => {
                    render.add_opaque_model(*self.object_id(), ModelData {
                        transform: self.global_transform(),
                        model_asset_id: self.model_asset_id.clone(),
                        nodes_transforms: self.nodes_transforms.clone(),
                        animation_settings: self.animation_settings.clone(),
                        shader_asset: self.shader_asset.clone(),
                        texture_asset_id: self.texture_asset_id.clone(),
                        programs,
                        layer: self.layer.clone(),
                        started: self.started,
                        error: self.error,
                        master_object_id: Some(self.name.clone()),
                    });
                },
            }
            self.started = true;
        }
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
                        let w_rot = channel.w_axis_spline.as_ref().unwrap().clamped_sample(time_elapsed).unwrap();
                        node.local_rotation = Quat::from_xyzw(x_rot, y_rot, z_rot, w_rot);
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
    parent_transform: Option<Mat4>,
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
    let local_scale = node_transform.local_scale;
    let local_transform = Mat4::from_translation(node_transform.local_position)
        * Mat4::from_quat(local_rotation)
        /* Mat4::from_rotation_z(-local_rotation.z)
        * Mat4::from_rotation_y(-local_rotation.y)
        * Mat4::from_rotation_x(-local_rotation.x)*/
        * Mat4::from_scale(local_scale);

    let global_transform: Mat4;
    match parent_transform {
        Some(parent_transform) => {
            global_transform = parent_transform * local_transform;
        }
        None => {
            global_transform = local_transform;
        },
    }
    node_transform.global_transform = global_transform;

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
            Some(global_transform),
            nodes_transforms,
        );
    }
}

#[derive(Debug)]
pub enum ModelObjectError {
    AnimationNotFound,
}
