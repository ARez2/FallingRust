



pub mod CellHandler {
    use glam::IVec2;

    use crate::{Cell, Matrix, MaterialType, Material};

    pub fn handle_cell(matrix: &mut Matrix, cell_index: usize) {
        let cell = matrix.get_cell_by_cellindex_mut(cell_index);
        if cell.is_none() {
            return;
        };
        let cell = cell.unwrap();
        let material = cell.material;
        std::mem::drop(cell);

        let did_move = match material.get_type() {
            MaterialType::MovableSolid => movable_solid_step(matrix, cell_index, material),
            MaterialType::Liquid => liquid_step(matrix, cell_index, material),
            _ => false,
        };
    }


    fn movable_solid_step(matrix: &mut Matrix, cell_index: usize, material: Material) -> bool {
        let (mut freefall, cellpos, mut cellvel) = {
            let cell = matrix.get_cell_by_cellindex_mut(cell_index);
            if cell.is_none() {
                return false;
            };
            let cell = cell.unwrap();
            (cell.is_free_falling, cell.pos, cell.velocity)
        };
        //std::mem::drop(cell);


        if freefall {
            for y in -1..=1 {
                for x in -1..=1 {
                    let p = IVec2::new(x, y);
                    if p.abs() == IVec2::ONE && p == IVec2::ZERO {
                        continue;
                    };
                    let neighbour = matrix.get_cell_mut(cellpos + p);
                    if let Some(n_cell) = neighbour {
                        n_cell.attempt_free_fall();
                    };
                }
            }
        };

        let bottom = cellpos + IVec2::new(0, cellvel.y.round() as i32);
        if try_move(matrix, cell_index, material, bottom, false) {
            freefall = true;
            return true;
        };
        if !freefall {
            {
            let cell = matrix.get_cell_by_cellindex_mut(cell_index);
            if cell.is_none() {
                return false;
            };
            let cell = cell.unwrap();
            cell.is_free_falling = freefall;
            }
            return false;
        };
        let mut fac = 1.0;
        if cellvel.x > 0.0 {
            fac = -1.0;
        } else if cellvel.x == 0.0 {
            if rand::random() {
                fac = -1.0;
            };
        };
        cellvel.x = (cellvel.y / 2.0) * fac;
        cellvel.y *= -0.1;

        {
            let cell = matrix.get_cell_by_cellindex_mut(cell_index);
            if cell.is_none() {
                return false;
            };
            let cell = cell.unwrap();
            cell.velocity = cellvel;
            cell.is_free_falling = freefall;
        }
        
        let x_vel_check = cellvel.x.round().abs().max(1.0) as i32;
        let disp = material.get_dispersion() as i32;
        let bottom_left = cellpos + IVec2::new(-1 * disp * x_vel_check, 1);
        let bottom_right = cellpos + IVec2::new(1 * disp * x_vel_check, 1);
        let mut first = bottom_left;
        let mut second = bottom_right;
        if rand::random() {
            first = bottom_right;
            second = bottom_left
        };
        if try_move(matrix, cell_index, material, first, true) {
            return true;
        };
        if try_move(matrix, cell_index, material, second, true) {
            return true;
        };
        return false;
    }


    fn liquid_step(matrix: &mut Matrix, cell_index: usize, material: Material) -> bool {
        if movable_solid_step(matrix, cell_index, material) {
            return true;
        };
        
        let dir_multi = match rand::random::<bool>() {
            true => 1,
            false => -1,
        };
        
        if try_move(matrix, cell_index, material, IVec2::new(material.get_dispersion() as i32 * dir_multi, 0), false) {return true;};
        return false;
    }



    fn try_move(matrix: &mut Matrix, cell_index: usize, material: Material, to_pos: IVec2, diagonal: bool) -> bool {
        let cellpos = matrix.get_cellpos_by_cellindex(cell_index);
        if cellpos.is_none() {
            return false;
        };
        let cellpos = cellpos.unwrap();

        let mut swapped = false;
        let mut last_possible_cell: Option<_> = None;
        
        let x0 = cellpos.x.max(0).min(matrix.width as i32);
        let y0 = cellpos.y.max(0).min(matrix.height as i32);
        for (x, y) in line_drawing::WalkGrid::new((x0, y0), (to_pos.x, to_pos.y)) {
            let cur_pos = IVec2::new(x as i32, y as i32);
            if cur_pos == cellpos {
                continue;
            };
            let target_material = matrix.get_material_at_pos(cur_pos);
            if let Some(mat) = target_material {
                if mat.get_density() < material.get_density() {
                    last_possible_cell = Some(cur_pos);
                };
                if last_possible_cell.is_none() && !diagonal {
                    break;
                };
            } else {
                break;
            };
        };

        match last_possible_cell {
            None => (),
            Some(last_pos) => {
                if last_pos != IVec2::new(x0, y0) {
                    matrix.set_cell_by_pos(last_pos, cellpos, true);
                    swapped = true;
                }
            },
        }

        return swapped;
    }
}