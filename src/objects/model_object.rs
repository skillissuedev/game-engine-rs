use std::{collections::HashMap, time::Instant};
use egui_glium::egui_winit::egui;
use glam::{Mat4, Quat, Vec3};
use glium::{index::PrimitiveType, IndexBuffer, Program, VertexBuffer};
use super::{gen_object_id, Object, ObjectGroup, Transform};
use crate::{assets::{model_asset::{ModelAsset, ModelAssetAnimation, ModelAssetObject}, shader_asset::ShaderAsset}, framework::{self, Framework}, managers::{assets::{AssetManager, ModelAssetId, TextureAssetId}, debugger, physics::ObjectBodyParameters, render::{RenderLayer, RenderManager, RenderObjectData, RenderShader, RenderUniformValue}}, math_utils::deg_vec_to_rad};

#[derive(Debug)]
pub struct CurrentAnimationData {
    pub animation_name: String,
    pub animation_timer: Instant,
    pub looping: bool,
    pub current_animation_frame: f32,
    pub animation_ended: bool,
}

#[derive(Debug)]
pub struct ModelObject {
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
    shader: ShaderAsset,
    layer: RenderLayer,
    transparent: bool,
    started: bool,
    error: bool,
    objects_global_transforms: HashMap<String, Mat4>,
    animation_data: Option<CurrentAnimationData>,
    uniforms_queue: HashMap<String, RenderUniformValue>,
    cast_shadows: bool,
    bone_offsets: HashMap<String, Mat4>,
}

impl ModelObject {
    pub fn new(name: &str, model_asset_id: ModelAssetId, texture_asset_id: Option<TextureAssetId>, shader_asset: ShaderAsset,
            layer: RenderLayer, transparent: bool) -> Self {
        let object_id = gen_object_id();

        ModelObject {
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
            shader: shader_asset,
            layer,
            transparent,
            started: false,
            error: false,
            objects_global_transforms: HashMap::new(),
            animation_data: None,
            uniforms_queue: HashMap::new(),
            cast_shadows: true,
            bone_offsets: HashMap::new(),
        }
    }
}

impl Object for ModelObject {
    fn start(&mut self) {}

    fn update(&mut self, _: &mut Framework) {}

    fn render(&mut self, framework: &mut Framework) {
        // we don't want to run render if the initialization of the object failed 
        if self.error == true { return }


        let assets = &framework.assets;
        let asset = framework.assets.get_model_asset(&self.model_asset_id);
        if let Some(asset) = asset {
            let render = framework
                .render.as_mut()
                .expect("No render manager! - ModelObject(render)");

            // initialize the object if it wasn't already
            if self.started == false {
                let shader_asset = self.shader.clone();
                self.add_all_objects(assets, render, shader_asset);
                self.started = true;

                for (uniform_name, uniform) in self.uniforms_queue.clone() {
                    self.add_uniform_render(render, uniform_name, uniform);
                }
                self.uniforms_queue.clear();
            }

            self.loop_animation_if_needed(asset);
            self.update_all_object_transforms(asset);
            self.update_all_render_objects(render, asset);
        } else {
            self.error = true;
            debugger::error(
                &format!(
                    "Failed to get the model asset with id '{}'! - ModelObject(render)",
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
        "ModelObject"
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
        ui.heading("ModelObject");
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

impl ModelObject {
    fn model_object_transform(&self) -> Mat4 {
        let global_transform = self.global_transform();
        let global_rotation = deg_vec_to_rad(global_transform.rotation);

        Mat4::from_translation(global_transform.position)
            * Mat4::from_rotation_z(global_rotation.z)
            * Mat4::from_rotation_y(global_rotation.y)
            * Mat4::from_rotation_x(global_rotation.x)
            * Mat4::from_scale(global_transform.scale)
    }

    fn add_all_objects(&mut self, assets: &AssetManager, render: &mut RenderManager, shader: ShaderAsset) {
        let mut objects_list: HashMap<String, Vec<RenderObjectData>> = HashMap::new();
        let asset = assets.get_model_asset(&self.model_asset_id);
        if let Some(asset) = asset {
            self.add_objects_to_list(&shader, render, asset.root.object_name.clone().unwrap_or(String::new()), &asset.root, &mut objects_list, None);
        }

        render.add_object(self.id, objects_list);
    }

    fn add_objects_to_list(&mut self, shader_asset: &ShaderAsset, render: &RenderManager, node_id: String, node: &ModelAssetObject, objects_list: &mut HashMap<String, Vec<RenderObjectData>>, parent_transform: Option<Mat4>) {
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
            let program = Program::from_source(
                &render.display,
                &shader_asset.vertex_shader_source,
                &shader_asset.fragment_shader_source,
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

            let vbo = VertexBuffer::new(&render.display, &render_data.vertices)
                .expect("Failed to create a VBO!");
            let ibo = IndexBuffer::new(&render.display, PrimitiveType::TrianglesList, &render_data.indices)
                .expect("Failed to create an IBO!");

            let render_object_data = RenderObjectData {
                transform,
                transparent,
                uniforms: HashMap::new(),
                texture_asset_id: self.texture_asset_id.clone(),
                shader: RenderShader::Program(program),
                layer: layer.clone(),
                vbo,
                ibo,
                model_object_transform: Mat4::IDENTITY,
                instanced_master_name: None,
                // gotta do 'em after setting all transforms
                joint_matrices: [[[0.0, 0.0, 0.0, 0.0]; 4]; 128],   
                joint_inverse_bind_matrices: [[[0.0, 0.0, 0.0, 0.0]; 4]; 128],
                cast_shadows: true,
            };

            objects_list.get_mut(&node_id).expect("add_objects_to_list err").push(render_object_data);

            for (bone_name, offset) in render_data.bone_offsets.clone() {
                self.bone_offsets.insert(bone_name, offset);
            }
        }

        for (child_idx, child) in &node.children {
            self.add_objects_to_list(shader_asset, render, child_idx.to_owned(), child, objects_list, Some(transform));
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

                    let transform = parent_transform * local_transform;
                    self.objects_global_transforms.insert(node_id, transform);

                    for (child_id, child) in &node.children {
                        self.update_children_transforms(animations, child_id.to_owned(), child, Some(transform));
                    }

                    return
                }
            }
        }

        let transform = parent_transform * node.default_transform;
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
            self.update_children_render_objects(render_object_list, asset.root.object_name.clone().unwrap_or_default(), &asset.root, model_object_transform, self.cast_shadows)
        }
    }

    fn update_children_render_objects(&self, render_object_list: &mut HashMap<String, Vec<RenderObjectData>>, node_id: String, node: &ModelAssetObject, model_object_transform: Mat4, cast_shadows: bool) {
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
                render_object[node_data_idx].cast_shadows
                    = cast_shadows;
            }
        }
        for (child_id, child) in &node.children {
            self.update_children_render_objects(render_object_list, child_id.to_owned(), child, model_object_transform, cast_shadows);
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

    pub fn cast_shadows(&mut self, cast: bool) {
        self.cast_shadows = cast;
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

    fn add_uniform_render(&mut self, render: &mut RenderManager, name: String, value: RenderUniformValue) {
        if let Some(object) = render.get_object(self.id) {
            for (_, object) in object {
                for object_data in object {
                    object_data.uniforms.insert(name.clone(), value.clone());
                }
            }
        }
    }

    pub(crate) fn add_uniform(&mut self, framework: &mut Framework, name: String, value: RenderUniformValue) {
        if !self.started {
            self.uniforms_queue.insert(name, value);
            return
        }

        if let Some(render) = &mut framework.render {
            if let Some(object) = render.get_object(self.id) {
                for (_, object) in object {
                    for object_data in object {
                        object_data.uniforms.insert(name.clone(), value.clone());
                    }
                }
            }
        }
    }
}

impl Drop for ModelObject {
    fn drop(&mut self) {
        let framework_ptr: *mut Framework = unsafe { framework::FRAMEWORK_POINTER } as *mut Framework;
        let framework = unsafe { &mut *framework_ptr };
        if let Some(render) = &mut framework.render {
            render.remove_object(self.object_id());
        }
    }
}
