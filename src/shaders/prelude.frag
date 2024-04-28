#version 450

layout(binding = 0) uniform float iTime;
layout(binding = 0) uniform vec3 iResolution;
layout(binding = 0) uniform vec4 iMouse;
layout(binding = 0) uniform int iFrame;
layout(binding = 0) uniform float iTimeDelta;

layout(location = 0) out vec4 fragColor;

void mainImage(out vec4 fragColor, in vec2 fragCoord);

void main()  {
    fragColor.w = 1.;
    mainImage(fragColor, gl_FragCoord.xy);
}
