//!ignore

let MAT_EMPTY = 0;
let MAT_SAND = 1;
let MAT_DIRT = 2;
let MAT_WATER = 3;
let MAT_ROCK = 4;
let MAT_SMOKE = 5;
let MAT_WOOD = 6;

let MATTYPE_EMPTY = 0;
let MATTYPE_SOLID = 1;
let MATTYPE_MOVABLESOLID = 2;
let MATTYPE_LIQUID = 3;
let MATTYPE_GAS = 4;
//!ignore

let tau: f32 = 6.283185307179586476925286766559;
let bias: f32 = 0.2376; // Offset the circular time input so it is never 0

// Random functions based on https://thebookofshaders.com/10/
let random_scale: f32 = 43758.5453123;
let random_x: f32 = 12.9898;
let random_y: f32 = 78.233;

fn random(x: f32) -> f32 {
    return fract(sin(x) * random_scale);
}

fn random_vec2(st: vec2<f32>) -> f32 {
    return random(dot(st, vec2<f32>(random_x, random_y)));
}
//!include random.wgsl material.wgsl


struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = fma(position, vec2<f32>(0.5, -0.5), vec2<f32>(0.5, 0.5));
    out.position = vec4<f32>(position, 0.0, 1.0);
    return out;
};


@group(0) @binding(0) var cell_data: texture_2d<i32>;
@group(0) @binding(1) var cell_data_sampler: sampler;
@group(0) @binding(2) var output: texture_2d<f32>;


@fragment
fn fs_main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {
    var cell : vec4<i32> = textureSample(cell_data, cell_data_sampler, tex_coord);
    var celltype = cell & 0xff;
    if (celltype == MAT_SAND) {
        return vec4<f32>(0.0, 1.0, 1.0, 1.0);
    };
    return vec4<f32>(vec3<f32>(0.0), 1.0);
}