#version 450

precision highp float;
precision highp int;

layout(binding = 0) uniform Uniforms {
    float iTime;
    vec3 iResolution;
    vec4 iMouse;
    int iFrame;
    float iTimeDelta;
};

layout(location = 0) out vec4 fragColor;

void mainImage(out vec4 fragColor, in vec2 fragCoord);

void main()  {
    fragColor.w = 1.;
    mainImage(fragColor, gl_FragCoord.xy);
}
