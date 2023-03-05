
pub mod cell_handler {
    use glam::{IVec2, Vec2};
    use fastrand::shuffle;

    use crate::{Matrix, MaterialType, rand_multiplier, Material, Assets, Cell, Rng, gen_range, RNG};

    /// Function which gets called for all the cells.
    /// 
    /// Calls the respective methods depending on the cell material
    pub fn handle_cell(matrix: &mut Matrix, cell_index: usize, chunk_index: usize) {
        // if !matrix.chunks[chunk_index].should_step {
        //     return;
        // };

        let cell = matrix.get_cell_by_cellindex_mut(cell_index);
        if cell.is_none() {
            return;
        };
        let (cellpos, hp, on_fire, was_on_fire, cellmat, hp_changed, cellvelocity) = {
            let cell = cell.unwrap();
            let hp = cell.hp;
            let cellvel = cell.velocity;
            cell.update();
            cell.processed_this_frame = true;
            (cell.pos, cell.hp, cell.is_on_fire, cell.was_on_fire_last_frame, cell.material, hp != cell.hp, cellvel)
        };
        // if on_fire || was_on_fire || hp_changed || cellvelocity.length() > 0.0 {
        //     matrix.set_chunk_active(cellpos);
        // };
        
        // This cell died; delete it
        if hp == 0 {
            matrix.set_cell_material(cellpos, Material::Empty, false);
            return;
        };

        if was_on_fire && on_fire {
            fire_step(matrix, cell_index);
        };

        let _ = match cellmat.get_type() {
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
            let (freefall, cellp) = {
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
                        let neighbour = matrix.get_cell_mut(cellp + p);
                        if let Some(n_cell) = neighbour {
                            n_cell.attempt_free_fall();
                        };
                    }
                }
            };
            
        };
        
        if try_move(matrix, cell_index, bottom, false) {
            let cell = matrix.get_cell_by_cellindex_mut(cell_index).unwrap();
            cell.is_free_falling = true;
            return true;
        };

        let rand_bool = gen_range(0.0, 1.0) > 0.5;
        let cell = matrix.get_cell_by_cellindex_mut(cell_index).unwrap();
        if !cell.is_free_falling {
            cell.velocity = Vec2::ZERO;
            return false;
        };
        
        let mut fac = 1.0;
        if cell.velocity.x < 0.0 || rand_bool {
            fac = -1.0;
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
        matrix.set_chunk_active(cellpos);
        let bottom_left = cellpos + IVec2::new(-disp * x_vel_check, 1);
        let bottom_right = cellpos + IVec2::new(disp * x_vel_check, 1);
        let mut first = bottom_left;
        let mut second = bottom_right;
        if rand_bool {
            first = bottom_right;
            second = bottom_left
        };
        if try_move(matrix, cell_index, first, true) {
            return true;
        };
        if try_move(matrix, cell_index, second, true) {
            return true;
        };
        false
    }

    /// Handles the cell logic for gases (upside down movable solids)
    fn gas_step(matrix: &mut Matrix, cell_index: usize) -> bool {
        let (cellpos, cellmat) = {let c = matrix.get_cell_by_cellindex_mut(cell_index).unwrap(); (c.pos, c.material)};
        let up = cellpos + IVec2::new(0, -1);
        if try_move(matrix, cell_index, up, false) {
            return true;
        };

        let disp = cellmat.get_dispersion() as i32;
        let up_left = cellpos + IVec2::new(-disp, -1);
        let up_right = cellpos + IVec2::new(disp, -1);
        let mut first = up_left;
        let mut second = up_right;
        if gen_range(0.0, 1.0) > 0.5 {
            first = up_right;
            second = up_left
        };
        if try_move(matrix, cell_index, first, true) {
            return true;
        };
        if try_move(matrix, cell_index, second, true) {
            return true;
        };
        false
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
        
        let horizontal_movement = cellpos + IVec2::new(disp * dir, cell.velocity.y.round() as i32);
        if try_move(matrix, cell_index, horizontal_movement, false) {
            return true;
        };
        false
    }

    /// Tries to move the cell to the specified position. Stops when it encounters an obstacle
    fn try_move(matrix: &mut Matrix, cell_index: usize, to_pos: IVec2, diagonal: bool) -> bool {
        let mut last_possible_cell: Option<_> = None;
        
        let width = matrix.width as i32;
        let height = matrix.height as i32;
        
        let (cellpos, cellmat) = matrix.get_cell_by_cellindex_mut(cell_index)
            .and_then(|cell| Some((cell.pos, cell.material)))
            .unwrap_or((IVec2::new(0, 0), Material::Empty));
        if cellpos == to_pos {
            return false;
        };
        
        let x0 = cellpos.x.clamp(0, width);
        let y0 = cellpos.y.clamp(0, height);
        let mut num_steps = 0;

        let iter = line_drawing::WalkGrid::new((x0, y0), (to_pos.x.clamp(0, width), to_pos.y.clamp(0, height)));

        for (x, y) in iter {
            let cur_pos = IVec2::new(x, y);
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

        if let Some(last_pos) = last_possible_cell {
            if last_pos != IVec2::new(x0, y0) {
                matrix.set_cell_by_pos(last_pos, cellpos, true);
                return true;
            }
        }

        false
    }

    /// Handles fire logic
    fn fire_step(matrix: &mut Matrix, cell_index: usize) -> bool {
        let cell = matrix.get_cell_by_cellindex_mut(cell_index).unwrap();
        cell.hp = cell.hp.saturating_sub(1);
        if cell.hp == 0 {
            return false;
        };

        let cellpos = cell.pos;
        let mut spread = vec![];
        let radius = 2;
        let num_neighbors = 8*radius;
        let mut rand_probs: Vec<f32> = vec![0.0; num_neighbors];
        for i in 0..num_neighbors {
            rand_probs[i] = gen_range(0.0, 1.0);
        }
        let mut indices: Vec<usize> = (0..num_neighbors).collect();
        unsafe {&mut *RNG}.shuffle(&mut indices);

        let neighbors = matrix.get_neighbor_cells(cellpos, radius as i32);
        let neighbor_cells: Vec<&Cell> = neighbors.into_iter().flatten().collect();
        let mut extinguisher = (None, 1.0);
        let mut i = 0;
        let num_neigh = neighbor_cells.len();
        for index in indices {
            if index >= num_neigh {
                continue;
            };
            let n_cell = neighbor_cells[index];
            let ext = n_cell.material.extinguishes_fire();
            if ext.0 {
                extinguisher = (Some(n_cell.pos), ext.1);
                break;
            };
    
            let flammability = n_cell.material.get_flammability();
            if n_cell.is_on_fire {
                continue;
            };
            if rand_probs[i] < flammability {
                let mut has_protection = false;
                let n_cell_neighbors = matrix.get_neighbor_cells(n_cell.pos, 5);
                for n_cell_neigh in n_cell_neighbors.into_iter().flatten() {
                    if n_cell_neigh.material.protects_from_fire() {
                        has_protection = true;
                        break;
                    };
                };
                if !has_protection {
                    spread.push(n_cell.pos);
                    i += 1;
                };
            }
        };
       
        if extinguisher.0.is_some() {
            let ext = matrix.get_cell_mut(extinguisher.0.unwrap());
            if let Some(ext) = ext {
                ext.hp = (ext.hp as f32 * extinguisher.1).round() as u64;
                if ext.material.get_type() == MaterialType::Liquid {
                    matrix.set_cell_material(cellpos + IVec2::new(0, -1), Material::Smoke, false);
                };
            };
            let cell = matrix.get_cell_by_cellindex_mut(cell_index).unwrap();
            cell.is_on_fire = false;
            return false;
        };

        // If this fire cell did find another cell to spread to
        for spread_cell_pos in spread {
            if let Some(spread_cell) = matrix.get_cell_mut(spread_cell_pos) {
                spread_cell.is_on_fire = true;
                matrix.set_chunk_active(spread_cell_pos);
            };
        };

        false
    }
}