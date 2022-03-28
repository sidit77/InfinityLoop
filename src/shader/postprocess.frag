#version 300 es

precision mediump float;

out vec4 finalColor;

in vec2 tex_coords;

uniform sampler2D tex;
uniform float time;

vec4 foreground = vec4(1,1,1,1);
vec4 background = vec4(0,0,0,1);

void main() {
    float sd = (texture(tex, tex_coords).r - 0.5) * 10.0;

    vec2 tx = tex_coords;
    tx.y = 1.0 - tx.y;
    float sd2 = (texture(tex, tx).r - 0.5) * 10.0;

    //float f = length((tex_coords - 0.5) * 2.0) - mod(time * 0.2, 2.0);
    //float si = smoothstep(-0.3, 0.3, f);
    //finalColor = mix(background, foreground, 1.0 - clamp(abs(sd - si * 60.0) - 0.25, 0.0, 1.0));

    float f = length((tex_coords - 0.5) * 2.0) - mod(time * 0.2, 2.0);
    float sj = smoothstep(-0.25, 0.07, f);
    float si = smoothstep(-0.07, 0.25, f);
    finalColor = mix(background, foreground, 1.0 - clamp(min(abs(sd - si * 60.0), abs(sd - (1.0 - sj) * 60.0)) - 0.25, 0.0, 1.0));
}