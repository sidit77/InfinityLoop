#version 300 es

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 texcoords;

uniform mat4 camera;
uniform mat4 model;

out vec2 tex_coords;

void main() {
    gl_Position = camera * model * vec4(position, 0, 1);
    tex_coords = texcoords;
}