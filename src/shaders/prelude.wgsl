struct Uniforms {
    time: f32,
    resolution: vec3<f32>,
    mouse: vec4<f32>,
    frame: i32,
    delta_time: f32,
};

@group(0) @binding(0) var<uniform> i: Uniforms;

@fragment
fn main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    var fragColor = vec4<f32>();
    mainImage(&fragColor, coord.xy);
    return fragColor;
}
