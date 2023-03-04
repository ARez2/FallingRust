//!ignore
//!include material.wgsl


fn mat_get_type(id: i32) -> i32 {
  if (id == MAT_EMPTY) {
    return MATTYPE_EMPTY;
  } else if (id == MAT_SAND || id == MAT_DIRT) {
    return MATTYPE_MOVABLESOLID;
  } else if (id == MAT_WATER) {
    return MATTYPE_LIQUID;
  } else if (id == MAT_ROCK || id == MAT_WOOD) {
    return MATTYPE_SOLID;
  } else if (id == MAT_SMOKE) {
    return MATTYPE_GAS;
  };
  return 0;
}




struct Cell {
  pos: vec2<i32>,
  prev_pos: vec2<f32>,
  velocity: vec2<f32>,
  hp: u32,
  base_color: vec4<f32>,
  color: vec4<f32>,
  material_id: i32,
  booleans: vec4<i32>,
}
// processed_this_frame: bool,
// is_free_falling: bool,
// is_on_fire: bool,
// was_on_fire_last_frame: bool,


struct CellStorage {
  cell_count : u32,
  cells : array<Cell, 4096>,
}


@group(0) @binding(0) var read_cells : texture_2d<u32>;
@group(0) @binding(1) var write_cells : texture_2d<u32>;
@group(0) @binding(2) var<storage, read> cells : CellStorage;


@compute @workgroup_size(8, 1, 1)
fn compute_main(@builtin(global_invocation_id) gid : vec3<u32>) {
    
}