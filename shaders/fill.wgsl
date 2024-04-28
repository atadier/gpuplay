fn mainImage(fragColor: ptr<function, vec4<f32>>, fragCoord: vec2<f32>) {
    *fragColor = vec4(1., sin(iTime), 0., 1.);
}
