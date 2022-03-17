#version 300 es

precision highp float;

out vec4 finalColor;

in vec2 tex_coords;

uniform sampler2D tex;

vec4 foreground = vec4(1,1,1,1);
vec4 background = vec4(0,0,0,1);

void main() {
    float d = texture(tex, tex_coords).r;

    //finalColor = (d > 0.5) ? foreground : background;
    finalColor = mix(background, foreground, 1.0 - clamp(20.0 * abs(d - 0.50), 0.0, 1.0));
    //finalColor = vec4(d, d, d, 1);
}