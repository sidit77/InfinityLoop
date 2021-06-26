#version 300 es

precision highp float;

out vec4 finalColor;

in vec2 worldPos;

uniform vec4 color;
uniform vec2 clickPos;
uniform float radius;

void main() {
    if(length(worldPos - clickPos) > radius) {
        finalColor = color;
    }else {
        finalColor = vec4(vec3(1) - color.rgb, color.a);
    }

}