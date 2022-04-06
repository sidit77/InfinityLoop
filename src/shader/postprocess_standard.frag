#version 300 es

precision highp float;

out vec4 finalColor;

in vec2 tex_coords;
in vec2 world_pos;

uniform sampler2D tex;
uniform bool completed;

vec4 foreground  = vec4(0.847,0.871,0.914,1.0);
vec4 background1 = vec4(0.180,0.204,0.251,1.0);
vec4 background2 = vec4(0.231,0.259,0.322,1.0);

void main() {
    float sd = (texture(tex, tex_coords).r - 0.5) * 10.0;

    float final_opacity = abs(sd) - 0.3;

    finalColor = mix(completed ? background2 : background1, foreground, 1.0 - clamp(final_opacity, 0.0, 1.0));

}