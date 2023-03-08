
pub mod cell_handler {
    use glam::{IVec2, Vec2};
    use fastrand::shuffle;
    use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

    use crate::{Matrix, MaterialType, rand_multiplier, Material, Assets, Cell, Rng, gen_range, RNG, WALL, EMPTY};

    /// Function which gets called for all the cells.
    /// 
    /// Calls the respective methods depending on the cell material
    pub fn handle_cell(matrix: &mut Matrix, mut cell: Cell) {
        let hp = cell.hp;
        
        // This cell died; delete it
        if hp == 0 {
            matrix.set_cell_material(cell.pos, Material::Empty, false);
            return;
        };

        if cell.was_on_fire_last_frame && cell.is_on_fire {
            cell = fire_step(matrix, cell);
        };

        cell = match cell.material.get_type() {
            MaterialType::MovableSolid => movable_solid_step(matrix, cell),
            MaterialType::Liquid => liquid_step(matrix, cell),
            MaterialType::Gas => gas_step(matrix, cell),
            _ => cell,
        };
        if cell.is_free_falling {
            cell.color = pixels::wgpu::Color::GREEN;
        } else {
            cell.color = pixels::wgpu::Color::RED;
        };
        matrix.update_cell(cell);
    }


    /// Handles the cell logic for movable solids like sand (first down then diagonally down)
    fn movable_solid_step(matrix: &mut Matrix, cell: Cell) -> Cell {
        let bottom = IVec2::new(0, 1);
        let is_movable_solid = cell.material.get_type() == MaterialType::MovableSolid;
        
        let cellpos = cell.pos;
        let mut cell2 = try_move(matrix, cell, cellpos + bottom, false);
        //println!("Pos: {}  Try move down: {}", cellpos, cell2.pos);
        
        if cellpos != cell2.pos {
            cell2.is_free_falling = true;
            for mut neighbour in matrix.get_neighbor_cells(cell2.pos, 1) {
                if neighbour.attempt_free_fall() {
                    matrix.update_cell(neighbour);
                };
            };
            return cell2;
        };

        let rand_bool = gen_range(0.0, 1.0) < 0.5;
        if !cell2.is_free_falling {
            cell2.velocity = Vec2::ZERO;
            return cell2;
        };
        
        let mut fac = 1.0;
        if cell2.velocity.x < 0.0 || rand_bool {
            fac = -1.0;
        };
        
        // TODO: Maybe split up this function even more so i dont have to add liquid logic in here
        if is_movable_solid {
            cell2.velocity.x = (cell2.velocity.y / 4.0) * fac;
            cell2.velocity.y *= -0.1;
        } else {
            cell2.velocity.y = 0.0;
        };
        
        //let x_vel_check = cell2.velocity.x.round().abs().max(1.0) as i32;
        let disp = cell2.material.get_dispersion() as i32;
        let mut cellpos = cell2.pos;
        matrix.set_chunk_active(cellpos);
        let bottom_left = cellpos + IVec2::new(-disp, 1);
        let bottom_right = cellpos + IVec2::new(disp, 1);
        let mut first = bottom_left;
        let mut second = bottom_right;
        if rand_bool {
            first = bottom_right;
            second = bottom_left
        };
        //println!("{}", first == bottom_right);
        cellpos = cell2.pos;
        let cell = try_move(matrix, cell2, first, true);
        if cellpos != cell.pos {
            return cell;
        };
        try_move(matrix, cell, second, true)
    }


    /// Handles the cell logic for gases (upside down movable solids)
    fn gas_step(matrix: &mut Matrix, cell: Cell) -> Cell {
        let up = cell.pos + IVec2::new(0, -1);
        let mut cellpos = cell.pos;
        let cell2 = try_move(matrix, cell, up, false);
        if cell2.pos != cellpos {
            return cell2;
        };

        let disp = cell2.material.get_dispersion() as i32;
        let up_left = cell2.pos + IVec2::new(-disp, -1);
        let up_right = cell2.pos + IVec2::new(disp, -1);
        let mut first = up_left;
        let mut second = up_right;
        if gen_range(0.0, 1.0) > 0.5 {
            first = up_right;
            second = up_left
        };
        cellpos = cell2.pos;
        let cell = try_move(matrix, cell2, first, true);
        if cellpos != cell.pos {
            return cell;
        };
        try_move(matrix, cell, second, true)
    }

    /// Handles the cell logic for liquids (first movable solid step the horizontal)
    fn liquid_step(matrix: &mut Matrix, cell: Cell) -> Cell {
        let mut cellpos = cell.pos;
        let cell2 = movable_solid_step(matrix, cell);
        if cell2.pos != cellpos {
            return cell2;
        };
        
        cellpos = cell2.pos;
        let cellmat = cell2.material;
        let disp = cellmat.get_dispersion() as i32;
        let dir = rand_multiplier();
        
        let horizontal_movement = cellpos + IVec2::new(disp * dir, cell2.velocity.y.round() as i32);
        let cell = try_move(matrix, cell2, horizontal_movement, false);
        if cell.pos != cellpos {
            matrix.set_chunk_active(cell.pos);
        };
        cell
    }


    /// Tries to move the cell to the specified position. Stops when it encounters an obstacle
    fn try_move(matrix: &mut Matrix, mut cell: Cell, to_pos: IVec2, diagonal: bool) -> Cell {
        let width = matrix.width as i32;
        let height = matrix.height as i32;
        
        let (cellpos, cellmat) = (cell.pos, cell.material);
        if cellpos == to_pos {
            return cell;
        };
        
        let x0 = cellpos.x.clamp(0, width);
        let y0 = cellpos.y.clamp(0, height);
        let mut num_steps = 0;
        let start = IVec2::new(x0, y0);
        let mut last_possible_cell = start;

        let iter = line_drawing::WalkGrid::new((x0, y0), (to_pos.x.clamp(0, width - 1), to_pos.y.clamp(0, height - 1)));

        for (x, y) in iter {
            let cur_pos = IVec2::new(x, y);
            if cur_pos == cellpos {
                continue;
            };
            let target_cell = matrix.get_cell_by_pos(cur_pos);
            if target_cell != EMPTY {
                if num_steps > 1 {
                    break;
                };
                if target_cell.material == cellmat && !diagonal {
                    break;
                } else if target_cell.material.get_density() < cellmat.get_density() {
                    last_possible_cell = cur_pos;
                };
                if last_possible_cell == start && !diagonal {
                    break;
                };
            } else {
                if cell != WALL {
                    last_possible_cell = cur_pos;
                };
            };
            num_steps += 1;
        };

        if last_possible_cell != start {
            cell = matrix.set_cell_by_pos(last_possible_cell, cellpos, true);
            return cell;
        };

        cell
    }


    /// Handles fire logic
    fn fire_step(matrix: &mut Matrix, mut cell: Cell) -> Cell {
        cell.hp = cell.hp.saturating_sub(1);

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
        let mut extinguisher = (None, 1.0);
        let mut i = 0;
        let num_neigh = neighbors.len();
        for index in indices {
            if index >= num_neigh {
                continue;
            };
            let n_cell = &neighbors[index];
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
                for n_cell_neigh in n_cell_neighbors {
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
            let mut ext = matrix.get_cell_by_pos(extinguisher.0.unwrap());
            if ext != EMPTY {
                ext.hp = (ext.hp as f32 * extinguisher.1).round() as u64;
                if ext.material.get_type() == MaterialType::Liquid {
                    matrix.set_cell_material(cellpos + IVec2::new(0, -1), Material::Smoke, false);
                };
                matrix.update_cell(ext);
            };
            cell.is_on_fire = false;
            return cell;
        };

        // If this fire cell did find another cell to spread to
        for spread_cell_pos in spread {
            let mut spread_cell = matrix.get_cell_by_pos(spread_cell_pos);
            if spread_cell != EMPTY || spread_cell != WALL {
                spread_cell.is_on_fire = true;
                matrix.update_cell(spread_cell);
                matrix.set_chunk_active(spread_cell_pos);
            };
        };

        cell
    }
}