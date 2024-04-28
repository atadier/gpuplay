@group(0) @binding(0) var<uniform> iTime: f32;
@group(0) @binding(0) var<uniform> iResolution: vec3<f32>;
@group(0) @binding(0) var<uniform> iMouse: vec4<f32>;
@group(0) @binding(0) var<uniform> iFrame: i32;
@group(0) @binding(0) var<uniform> iTimeDelta: f32;

@fragment
fn main(@builtin(position) coord: vec4<f32>) -> @location(0) vec4<f32> {
    var fragColor = vec4<f32>();
    mainImage(&fragColor, coord.xy);
    return fragColor;
}
