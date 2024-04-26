#version 450

layout(binding = 0) uniform float iTime;
layout(location = 0) out vec4 fragColor;

void main() {
    fragColor = vec4(sin(iTime), 1., 0., 1.);
}
