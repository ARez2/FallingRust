

pub mod cell_handler {
    use glam::{IVec2, Vec2};

    use crate::{Matrix, MaterialType, rand_multiplier, Material, Cell, Assets};

    /// Function which gets called for all the cells.
    /// 
    /// Calls the respective methods depending on the cell material
    pub fn handle_cell(matrix: &mut Matrix, cell_index: usize, assets: &mut Assets) {
        let cell = matrix.get_cell_by_cellindex_mut(cell_index);
        if cell.is_none() {
            return;
        };
        let cell = cell.unwrap();
        let cellpos = cell.pos;
        let hp = cell.hp;
        let on_fire = cell.is_on_fire;
        let was_on_fire = cell.was_on_fire_last_frame;
        let cellmat = cell.material;
        
        // This cell died; delete it
        if hp == 0 {
            matrix.set_cell_material(cellpos, Material::Empty, false, assets);
            return;
        };

        if was_on_fire && on_fire {
            fire_step(matrix, cell_index, assets);
        };

        let did_move = match cellmat.get_type() {
            MaterialType::MovableSolid => movable_solid_step(matrix, cell_index),
            MaterialType::Liquid => liquid_step(matrix, cell_index),
            MaterialType::Gas => gas_step(matrix, cell_index),
            _ => false,
        };
    }

    /// Handles the cell logic for movable solids like sand (first down then diagonally down)
    fn movable_solid_step(matrix: &mut Matrix, cell_index: usize) -> bool {
        let mut bottom = IVec2::new(0, 1);
        let mut is_movable_solid = true;
        {
            let cell = matrix.get_cell_by_cellindex_mut(cell_index);
            if cell.is_none() {
                return false;
            };
            let (freefall, cellpos) = {
                let cell = cell.unwrap();
                bottom = cell.pos + IVec2::new(0, cell.velocity.y.round() as i32);
                is_movable_solid = cell.material.get_type() == MaterialType::MovableSolid;
                (cell.is_free_falling, cell.pos)
            };
            
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
        }
        
        if try_move(matrix, cell_index, bottom, false) {
            let cell = matrix.get_cell_by_cellindex_mut(cell_index).unwrap();
            cell.is_free_falling = true;
            return true;
        };

        let cell = matrix.get_cell_by_cellindex_mut(cell_index).unwrap();
        if !cell.is_free_falling {
            cell.velocity = Vec2::ZERO;
            return false;
        };
        let mut fac = 1.0;
        if cell.velocity.x > 0.0 {
            fac = -1.0;
        } else if cell.velocity.x == 0.0 {
            if rand::random() {
                fac = -1.0;
            };
        };
        // TODO: Maybe split up this function even more so i dont have to add liquid logic in here
        if is_movable_solid {
            cell.velocity.x = (cell.velocity.y / 4.0) * fac;
            cell.velocity.y *= -0.1;
        } else {
            cell.velocity.y = 0.0;
        };
        
        let x_vel_check = cell.velocity.x.round().abs().max(1.0) as i32;
        let disp = cell.material.get_dispersion() as i32;
        let cellpos = cell.pos;
        let bottom_left = cellpos + IVec2::new(-1 * disp * x_vel_check, 1);
        let bottom_right = cellpos + IVec2::new(1 * disp * x_vel_check, 1);
        let mut first = bottom_left;
        let mut second = bottom_right;
        if rand::random() {
            first = bottom_right;
            second = bottom_left
        };
        if try_move(matrix, cell_index, first, true) {
            return true;
        };
        if try_move(matrix, cell_index, second, true) {
            return true;
        };
        return false;
    }

    /// Handles the cell logic for gases (upside down movable solids)
    fn gas_step(matrix: &mut Matrix, cell_index: usize) -> bool {
        let (cellpos, cellmat) = {let c = matrix.get_cell_by_cellindex_mut(cell_index).unwrap(); (c.pos, c.material)};
        let up = cellpos + IVec2::new(0, -1);
        if try_move(matrix, cell_index, up, false) {
            return true;
        };

        let disp = cellmat.get_dispersion() as i32;
        let up_left = cellpos + IVec2::new(-1 * disp, -1);
        let up_right = cellpos + IVec2::new(1 * disp, -1);
        let mut first = up_left;
        let mut second = up_right;
        if rand::random() {
            first = up_right;
            second = up_left
        };
        if try_move(matrix, cell_index, first, true) {
            return true;
        };
        if try_move(matrix, cell_index, second, true) {
            return true;
        };
        return false;
    }

    /// Handles the cell logic for liquids (first movable solid step the horizontal)
    fn liquid_step(matrix: &mut Matrix, cell_index: usize) -> bool {
        if movable_solid_step(matrix, cell_index) {
            return true;
        };
        
        let cell = matrix.get_cell_by_cellindex_mut(cell_index).unwrap();
        let cellpos = cell.pos;
        let cellmat = cell.material;
        let disp = cellmat.get_dispersion() as i32;
        let dir = rand_multiplier();
        
        let horizontal_movement = cellpos + IVec2::new(disp * dir, 0);
        if try_move(matrix, cell_index, horizontal_movement, false) {
            return true;
        };
        return false;
    }

    /// Tries to move the cell to the specified position. Stops when it encounters an obstacle
    fn try_move(matrix: &mut Matrix, cell_index: usize, mut to_pos: IVec2, diagonal: bool) -> bool {
        //to_pos = matrix.clamp_pos(to_pos);
        let mut last_possible_cell: Option<_> = None;
        
        let width = matrix.width as i32;
        let height = matrix.height as i32;
        
        let (cellpos, cellmat) = {
            let cell = matrix.get_cell_by_cellindex_mut(cell_index);
            if cell.is_none() {
                return false;
            };
            let cell = cell.unwrap();
            (cell.pos, cell.material)
        };
        if cellpos == to_pos {
            return false;
        };
        
        let x0 = cellpos.x.max(0).min(width);
        let y0 = cellpos.y.max(0).min(height);
        let mut num_steps = 0;
        for (x, y) in line_drawing::WalkGrid::new((x0, y0), (to_pos.x, to_pos.y)) {
            let cur_pos = IVec2::new(x as i32, y as i32);
            if cur_pos == cellpos {
                continue;
            };
            let target_cell = matrix.get_cell(cur_pos);
            if let Some(tcell) = target_cell {
                if num_steps > 1 {
                    break;
                };

                let tcell_mat = tcell.material;
                if tcell_mat == cellmat && !diagonal {
                    break;
                } else if tcell_mat.get_density() < cellmat.get_density() {
                    last_possible_cell = Some(cur_pos);
                };
                if last_possible_cell.is_none() && !diagonal {
                    break;
                };
            } else {
                // Cell is empty
                if matrix.is_in_bounds(cur_pos) {
                    last_possible_cell = Some(cur_pos);
                };
            };

            num_steps += 1;
        };

        match last_possible_cell {
            None => (),
            Some(last_pos) => {
                if last_pos != IVec2::new(x0, y0) {
                    matrix.set_cell_by_pos(last_pos, cellpos, true);
                    return true;
                }
            },
        }

        return false;
    }

    /// Handles fire logic
    // TODO: Make water stop fire
    fn fire_step(matrix: &mut Matrix, cell_index: usize, assets: &mut Assets) -> bool {
        let cell = matrix.get_cell_by_cellindex_mut(cell_index).unwrap();
        cell.hp = cell.hp.saturating_sub(1);
        let cellpos = cell.pos;
        let mut spread = vec![];
        let neighbours = matrix.get_neighbor_cells(cellpos, 2);
        let mut extinguisher = (None, 1.0);
        for n in neighbours {
            if let Some(n_cell) = n {
                let ext = n_cell.material.extinguishes_fire();
                if ext.0 {
                    extinguisher = (Some(n_cell.pos), ext.1);
                    break;
                };

                let flammability = n_cell.material.get_flammability();
                if n_cell.is_on_fire {
                    continue;
                };
                let mut has_protection = false;
                let n_cell_neighbors = matrix.get_neighbor_cells(n_cell.pos, 7);
                for n_cell_neigh in n_cell_neighbors {
                    if let Some(n_cell_neigh) = n_cell_neigh {
                        if n_cell_neigh.material.extinguishes_fire().0 {
                            has_protection = true;
                        };
                    };
                };
                if !has_protection && rand::random::<f32>() < flammability {
                    spread.push(n_cell.pos);
                };
            };
        };
        if extinguisher.0.is_some() {
            let ext = matrix.get_cell_mut(extinguisher.0.unwrap());
            if let Some(ext) = ext {
                ext.hp = (ext.hp as f32 * extinguisher.1).round() as u64;
            };
            let cell = matrix.get_cell_by_cellindex_mut(cell_index).unwrap();
            cell.is_on_fire = false;
            matrix.set_cell_material(cellpos + IVec2::new(0, -1), Material::Smoke, false, assets);
            return false;
        };

        // If this fire cell did find another cell to spread to
        for spread_cell_pos in spread {
            if let Some(spread_cell) = matrix.get_cell_mut(spread_cell_pos) {
                spread_cell.is_on_fire = true;
            };
        };

        return false;
    }
}