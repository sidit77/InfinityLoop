#version 300 es

precision highp float;

out vec4 finalColor;

in vec2 worldPos;

uniform vec4 color;

void main() {
    finalColor = color;
}