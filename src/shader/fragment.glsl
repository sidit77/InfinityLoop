#version 100

precision highp float;

varying highp vec2 worldPos;

uniform vec4 color;
uniform vec2 clickPos;
uniform float radius;

void main() {
    if(length(worldPos - clickPos) > radius) {
        gl_FragColor = color;
    }else {
        gl_FragColor = vec4(vec3(1) - color.rgb, color.a);
    }

}