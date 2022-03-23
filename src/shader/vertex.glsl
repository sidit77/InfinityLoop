#version 300 es

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 texcoords;

uniform mat3 camera;
uniform mat3 model;

out vec2 tex_coords;

void main() {
    vec3 pos = camera * model * vec3(position, 1);
    gl_Position = vec4(pos.xy / pos.z, 0, 1);
    tex_coords = texcoords;
}