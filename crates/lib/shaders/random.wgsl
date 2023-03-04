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