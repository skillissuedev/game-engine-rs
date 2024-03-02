use std::collections::HashMap;
use basic_pathfinding::{coord::Coord, pathfinding::{find_path, SearchOpts}};
use das_grid::Grid;
use glam::Vec2;
use once_cell::sync::Lazy;

use crate::managers::debugger;

/// u128 is object's id
static mut NAVMESH_DIMENSIONS: Lazy<HashMap<u128, NavMeshDimensions>> = Lazy::new(|| HashMap::new());
/// u128 is object's id
static mut NAVMESH_OBSTACLES: Lazy<HashMap<u128, Vec<NavMeshObstacleTransform>>> = Lazy::new(|| HashMap::new());
//static mut NAVMESH_GRIDS: Lazy<HashMap<u128, Grid<Option<()>>>> = Lazy::new(|| HashMap::new());
static mut NAVMESH_GRIDS: Lazy<HashMap<u128, Grid<bool>>> = Lazy::new(|| HashMap::new());

#[derive(Debug, Clone)]
pub struct NavMeshDimensions {
    pub position: Vec2,
    pub area_size_world: Vec2,
    pub x_cells_count: i32,
    pub z_cells_count: i32,
}

#[derive(Debug, Clone)]
pub struct NavMeshObstacleTransform {
    pub position_x: i32,
    pub position_z: i32,
    pub area_size_world: Vec2,
}

impl NavMeshObstacleTransform {
    pub fn new(position: Vec2, size: Vec2) -> NavMeshObstacleTransform {
        let position_x = position.x.round() as i32;
        let position_z = position.y.round() as i32;
        
        NavMeshObstacleTransform {
            position_x,
            position_z,
            area_size_world: size,
        }
    }
}

pub fn add_navmesh(id: u128, dimensions: NavMeshDimensions) {
    unsafe {
        NAVMESH_DIMENSIONS.insert(id, dimensions);
    }
}

pub fn add_obstacle(transform: NavMeshObstacleTransform) {
    unsafe {
        for (navmesh_id, dim) in NAVMESH_DIMENSIONS.iter() {
            let round_pos_x = dim.position.x.round() as i32;
            let round_pos_z = dim.position.y.round() as i32;
            let round_area_size_x = dim.area_size_world.x.round() as i32;
            let round_area_size_z = dim.area_size_world.y.round() as i32;

            let obstacle_pos_x = &transform.position_x;
            let obstacle_pos_z = &transform.position_z;

            let x1 = round_pos_x - round_area_size_x / 2;
            let x2 = round_pos_x + round_area_size_x / 2;
            let z1 = round_pos_z - round_area_size_z / 2;
            let z2 = round_pos_z + round_area_size_z / 2;
  
            //dbg!(x1, x2, z1, z2);
            //dbg!(obstacle_pos_x, obstacle_pos_z);
            if *obstacle_pos_x >= x1 && *obstacle_pos_x <= x2 && *obstacle_pos_z >= z1 && *obstacle_pos_z <= z2 {
                match NAVMESH_OBSTACLES.get_mut(navmesh_id) {
                    Some(obstacles) => obstacles.push(transform),
                    None => {
                        NAVMESH_OBSTACLES.insert(*navmesh_id, vec![transform]);
                    },
                }

                return;
            }
        }
    }
}

pub fn update() {
    unsafe {
        NAVMESH_OBSTACLES.clear();
    }
}

pub fn create_grids() {
    unsafe {
        NAVMESH_GRIDS.clear();
        for (id, navmesh) in NAVMESH_DIMENSIONS.iter() {
            let round_pos_x = navmesh.position.x.round() as i32;
            let round_pos_z = navmesh.position.y.round() as i32;
            let round_area_size_x = navmesh.area_size_world.x.round() as i32;
            let round_area_size_z = navmesh.area_size_world.y.round() as i32;

            let x1 = round_pos_x - round_area_size_x / 2;
            let z1 = round_pos_z - round_area_size_z / 2;

            NAVMESH_GRIDS.insert(*id, Grid::new(navmesh.x_cells_count, navmesh.z_cells_count, true));
            match NAVMESH_OBSTACLES.get(id) {
                Some(obstacles) => {
                    for obstacle in obstacles {
                        let obstacle_x1 = 
                            (obstacle.position_x as f32 - obstacle.area_size_world.x / 2.0).round() as i32;
                        //let obstacle_x2 = 
                        //    (obstacle.position_x as f32 + obstacle.area_size_world.x / 2.0).round() as i32;
                        let obstacle_z1 = 
                            (obstacle.position_z as f32 - obstacle.area_size_world.y / 2.0).round() as i32;
                        //let obstacle_z2 = 
                        //    (obstacle.position_z as f32 + obstacle.area_size_world.y / 2.0).round() as i32;

                        let mut relative_x_pos = (obstacle_x1.abs_diff(x1) as f32 / 2.0).round() as i32 - 1;
                        let mut relative_x2_pos = (relative_x_pos as f32 + (obstacle.area_size_world.x / 2.0 / 2.0 - 1.0)).round() as i32;
                        let mut relative_z_pos = (obstacle_z1.abs_diff(z1) as f32 / 2.0).round() as i32 - 1;
                        let mut relative_z2_pos = (relative_z_pos as f32 + (obstacle.area_size_world.y / 2.0 / 2.0 - 1.0)).round() as i32;

                        //dbg!(relative_x_pos, relative_x2_pos, relative_z_pos, relative_z2_pos);
                        if relative_x_pos < 0 {
                            relative_x_pos = 0;
                        }
                        if relative_x2_pos < 0 {
                            relative_x2_pos = 0;
                        }
                        if relative_z_pos < 0 {
                            relative_z_pos = 0;
                        }
                        if relative_z2_pos < 0 {
                            relative_z2_pos = 0;
                        }
                        //dbg!(relative_x_pos, relative_x2_pos, relative_z_pos, relative_z2_pos);

                        for x in relative_x2_pos..=relative_x_pos {
                            for z in relative_z2_pos..=relative_z_pos {
                                let grid = NAVMESH_GRIDS.get_mut(id).unwrap();
                                match grid.set((x, z), &false) {
                                    Ok(_) => (),
                                    Err(err) => debugger::error(&format!("Navigation error!\nfailed to set obstacle position on grid!\nerr: {}", err)),
                                };
                            }
                        }
                    }
                },
                None => {
                    break;
                },
            }
        }

        //dbg!(&NAVMESH_GRIDS[&3]);
    }
}


pub fn find_path_map(start_world: Vec2, finish_world: Vec2) -> Option<Vec2> {
    //println!("path");
    unsafe {
        for (navmesh_id, dim) in NAVMESH_DIMENSIONS.iter() {
            //println!("{}", navmesh_id);
            let pos_x = dim.position.x;
            let pos_z = dim.position.y;

            let round_pos_x = dim.position.x.round() as i32;
            let round_pos_z = dim.position.y.round() as i32;

            let area_size_x = dim.area_size_world.x;
            let area_size_z = dim.area_size_world.y;

            let round_area_size_x = dim.area_size_world.x.round() as i32;
            let round_area_size_z = dim.area_size_world.y.round() as i32;

            let start_x = &start_world.x;
            let start_z = &start_world.y;

            let round_start_x = start_x.round() as i32;
            let round_start_z = start_z.round() as i32;

            let finish_x = &finish_world.x;
            let finish_z = &finish_world.y;

            let round_finish_x = finish_x.round() as i32;
            let round_finish_z = finish_z.round() as i32;

            let x1 = pos_x - area_size_x / 2.0;
            let round_x1 = (pos_x - area_size_x / 2.0) as i32;
            let x2 = pos_x + area_size_x / 2.0;
            let z1 = pos_z - area_size_z / 2.0;
            let round_z1 = (pos_z - area_size_z / 2.0) as i32;
            let z2 = pos_z + area_size_z / 2.0;
  
            //dbg!(x1, x2, z1, z2);
            //dbg!(obstacle_pos_x, obstacle_pos_z);
            if *start_x >= x1 && *start_x <= x2 && *start_z >= z1 && *start_z <= z2
                && *finish_x >= x1 && *finish_x <= x2 && *finish_z >= z1 && *finish_z <= z2 {
                    let relative_start_x_pos = (round_start_x.abs_diff(round_x1) as f32 / 2.0).round() as i32 - 1;
                    let relative_start_z_pos = (round_start_z.abs_diff(round_x1) as f32 / 2.0).round() as i32 - 1;

                    let relative_finish_x_pos = (round_finish_x.abs_diff(round_x1) as f32 / 2.0).round() as i32 - 1;
                    let relative_finish_z_pos = (round_finish_z.abs_diff(round_x1) as f32 / 2.0).round() as i32 - 1;
                    //dbg!(&relative_start_x_pos);
                    //dbg!(&relative_start_z_pos);
                    //dbg!(&relative_finish_x_pos);
                    //dbg!(&relative_finish_z_pos);

                    let navmesh_grid = NAVMESH_GRIDS.get(navmesh_id);
                    match navmesh_grid {
                        Some(grid) => {
                            let mut astar_grid: basic_pathfinding::grid::Grid = basic_pathfinding::grid::Grid {
                                tiles: vec![],
                                walkable_tiles: vec![1],
                                grid_type: basic_pathfinding::grid::GridType::Intercardinal,
                                ..Default::default()
                            };

                            for (x, y) in grid.enumerate() {
                                let val = grid.get((x, y)).unwrap();
                                if x >= astar_grid.tiles.len() as i32 {
                                    astar_grid.tiles.push(vec![]);
                                }

                                let astar_grid_x = &mut astar_grid.tiles[x as usize];
                                if y >= astar_grid_x.len() as i32 {
                                    astar_grid_x.push(1);
                                }
                                match val  {
                                    true => astar_grid_x[y as usize] = 1,
                                    false => astar_grid_x[y as usize] = 0,
                                }
                            }
                            //dbg!(&astar_grid.tiles);
                            //dbg!(&grid);
                            let start_point = Coord::new(relative_start_x_pos, relative_start_z_pos);

                            let finish_point = Coord::new(relative_finish_x_pos, relative_finish_z_pos);
                            let next_point = find_path(&astar_grid, start_point, finish_point, SearchOpts::default());
                            println!("next_point:");
                            return match next_point {
                                Some(next) => {
                                    //dbg!(&grid);
                                    if let Some(next) = next.get(1) {
                                        let next_x = x1 + next.x as f32 * 2.0;
                                        let next_z = z1 + next.y as f32 * 2.0;

                                        Some(Vec2::new(next_x, next_z))
                                    } else {
                                        None
                                    }
                                },
                                None => None
                            }
                        },
                        None => {
                            debugger::error("navigation error\nfailed to find a path\ncan't get a grid for a*");
                        },
                    }
            }
        }
    }
    None
}

