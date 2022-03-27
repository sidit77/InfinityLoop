#version 300 es

precision mediump float;

out vec4 finalColor;

in vec2 tex_coords;

uniform sampler2D tex;

vec4 foreground = vec4(1,1,1,1);
vec4 background = vec4(0,0,0,1);

void main() {
    float sd = (texture(tex, tex_coords).r - 0.5) * 10.0;

    finalColor = mix(background, foreground, 1.0 - clamp(abs(sd) - 0.25, 0.0, 1.0));
}