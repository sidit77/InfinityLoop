#version 300 es

precision highp float;

out vec4 finalColor;

in vec2 tex_coords;

uniform sampler2D tex;

void main() {
    finalColor = texture(tex, tex_coords);
}