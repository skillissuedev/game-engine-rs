use std::{collections::HashMap, time::Instant};
use egui_glium::egui_winit::egui;
use glam::{Mat4, Quat, Vec3};
use rand::Rng;
use super::{gen_object_id, Object, ObjectGroup, Transform};
use crate::{framework::Framework, managers::physics::ObjectBodyParameters, math_utils::deg_vec_to_rad};

#[derive(Debug)]
pub struct ParticleSystem {
    name: String,
    transform: Transform,
    parent_transform: Option<Transform>,
    children: Vec<Box<dyn Object>>,
    body: Option<ObjectBodyParameters>,
    id: u128,
    groups: Vec<ObjectGroup>,
    object_properties: HashMap<String, Vec<crate::managers::systems::SystemValue>>,
    master_object: String,

    particle_count: u32,
    gravity: f32,
    life_seconds: f32,
    random_factor: f32,
    particles: Vec<ParticleData>,
    max_particle_distance: Option<f32>,
}

#[derive(Debug)]
struct ParticleData {
    position: Vec3,
    rotation: Vec3,
    gravity: f32,
    scale: Vec3,
    velocity: Vec3,
    life_timer: Instant,
}

impl ParticleSystem {
    pub fn new(name: &str, master_object: String, particle_count: u32, gravity: f32, random_factor: f32, life_seconds: f32) -> Self {
        let object_id = gen_object_id();

        ParticleSystem {
            transform: Transform::default(),
            children: vec![],
            name: name.to_string(),
            parent_transform: None,
            body: None,
            id: object_id,
            groups: vec![],
            object_properties: HashMap::new(),
            master_object,
            particle_count,
            gravity,
            life_seconds,
            random_factor,
            particles: Vec::new(),
            max_particle_distance: None,
        }
    }
}

impl Object for ParticleSystem {
    fn start(&mut self) {}

    fn update(&mut self, _: &mut Framework) {}

    fn render(&mut self, framework: &mut Framework) {
        let delta_time = framework.delta_time().as_secs_f32();

        if let Some(render) = &mut framework.render {
            let max_distance_squared = match self.max_particle_distance {
                Some(max_distance) => Some(max_distance * max_distance),
                None => None,
            };
            let camera_position = render.camera_position();

            // leave only particles for which the life time hasn't passed yet and those that aren't
            // too far away
            self.particles.retain(|particle| {
                let are_alive = particle.life_timer.elapsed().as_secs_f32() < self.life_seconds;
                let are_close_enough = match max_distance_squared {
                    Some(max_distance_squared) => {
                        let distance_squared = particle
                            .position
                            .distance_squared(camera_position);

                        distance_squared < max_distance_squared
                    },
                    None => true,
                };

                are_alive && are_close_enough
            });

            // update their positions
            for particle in &mut self.particles {
                particle.position.x += particle.velocity.x * delta_time;
                particle.position.y += (particle.velocity.y + particle.gravity) * delta_time;
                particle.position.z += particle.velocity.z * delta_time;
            }

            // update their positions in a RenderManager
            match render.instanced_positions.get_mut(&self.master_object) {
                Some(instanced_positions) => {
                    for particle in &self.particles {
                        let rotation_rads = deg_vec_to_rad(particle.rotation);
                        let rotation_quat = 
                            Quat::from_euler(glam::EulerRot::XYZ, rotation_rads.x, rotation_rads.y, rotation_rads.z);

                        let transform = Mat4::from_scale_rotation_translation(
                            particle.scale, rotation_quat, particle.position,
                        );

                        instanced_positions.push(transform);
                    }
                },
                None => {
                    let mut instanced_positions = Vec::new();
                    for particle in &self.particles {
                        let rotation_rads = deg_vec_to_rad(particle.rotation);
                        let rotation_quat = 
                            Quat::from_euler(glam::EulerRot::XYZ, rotation_rads.x, rotation_rads.y, rotation_rads.z);

                        let transform = Mat4::from_scale_rotation_translation(
                            particle.scale, rotation_quat, particle.position,
                        );

                        instanced_positions.push(transform);
                    }

                    render.instanced_positions
                        .insert(self.master_object.clone(), instanced_positions);
                },
            }
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
        "ParticleSystem"
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
        ui.heading("ParticleSystem");
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

impl ParticleSystem {
    pub fn start_particles(&mut self, base_velocity: Vec3, random_velocity_scale: f32) {
        let position = self.global_transform().position;
        let rotation = self.global_transform().rotation;
        let scale = self.global_transform().scale.x;

        for _ in 0..self.particle_count {
            let randomized_scale = scale + (scale * ((rand::thread_rng().gen::<f32>() * 2.0 - 1.0) * self.random_factor));

            let direction_x: f32 = rand::thread_rng().gen::<f32>() * 2.0 - 1.0;
            let direction_z: f32 = rand::thread_rng().gen::<f32>() * 2.0 - 1.0;

            let mut velocity = base_velocity;
            velocity.x += direction_x * random_velocity_scale;
            velocity.z += direction_z * random_velocity_scale;

            self.particles.push(ParticleData { 
                position,
                rotation,
                scale: Vec3::new(randomized_scale, randomized_scale, randomized_scale),
                velocity,
                gravity: self.gravity,
                life_timer: Instant::now(),
            });
        }
    }

    pub fn set_max_particle_distance(&mut self, max_particle_distance: f32) {
        self.max_particle_distance = Some(max_particle_distance);
    }
}
