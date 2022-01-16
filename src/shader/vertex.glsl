#version 300 es

layout(location = 0) in vec2 position;

uniform mat4 camera;
uniform mat4 model;

out vec2 worldPos;

void main() {
    vec4 wp = model * vec4(position, 0, 1);
    worldPos = wp.xy / wp.w;
    gl_Position = camera * wp;
}