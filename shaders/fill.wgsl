fn mainImage(fragColor: ptr<function, vec4<f32>>, fragCoord: vec2<f32>) {
    *fragColor = vec4(sin(i.time), 1., 0., 1.);
}
