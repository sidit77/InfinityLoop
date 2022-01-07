#version 100

attribute vec2 pos;

uniform mat4 camera;
uniform mat4 model;

varying highp vec2 worldPos;

void main() {
    vec4 wp = model * vec4(pos, 0, 1);
    worldPos = wp.xy / wp.w;
    gl_Position = camera * wp;
}