use glam::Vec2;
use grid::Grid;
use once_cell::sync::Lazy;

// Map that is only for 0+ values
pub static mut MAP: Lazy<Grid<bool>> = Lazy::new(|| Grid::new(0, 0)); 
pub static mut MIN_WORLD_X: i32 = 0;
pub static mut MIN_WORLD_Z: i32 = 0;
pub static mut ROW_LEN: usize = 0; // X
pub static mut COL_LEN: usize = 0; // Z

pub fn world_x_to_map_x(world_x: f32) -> i32 {
    if world_x >= 0.0 {
        let min_map_x_abs = unsafe { (MIN_WORLD_X.abs_diff(0) as f32 / 2 as f32).round() as i32 };
        (world_x / 4.0).round() as i32 + min_map_x_abs as i32
    } else {
        let min_map_x = unsafe { MIN_WORLD_X };
        dbg!(min_map_x.abs_diff((world_x / 2.0) as i32));
        min_map_x.abs_diff((world_x / 2.0) as i32) as i32
    }
}

pub fn world_z_to_map_z(world_z: f32) -> i32 {
    (world_z / 4.0).round() as i32
}



pub fn get_map_val_by_world_coords(world_xz: Vec2) -> bool {
    todo!()
}

pub fn set_map_val_by_map_coords(map_xz: Vec2, val: bool) {
    let map_x = map_xz.x as i32;
    let map_z = map_xz.y as i32;

    let map = unsafe { &mut MAP };
    let row_len = unsafe { &mut ROW_LEN };
    let col_len = unsafe { &mut COL_LEN };

    if map_x > *row_len as i32 {
        let mut col = Vec::new();
        col.resize(*col_len, false);
        map.push_col(col);

        *row_len = map_x as usize;
    }


    // TODO
}
