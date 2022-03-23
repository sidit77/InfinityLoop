#version 300 es

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 texcoords;
layout(location = 2) in mat3 model;
layout(location = 5) in uint texId;

uniform mat3 camera;

out vec3 tex_coords;

void main() {
    vec3 pos = camera * model * vec3(position, 1);
    gl_Position = vec4(pos.xy / pos.z, 0, 1);
    tex_coords = vec3(texcoords, float(texId));
}