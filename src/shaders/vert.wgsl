@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let x = 1 - f32(index & 1) * 4;
    let y = 1 - f32(index & 2) * 2;
    return vec4(x, y, 0., 1.);
}
