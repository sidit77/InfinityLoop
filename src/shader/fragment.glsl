#version 300 es

precision highp float;

out vec4 finalColor;

uniform vec4 color;

void main() {
    finalColor = color;
}