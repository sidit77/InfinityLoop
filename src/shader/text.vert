#version 300 es

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 texcoord;

out vec2 v_texcoords;

uniform mat4 matrix;

void main() {
    gl_Position = matrix * vec4(position, 0.0, 1.0);
    v_texcoords = texcoord;
}
