#version 300 es

precision mediump float;
precision mediump sampler2DArray;

out float finalColor;

in vec3 tex_coords;

uniform sampler2DArray tex;
uniform vec4 color;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    //vec3 msd = texture(tex, tex_coords).rgb;
    //float sd = median(msd.r, msd.g, msd.b);
    //float screenPxDistance = 24.4 * (sd - 0.5);
    //float opacity = clamp(screenPxDistance + 0.5, 0.0, 1.0);
    //finalColor = color * vec4(1,1,1,opacity);
    finalColor = texture(tex, tex_coords).r;
}
