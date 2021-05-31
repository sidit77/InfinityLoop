#version 300 es

layout(location = 0) in vec2 position;

uniform mat4 cam;
uniform mat4 obj;

void main() {
    gl_Position = cam * obj * vec4(position, 0, 1);
}