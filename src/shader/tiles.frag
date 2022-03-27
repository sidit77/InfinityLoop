#version 300 es

precision mediump float;
precision mediump sampler2DArray;

out float finalColor;

in vec3 tex_coords;

uniform sampler2DArray tex;
uniform float range;

float screenPxRange() {
    vec2 unitRange = vec2(range);
    vec2 screenTexSize = vec2(1.0)/ vec2(length(dFdx(tex_coords.xy)), length(dFdy(tex_coords.xy)));
    return max(0.5*dot(unitRange, screenTexSize), 1.0);
}

void main() {
    float sd = texture(tex, tex_coords).r - 0.5;
    float screenPxDistance = screenPxRange() * (1.0 / 10.0) * sd;
    finalColor = screenPxDistance + 0.5;
}
