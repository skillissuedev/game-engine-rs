use std::{collections::HashMap, time::Instant};
use egui_glium::egui_winit::egui;
use glam::{Mat4, Quat, Vec3};
use glium::{index::PrimitiveType, IndexBuffer, Program, VertexBuffer};
use super::{gen_object_id, model_object::CurrentAnimationData, Object, ObjectGroup, Transform};
use crate::{assets::{model_asset::{ModelAsset, ModelAssetAnimation, ModelAssetObject}, shader_asset::ShaderAsset}, framework::Framework, managers::{assets::{AssetManager, ModelAssetId, ShaderAssetId, TextureAssetId}, debugger, physics::ObjectBodyParameters, render::{RenderLayer, RenderManager, RenderObjectData, RenderShader}}, math_utils::deg_vec_to_rad};

#[derive(Debug)]
pub struct MasterInstancedModelObject {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    body: Option<ObjectBodyParameters>,
    id: u128,
    groups: Vec<ObjectGroup>,
    object_properties: HashMap<String, Vec<crate::managers::systems::SystemValue>>,
    model_asset_id: ModelAssetId,
    texture_asset_id: Option<TextureAssetId>,
    shader: ShaderAssetId,
    layer: RenderLayer,
    transparent: bool,
    started: bool,
    error: bool,
    objects_global_transforms: HashMap<String, Mat4>,
    animation_data: Option<CurrentAnimationData>,
    bone_offsets: HashMap<String, Mat4>,
    cast_shadows: bool,
}

impl MasterInstancedModelObject {
    pub fn new(name: &str, model_asset_id: ModelAssetId, texture_asset_id: Option<TextureAssetId>, shader_asset_id: ShaderAssetId,
            layer: RenderLayer, transparent: bool) -> Self {
        let object_id = gen_object_id();

        MasterInstancedModelObject {
            transform: Transform::default(),
            children: vec![],
            name: name.to_string(),
            parent_transform: None,
            body: None,
            id: object_id,
            groups: vec![],
            object_properties: HashMap::new(),
            model_asset_id,
            texture_asset_id,
            shader: shader_asset_id,
            layer,
            transparent,
            started: false,
            error: false,
            objects_global_transforms: HashMap::new(),
            animation_data: None,
            bone_offsets: HashMap::new(),
            cast_shadows: true,
        }
    }
}

impl Object for MasterInstancedModelObject {
    fn start(&mut self) {}

    fn update(&mut self, _: &mut Framework) {}

    fn render(&mut self, framework: &mut Framework) {
        // we don't want to run render if the initialization of the object failed 
        if self.error == true { return }


        let assets = &mut framework.assets;
        let asset = assets.get_model_asset(&self.model_asset_id);
        if let Some(asset) = asset {
            let render = framework
                .render.as_mut()
                .expect("No render manager! - MasterInstancedModelObject(render)");

            // initialize the object if it wasn't already
            if self.started == false {
                self.add_all_objects(assets, render);
                self.started = true;
            }

            self.loop_animation_if_needed(&asset);
            self.update_all_object_transforms(&asset);
            self.update_all_render_objects(render, &asset);
        } else {
            self.error = true;
            debugger::error(
                &format!(
                    "Failed to get the model asset with id '{}'! - MasterInstancedModelObject(render)",
                    self.model_asset_id.get_id()
                )
            );
        };
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

    fn inspector_ui(&mut self, _: &mut Framework, ui: &mut egui::Ui) {
        ui.heading("MasterInstancedModelObject");
    }

    fn groups_list(&mut self) -> &mut Vec<super::ObjectGroup> {
        &mut self.groups
    }

    fn call(&mut self, _: &str, _: Vec<&str>) -> Option<String> {
        None
    }

    fn set_object_properties(&mut self, properties: HashMap<String, Vec<crate::managers::systems::SystemValue>>) {
        self.object_properties = properties.clone();
        crate::managers::systems::register_object_id_properties(self.object_id().to_owned(), properties);
    }

    fn object_properties(&self) -> &HashMap<String, Vec<crate::managers::systems::SystemValue>> {
        &self.object_properties
    }
}

impl MasterInstancedModelObject {
    pub fn cast_shadows(&mut self, cast: bool) {
        self.cast_shadows = cast;
    }

    fn model_object_transform(&self) -> Mat4 {
        let global_transform = self.global_transform();
        let global_rotation = deg_vec_to_rad(global_transform.rotation);
        let global_rotation =
            Quat::from_euler(glam::EulerRot::XYZ, global_rotation.x, global_rotation.y, global_rotation.z);

        Mat4::from_scale_rotation_translation(global_transform.scale, global_rotation, global_transform.position)
    }

    fn add_all_objects(&mut self, assets: &mut AssetManager, render: &mut RenderManager) {
        let mut objects_list: HashMap<String, Vec<RenderObjectData>> = HashMap::new();
        let asset = assets.get_model_asset(&self.model_asset_id);
        let shader = assets.get_shader_asset_mut(&self.shader);
        if let Some(asset) = asset {
            if let Some(shader) = shader {
                if let None = shader.program {
                    let program = Program::from_source(
                        &render.display,
                        &shader.vertex_shader_source,
                        &shader.fragment_shader_source,
                        None
                    );

                    let program = match program {
                        Ok(program) => program,
                        Err(err) => {
                            debugger::error(
                                &format!(
                                    "Failed to compile the shader! Error: {}\n - MasterInstancedModelObject's update", 
                                    err
                                )
                            );
                            self.error = true;
                            return
                        },
                    };
                    shader.program = Some(program);
                }

                self.add_objects_to_list(&render, asset.root.object_name.clone().unwrap_or(String::new()), &asset.root, &mut objects_list, None);
            }
        }

        render.add_object(self.id, objects_list);
    }

    fn add_objects_to_list(&mut self, render: &RenderManager, node_id: String, node: &ModelAssetObject, objects_list: &mut HashMap<String, Vec<RenderObjectData>>, parent_transform: Option<Mat4>) {
        let transparent = self.transparent;
        let layer = &self.layer;

        let parent_transform = match parent_transform {
            Some(parent_transform) => parent_transform,
            None => Mat4::IDENTITY,
        };
        let transform = parent_transform * node.default_transform;
        self.objects_global_transforms.insert(node_id.clone(), transform);
        
        objects_list.insert(node_id.clone(), Vec::new());
        for render_data in &node.render_data {

            let vbo = VertexBuffer::new(&render.display, &render_data.vertices)
                .expect("Failed to create a VBO!");
            let ibo = IndexBuffer::new(&render.display, PrimitiveType::TrianglesList, &render_data.indices)
                .expect("Failed to create an IBO!");

            let render_object_data = RenderObjectData {
                transform,
                transparent,
                uniforms: HashMap::new(),
                texture_asset_id: self.texture_asset_id.clone(),
                shader: self.shader.clone(),
                layer: layer.clone(),
                vbo,
                ibo,
                model_object_transform: Mat4::IDENTITY,
                instanced_master_name: Some(self.name.clone()),
                // gotta do 'em after setting all transforms
                joint_matrices: [[[0.0, 0.0, 0.0, 0.0]; 4]; 128],   
                joint_inverse_bind_matrices: [[[0.0, 0.0, 0.0, 0.0]; 4]; 128],
                cast_shadows: self.cast_shadows,
            };

            for (bone_name, offset) in render_data.bone_offsets.clone() {
                self.bone_offsets.insert(bone_name, offset);
            }

            objects_list.get_mut(&node_id).expect("add_objects_to_list err").push(render_object_data);
        }

        for (child_idx, child) in &node.children {
            self.add_objects_to_list(render, child_idx.to_owned(), child, objects_list, Some(transform));
        }
    }

    fn update_all_object_transforms(&mut self, asset: &ModelAsset) {
        self.update_children_transforms(&asset.animations, asset.root.object_name.clone().unwrap_or(String::new()), &asset.root, None)
    }

    fn update_children_transforms(&mut self, animations: &HashMap<String, ModelAssetAnimation>, node_id: String, node: &ModelAssetObject, parent_transform: Option<Mat4>) {
        let parent_transform = match parent_transform {
            Some(parent_transform) => parent_transform,
            None => Mat4::IDENTITY,
        };

        if let Some(animation_data) = &self.animation_data {
            if let Some(animation) = animations.get(&animation_data.animation_name) {
                if let Some(channel) = animation.channels.get(&node_id) {
                    let animation_time = animation_data.current_animation_frame;

                    // OH NO-
                    let translation = Vec3::new(
                        channel.translation_x.sample(animation_time).unwrap_or_else(|| {
                            match channel.translation_x.into_iter().last() {
                                Some(last) => last.value,
                                None => f32::default(),
                            }
                        }),
                        channel.translation_y.sample(animation_time).unwrap_or_else(|| {
                            match channel.translation_y.into_iter().last() {
                                Some(last) => last.value,
                                None => f32::default(),
                            }
                        }),
                        channel.translation_z.sample(animation_time).unwrap_or_else(|| {
                            match channel.translation_z.into_iter().last() {
                                Some(last) => last.value,
                                None => f32::default(),
                            }
                        }),
                    );
                    let rotation = Quat::from_xyzw(
                        channel.rotation_x.sample(animation_time).unwrap_or_else(|| {
                            match channel.rotation_x.into_iter().last() {
                                Some(last) => last.value,
                                None => f32::default(),
                            }
                        }),
                        channel.rotation_y.sample(animation_time).unwrap_or_else(|| {
                            match channel.rotation_y.into_iter().last() {
                                Some(last) => last.value,
                                None => f32::default(),
                            }
                        }),
                        channel.rotation_z.sample(animation_time).unwrap_or_else(|| {
                            match channel.rotation_z.into_iter().last() {
                                Some(last) => last.value,
                                None => f32::default(),
                            }
                        }),
                        channel.rotation_w.sample(animation_time).unwrap_or_else(|| {
                            match channel.rotation_w.into_iter().last() {
                                Some(last) => last.value,
                                None => f32::default(),
                            }
                        }),
                    );
                    let scale = Vec3::new(
                        channel.scale_x.sample(animation_time).unwrap_or_else(|| {
                            match channel.scale_x.into_iter().last() {
                                Some(last) => last.value,
                                None => f32::default(),
                            }
                        }),
                        channel.scale_y.sample(animation_time).unwrap_or_else(|| {
                            match channel.scale_y.into_iter().last() {
                                Some(last) => last.value,
                                None => f32::default(),
                            }
                        }),
                        channel.scale_z.sample(animation_time).unwrap_or_else(|| {
                            match channel.scale_z.into_iter().last() {
                                Some(last) => last.value,
                                None => f32::default(),
                            }
                        }),
                    );

                    let local_transform = Mat4::from_translation(translation)
                        * Mat4::from_quat(rotation)
                        * Mat4::from_scale(scale);
                    let transform = local_transform * parent_transform;
                    self.objects_global_transforms.insert(node_id, transform);

                    for (child_id, child) in &node.children {
                        self.update_children_transforms(animations, child_id.to_owned(), child, Some(transform));
                    }

                    return
                }
            }
        }

        let transform = node.default_transform * parent_transform;
        self.objects_global_transforms.insert(node_id, transform);

        for (child_id, child) in &node.children {
            self.update_children_transforms(animations, child_id.to_owned(), child, Some(transform));
        }
    }

    fn loop_animation_if_needed(&mut self, asset: &ModelAsset) {
        let mut animation_len = 0.0;
        let mut should_stop = false;
        if let Some(animation_data) = &mut self.animation_data {
            let animation_name = &animation_data.animation_name;
            match asset.animations.get(animation_name) {
                Some(animation) => {
                    for (_,channel) in &animation.channels {
                        if let Some(last_key) = channel.translation_x.keys().last() {
                            if last_key.t > animation_len {
                                animation_len = last_key.t;
                            }
                        }
                    }

                    let timer = animation_data.animation_timer.elapsed().as_secs_f32();
                    animation_data.current_animation_frame = timer;
                    if timer > animation_len {
                        if animation_data.looping {
                            animation_data.animation_timer = Instant::now();
                        } else {
                            animation_data.animation_ended = true;
                        }
                    }
                },
                None => {
                    should_stop = true;
                    debugger::error(
                        &format!("{}{}",
                            format!("Animation with name '{}' not found in model asset '{}'!", animation_name, asset.path),
                            "Setting current animation data to None - ModelObject (stop_animation_if_needed)"
                        )
                    )
                },
            }
        }
        if should_stop {
            self.animation_data = None;
        }
    }

    fn update_all_render_objects(&self, render: &mut RenderManager, asset: &ModelAsset) {
        let model_object_transform = self.model_object_transform();

        if let Some(render_object_list) = render.get_object(self.id) {
            self.update_children_render_objects(render_object_list, asset.root.object_name.clone().unwrap_or(String::new()), &asset.root, model_object_transform)
        }
    }

    fn update_children_render_objects(&self, render_object_list: &mut HashMap<String, Vec<RenderObjectData>>, node_id: String, node: &ModelAssetObject, model_object_transform: Mat4) {
        if let Some(render_object) = render_object_list.get_mut(&node_id) {
            for (node_data_idx, node_data) in node.render_data.iter().enumerate() {
                // list of values to change
                let mut joints = [[[0.0, 0.0, 0.0, 0.0]; 4]; 128];
                let joint_inverse_bind_matrices = [[[0.0, 0.0, 0.0, 0.0]; 4]; 128]; // idk if i
                                                                                    // need to use
                                                                                    // it anymore..

                for (joint_idx, node_name) in &node_data.bone_names {
                    /*let inv_bind_matrix = &node_data.inverse_bind_matrices[joint_obj_idx];
                    joint_inverse_bind_matrices[*joint_idx] = inv_bind_matrix.to_cols_array_2d();*/
                    joints[*joint_idx] = (self.objects_global_transforms[node_name] * self.bone_offsets[node_name]).to_cols_array_2d();
                }

                render_object[node_data_idx].joint_matrices = joints;
                render_object[node_data_idx].joint_inverse_bind_matrices
                    = joint_inverse_bind_matrices;
                render_object[node_data_idx].transform
                    = self.objects_global_transforms[&node_id];
                render_object[node_data_idx].model_object_transform
                    = model_object_transform;
            }
        }
        for (child_id, child) in &node.children {
            self.update_children_render_objects(render_object_list, child_id.to_owned(), child, model_object_transform);
        }
    }

    pub fn play_animation(&mut self, animation_name: String) {
        let mut looping = false;
        if let Some(animation_data) = &mut self.animation_data {
            looping = animation_data.looping;
        }

        self.animation_data = Some(CurrentAnimationData {
            animation_name,
            animation_timer: Instant::now(),
            looping,
            current_animation_frame: 0.0,
            animation_ended: false,
        });
    }

    pub fn stop_animation(&mut self) {
        self.animation_data = None;
    }

    pub fn set_looping(&mut self, looping: bool) {
        if let Some(animation_data) = &mut self.animation_data {
            animation_data.looping = looping;
        }
    }

    pub fn looping(&self) -> bool {
        match &self.animation_data {
            Some(animation_data) => animation_data.looping,
            None => false,
        }
    }

    pub fn current_animation(&self) -> Option<String> {
        match &self.animation_data {
            Some(animation_data) => {
                if animation_data.animation_ended {
                    None
                } else {
                    Some(animation_data.animation_name.clone())
                }
            },
            None => None,
        }
    }
}
