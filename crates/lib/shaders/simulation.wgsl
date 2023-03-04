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