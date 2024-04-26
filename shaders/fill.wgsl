@group(0) @binding(0) var<uniform> iTime: f32;

@fragment
fn main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4(1., sin(iTime), 0., 1.);
}