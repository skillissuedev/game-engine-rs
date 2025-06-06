use glam::Vec2;
use grid_pathfinding::PathingGrid;
use grid_util::{Grid, Point, Rect};
use std::collections::HashMap;

#[derive(Default)]
pub struct NavigationManager {
    // u128 is object's id
    navmesh_dimensions: HashMap<u128, NavMeshDimensions>,
    // u128 is navmesh's id
    navmesh_obstacles: HashMap<u128, Vec<NavMeshObstacleTransform>>,
    navmesh_grids: HashMap<u128, PathingGrid>,
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

impl NavigationManager {
    pub fn add_navmesh(&mut self, id: u128, dimensions: NavMeshDimensions) {
        self.navmesh_dimensions.insert(id, dimensions);
        self.create_grids();
    }

    pub fn add_obstacle(&mut self, transform: NavMeshObstacleTransform) {
        let obstacle_x1 = transform.position_x - transform.area_size[0] / 2;
        let obstacle_x2 = transform.position_x + transform.area_size[0] / 2;
        let obstacle_z1 = transform.position_z - transform.area_size[1] / 2;
        let obstacle_z2 = transform.position_z + transform.area_size[1] / 2;

        for (navmesh_id, navmesh_dim) in self.navmesh_dimensions.iter() {
            let navmesh_x1 = navmesh_dim.position[0] - navmesh_dim.area_size[0] as i32 / 2;
            let navmesh_x2 = navmesh_dim.position[0] + navmesh_dim.area_size[0] as i32 / 2;
            let navmesh_z1 = navmesh_dim.position[1] - navmesh_dim.area_size[1] as i32 / 2;
            let navmesh_z2 = navmesh_dim.position[1] + navmesh_dim.area_size[1] as i32 / 2;

            // if obstacle is on this navmesh
            if (obstacle_x1 >= navmesh_x1 && obstacle_x2 <= navmesh_x2)
                && (obstacle_z1 >= navmesh_z1 && obstacle_z2 <= navmesh_z2)
            {
                match self.navmesh_obstacles.get_mut(navmesh_id) {
                    Some(obstacles) => {
                        obstacles.push(transform);
                        //dbg!(obstacles);
                    },
                    None => {
                        self.navmesh_obstacles.insert(*navmesh_id, vec![transform]);
                    }
                }

                break;
            }
        }
    }

    pub fn update(&mut self) {
        self.create_grids();
        self.navmesh_obstacles.clear();
    }

    pub fn create_grids(&mut self) {
        for (navmesh_id, dim) in self.navmesh_dimensions.iter() {
            let navmesh_position_x = dim.position[0];
            let navmesh_position_z = dim.position[1];
            let area_size_x = dim.area_size[0];
            let area_size_z = dim.area_size[1];

            let navmesh_x1 = navmesh_position_x - area_size_x / 2;
            let navmesh_z1 = navmesh_position_z - area_size_z / 2;

            let mut grid = PathingGrid::new(area_size_x as usize, area_size_z as usize, false);

            for obstacle in self
                .navmesh_obstacles
                .get(navmesh_id)
                .unwrap_or(&Vec::new())
            {
                let obstacle_size_x = obstacle.area_size[0];
                let obstacle_size_z = obstacle.area_size[1];

                let obstacle_x1 = obstacle.position_x - obstacle_size_x / 2;
                let obstacle_z1 = obstacle.position_z - obstacle_size_z / 2;
                let obstacle_x2 = obstacle.position_x + obstacle_size_x / 2;
                let obstacle_z2 = obstacle.position_z + obstacle_size_z / 2;
                //dbg!(obstacle_x1, obstacle_x2);

                let distance_to_obstacle_x1 = navmesh_x1.abs_diff(obstacle_x1) as i32;
                let distance_to_obstacle_x2 = navmesh_x1.abs_diff(obstacle_x2) as i32;
                let distance_to_obstacle_z1 = navmesh_z1.abs_diff(obstacle_z1) as i32;
                let distance_to_obstacle_z2 = navmesh_z1.abs_diff(obstacle_z2) as i32;

                let rect_x = (distance_to_obstacle_x1 + distance_to_obstacle_x2) / 2;
                let rect_y = (distance_to_obstacle_z1 + distance_to_obstacle_z2) / 2;

                let rect_w = distance_to_obstacle_x1.abs_diff(distance_to_obstacle_x2) as i32;
                let rect_h = distance_to_obstacle_z1.abs_diff(distance_to_obstacle_z2) as i32;

                let rect = Rect::new(rect_x, rect_y, rect_w, rect_h);
                grid.set_rectangle(&rect, true);
            }

            grid.allow_diagonal_move = true;
            grid.heuristic_factor = 1.3;
            grid.generate_components();
            self.navmesh_grids.insert(*navmesh_id, grid);
        }
    }

    pub fn find_path(&self, start_world: Vec2, finish_world: Vec2) -> Option<Vec<Vec2>> {
        let start_x = start_world.x.round() as i32;
        let start_z = start_world.y.round() as i32;

        let finish_x = finish_world.x.round() as i32;
        let finish_z = finish_world.y.round() as i32;

        for (navmesh_id, dim) in self.navmesh_dimensions.iter() {
            let navmesh_position_x = dim.position[0];
            let navmesh_position_z = dim.position[1];
            let area_size_x = dim.area_size[0];
            let area_size_z = dim.area_size[1];

            let navmesh_x1 = navmesh_position_x - area_size_x as i32 / 2;
            let navmesh_x2 = navmesh_position_x + area_size_x as i32 / 2;
            let navmesh_z1 = navmesh_position_z - area_size_z as i32 / 2;
            let navmesh_z2 = navmesh_position_z + area_size_z as i32 / 2;

            if (start_x >= navmesh_x1 && start_x <= navmesh_x2)
                && (start_z >= navmesh_z1 && start_z <= navmesh_z2)
                && (finish_x >= navmesh_x1 && finish_x <= navmesh_x2)
                && (finish_z >= navmesh_z1 && finish_z <= navmesh_z2)
            {
                match self.navmesh_grids.get(navmesh_id) {
                    Some(grid) => {
                        let mut count = 0;
                        for x in 0..grid.width() {
                            for y in 0..grid.height() {
                                let val = grid.get(x, y);
                                if val == true {
                                    count += 1;
                                }
                            }
                        }
                        //dbg!(count);

                        let grid_start_x = navmesh_x1.abs_diff(start_x);
                        let grid_start_z = navmesh_z1.abs_diff(start_z);
                        let grid_finish_x = navmesh_x1.abs_diff(finish_x);
                        let grid_finish_z = navmesh_z1.abs_diff(finish_z);

                        let grid_start = Point::new(grid_start_x as i32, grid_start_z as i32);
                        let grid_finish = Point::new(grid_finish_x as i32, grid_finish_z as i32);

                        match grid.get_path_single_goal(grid_start, grid_finish, false) {
                            Some(path) => {
                                let mut converted_path = path.iter().map(|point| {
                                    let point_x = point.x as f32 - navmesh_x2 as f32;
                                    let point_z = point.y as f32 - navmesh_z2 as f32;
                                    Vec2::new(point_x, point_z)
                                }).collect::<Vec<Vec2>>();
                                converted_path.push(finish_world);

                                return Some(converted_path);
                            },
                            None => {
                                return None;
                            }
                        }
                    }
                    None => return None,
                }
            }
        }
        None
    }
}

