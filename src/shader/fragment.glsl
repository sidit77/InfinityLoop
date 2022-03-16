#version 300 es

precision highp float;

out vec4 finalColor;

in vec2 tex_coords;

void main() {
    finalColor = vec4(tex_coords, 1, 1);
}