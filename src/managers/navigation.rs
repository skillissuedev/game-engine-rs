use std::{collections::HashMap, sync::{Arc, Mutex, RwLock}};

use glam::{Vec2, Vec3};
use landmass::{Agent, AgentId, Archipelago, ArchipelagoOptions, Character, CharacterId, CoordinateSystem, FromAgentRadius, Island, IslandId, NavigationMesh, PointSampleDistance3d, TargetReachedCondition, ValidNavigationMesh, ValidationError};

use crate::{managers::debugger, objects::{nav_object::NavObjectData, Transform}};

use super::assets::AssetManager;

pub struct NavigationManager {
    objects: Arc<RwLock<HashMap<u128, Vec<IslandId>>>>,
    archipelago: Arc<Mutex<Archipelago<XYZFlip>>>,
    characters: HashMap<u128, CharacterId>,
    agents: HashMap<u128, AgentId>,
}

impl NavigationManager {
    pub fn new() -> NavigationManager {
        let archipelago = Arc::new(Mutex::new(Archipelago::new(ArchipelagoOptions::from_agent_radius(1.0))));

        Self {
            objects: Arc::new(RwLock::new(HashMap::new())),
            archipelago,
            characters: HashMap::new(),
            agents: HashMap::new(),
        }
    }

    pub fn add_object(&mut self, assets: &AssetManager, id: u128, data: NavObjectData, transform: Transform) {
        match &data {
            NavObjectData::StaticMesh(model_asset_id) => {
                let pos = transform.position;
                let rot = transform.rotation;

                struct NavmeshBuildData {
                    vertices: Vec<landmass::Vec3>,
                    polygons: Vec<Vec<usize>>,
                    polygon_type_indices: Vec<usize>
                }

                let mut build_data = vec![];

                match assets.get_model_asset(model_asset_id) {
                    Some(asset) => {
                        for data in &asset.root.render_data {
                            let mut current_build_data = NavmeshBuildData {
                                vertices: vec![],
                                polygons: vec![],
                                polygon_type_indices: vec![],
                            };
                            for vertex in &data.vertices {
                                let pos = vertex.position;
                                current_build_data.vertices.push(landmass::Vec3::new(pos[0], pos[2], pos[1]));
                            }

                            for index in data.indices.chunks_exact(3) {
                                current_build_data.polygons.push(vec![index[0] as usize, index[1] as usize, index[2] as usize]);
                                current_build_data.polygon_type_indices.push(0);
                            }

                            build_data.push(current_build_data);
                            break
                        }
                    },
                    None => {
                        debugger::error("Failed to create a navmesh! Failed to get the required model asset.");
                        return
                    },
                };


                self.objects.write().expect("objects was poisoned :c").insert(id, vec![]);
                for build_data in build_data {
                    let archipelago = self.archipelago.clone();
                    let objects = self.objects.clone();

                    std::thread::spawn(move || {
                        let transform: landmass::Transform<XYZFlip> = landmass::Transform {
                            translation: landmass::Vec3::new(pos.x, pos.z, pos.y),
                            rotation: rot.y.to_radians(), //???
                        };

                        let navmesh = NavigationMesh {
                            vertices: build_data.vertices,
                            polygons: build_data.polygons,
                            polygon_type_indices: build_data.polygon_type_indices,
                            height_mesh: None,
                        };

                        match validate_navmesh(navmesh, 0) {
                            Some(navmesh) => {
                                if let Ok(navmesh) = navmesh {
                                    let island_id = archipelago.lock().expect("archipelago was poisoned :(").add_island(
                                        Island::new(transform, navmesh.into())
                                    );
                                    objects.write().expect("objects was poisoned :c").get_mut(&id)
                                        .expect("failed to open the vec of object's island ids")
                                        .push(island_id);
                                }
                            },
                            None => (),
                        }
                    });
                }
            },
            NavObjectData::DynamicCapsule(radius) => {
                let pos = transform.position;
                let character_id = self.archipelago.lock().expect("archipelago was poisoned :(").add_character(Character {
                    position: landmass::Vec3::new(pos.x, pos.z, pos.y),
                    velocity: landmass::Vec3::ZERO,
                    radius: *radius,
                });
                self.characters.insert(id, character_id);
            },
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.archipelago.is_poisoned() {
            self.archipelago.clear_poison();
        }

        if let Ok(mut archipelago) = self.archipelago.try_lock() {
            archipelago.update(delta_time);
            for agent_id in archipelago.get_agent_ids().collect::<Vec<_>>() {
                let agent = archipelago.get_agent_mut(agent_id)
                    .expect("No agent for some reason?");
                agent.velocity = *agent.get_desired_velocity();
            }
        }
    }

    pub fn set_island_transform(&mut self, idx: u128, transform: Transform) {
        match self.objects.read().unwrap().get(&idx) {
            Some(islands) => {
                for island in islands {
                    match self.archipelago.lock().unwrap().get_island_mut(*island) {
                        Some(mut island) => {
                            let pos = transform.position;
                            let transform: landmass::Transform<XYZFlip> = landmass::Transform {
                                translation: landmass::Vec3::new(pos.x, pos.z, pos.y),
                                rotation: transform.rotation.y.to_radians(),
                            };

                            island.set_transform(transform);
                        },
                        None => debugger::error("Failed to update the island transform! Failed to get the island."),
                    }
                }
            },
            None => debugger::error("Failed to update the island transform! Failed to get the island id."),
        }
    }

    pub fn set_character_position(&mut self, idx: u128, position: Vec3) {
        match self.characters.get(&idx) {
            Some(character) => {
                match self.archipelago.lock().unwrap().get_character_mut(*character) {
                    Some(character) => {
                        let position = landmass::Vec3::new(position.x, position.z, position.y);

                        character.position = position;
                    },
                    None => debugger::error("Failed to update the character position! Failed to get the character."),
                }
            },
            None => debugger::error("Failed to update the character position! Failed to get the character id."),
        }
    }

    pub fn add_agent(&mut self, idx: u128, speed: f32, position: Vec3, radius: f32) {
        let mut position = landmass::Vec3::from_array(position.to_array());
        let y = position.y;
        position.y = position.z;
        position.z = y;

        let mut archipelago = self.archipelago.lock().unwrap();
        let sample_point = archipelago.sample_point(position, &PointSampleDistance3d {
            horizontal_distance: 0.1, distance_above: 100.0, distance_below: 100.0, vertical_preference_ratio: 0.0, animation_link_max_vertical_distance: 0.0, });

        match sample_point {
            Ok(position) => {
                let position = position.point();

                let mut agent = Agent::create(position/*landmass::Vec3::ZERO*/, landmass::Vec3::ZERO, radius, speed, speed);
                agent.target_reached_condition = TargetReachedCondition::StraightPathDistance(Some(0.5));
                self.agents.insert(idx, archipelago.add_agent(agent));
            },
            Err(err) => {
                dbg!(err);
            },
        }
    }

    pub fn set_agent_position(&mut self, idx: u128, position: Vec3) {
        match self.agents.get(&idx) {
            Some(agent) => {
                let mut position = landmass::Vec3::from_array(position.to_array());
                let y = position.y;
                position.y = position.z;
                position.z = y;

                let mut archipelago = self.archipelago.lock().unwrap();
                let sampled_position = archipelago.sample_point(position, &PointSampleDistance3d {
                    horizontal_distance: 0.1, distance_above: 10.0, distance_below: 10.0, vertical_preference_ratio: 0.0, animation_link_max_vertical_distance: 0.0, });
                let position: landmass::Vec3;

                match sampled_position {
                    Ok(sampled_position) => {
                        position = sampled_position.point();
                    },
                    Err(err) => {
                        debugger::error(&format!("set_agent_position failed! Failed to sample the point! Error: {}", err));
                        return;
                    },
                }

                match archipelago.get_agent_mut(*agent) {
                    Some(agent) => {
                        agent.position = position;
                    },
                    None => debugger::error("set_agent_position failed! Failed to get the agent!"),
                }
            }
            None => debugger::error("set_agent_position failed! Failed to get the agent id!"),
        }
    }

    pub fn get_agent_velocity(&self, idx: u128) -> Option<Vec3> {
        match self.agents.get(&idx) {
            Some(agent) => {
                let archipelago = self.archipelago.lock().unwrap();
                match archipelago.get_agent(*agent) {
                    Some(agent) => {
                        let mut velocity = Vec3::from_array(agent.velocity.to_array());
                        let y = velocity.y;
                        velocity.y = velocity.z;
                        velocity.z = y;
                        /*dbg!(velocity);
                        dbg!(agent.current_target);
                        dbg!(agent.state());*/
                        Some(velocity)
                    },
                    None => {
                        debugger::error("get_agent_velocity failed! Failed to get the agent!");
                        None
                    },
                }
            }
            None => {
                debugger::error("get_agent_velocity failed! Failed to get the agent id!");
                None
            },
        }
    }

    pub fn set_agent_target(&mut self, idx: u128, target: Option<Vec3>) {
        match self.agents.get(&idx) {
            Some(agent) => {
                match target {
                    Some(target) => {
                        let mut target = landmass::Vec3::from_array(target.to_array());
                        let y = target.y;
                        target.y = target.z;
                        target.z = y;

                        let mut archipelago = self.archipelago.lock().unwrap();
                        let sampled_target = archipelago.sample_point(target, &PointSampleDistance3d {
                            horizontal_distance: 0.5, distance_above: 100.0, distance_below: 100.0, vertical_preference_ratio: 0.0, animation_link_max_vertical_distance: 0.0, });

                        let target: landmass::Vec3;

                        match sampled_target {
                            Ok(sampled_target) => {
                                target = sampled_target.point();
                            },
                            Err(err) => {
                                debugger::error(&format!("set_agent_target failed! Failed to sample the point! Error: {}", err));
                                return;
                            },
                        }

                        match archipelago.get_agent_mut(*agent) {
                            Some(agent) => {
                                agent.current_target = Some(target);
                            },
                            None => debugger::error("set_agent_target failed! Failed to get the agent!"),
                        }
                    },
                    None => {
                        let mut archipelago = self.archipelago.lock().unwrap();
                        match archipelago.get_agent_mut(*agent) {
                            Some(agent) => {
                                agent.current_target = None;
                            },
                            None => debugger::error("set_agent_target failed! Failed to get the agent!"),
                        }
                    },
                }
            }
            None => debugger::error("set_agent_target failed! Failed to get the agent id!"),
        }
    }

    pub fn get_agent_position(&mut self, idx: u128) -> Option<Vec3> {
        match self.agents.get(&idx) {
            Some(agent) => {
                match self.archipelago.lock().unwrap().get_agent(*agent) {
                    Some(agent) => {
                        Some(Vec3::from_array(agent.position.to_array()))
                    },
                    None => {
                        debugger::error("get_agent_position failed! Failed to get the agent!");
                        None
                    },
                }
            },
            None => {
                debugger::error("get_agent_position failed! Failed to get the agent id!");
                None
            },
        }
    }

    // old:
    pub fn add_navmesh(&mut self, id: u128, dimensions: NavMeshDimensions) {
    }

    pub fn add_obstacle(&mut self, transform: NavMeshObstacleTransform) {
    }

    pub fn create_grids(&mut self) {
    }

    pub fn find_path(&self, start_point: Vec2, finish_point: Vec2) -> Option<Vec<Vec2>> {
        None
    }
}

fn validate_navmesh(navmesh: NavigationMesh<XYZFlip>, err_count: usize) -> Option<Result<ValidNavigationMesh<XYZFlip>, NavigationMesh<XYZFlip>>> {
    let mut navmesh = navmesh;
    match navmesh.clone().validate() {
        Ok(valid_navmesh) => {
            Some(Ok(valid_navmesh))
        },
        Err(err) => {
            match err {
                ValidationError::ConcavePolygon(idx) => {
                    if err_count > 1000 {
                        debugger::error("Failed to create a navmesh from a mesh! Navmesh validation error (> 1000 errors)");
                        return None
                    }

                    navmesh.polygons.remove(idx);
                    navmesh.polygon_type_indices.remove(idx);

                    return validate_navmesh(navmesh, err_count + 1)
                },
                _ => {
                    debugger::error("Failed to create a navmesh! Navmesh validation error.");
                    None
                }
            }
        },
    }
}


#[derive(Debug, Clone)]
pub struct NavMeshDimensions {
    pub position: [i32; 2],
    pub area_size: [i32; 2],
}

impl NavMeshDimensions {
    pub fn new(position: Vec2, size: Vec2) -> NavMeshDimensions {
        let position_x = position.x.round() as i32;
        let position_z = position.y.round() as i32;
        let position = [position_x, position_z];

        let size_x = size.x.round() as i32;
        let size_z = size.y.round() as i32;
        let area_size = [size_x, size_z];

        NavMeshDimensions {
            position,
            area_size,
        }
    }

    pub fn set_position(&mut self, position: Vec2) {
        let position_x = position.x.round() as i32;
        let position_z = position.y.round() as i32;

        self.position = [position_x, position_z];
    }
}

#[derive(Debug, Clone)]
/// all positions are in grid coords
pub struct NavMeshObstacleTransform {
    pub position_x: i32,
    pub position_z: i32,
    pub area_size: [i32; 2],
}

impl NavMeshObstacleTransform {
    pub fn new(position: Vec2, size: Vec2) -> NavMeshObstacleTransform {
        let position_x = position.x.round() as i32;
        let position_z = position.y.round() as i32;

        let size_x = size.x.round() as i32;
        let size_z = size.y.round() as i32;
        let area_size = [size_x, size_z];

        NavMeshObstacleTransform {
            position_x,
            position_z,
            area_size,
        }
    }
}

pub struct XYZFlip;

impl CoordinateSystem for XYZFlip {
    type Coordinate = landmass::Vec3;
    type SampleDistance = PointSampleDistance3d;

    const FLIP_POLYGONS: bool = true;

    fn to_landmass(v: &Self::Coordinate) -> landmass::Vec3 {
        *v
    }

    fn from_landmass(v: &landmass::Vec3) -> Self::Coordinate {
        *v
    }
}
