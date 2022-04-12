#version 300 es

precision highp float;
precision highp sampler2D;

in vec2 v_texcoords;

out vec4 finalColor;

uniform sampler2D tex;
uniform float screenPxRange;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    vec3 msd = texture(tex, v_texcoords).rgb;
    float sd = median(msd.r, msd.g, msd.b);
    float screenPxDistance = screenPxRange * (sd - 0.5);
    screenPxDistance = abs(screenPxDistance) - 0.6;
    float opacity = clamp(-screenPxDistance + 0.5, 0.0, 1.0);
    finalColor = vec4(1, 1, 1, opacity);
}
