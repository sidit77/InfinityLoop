#version 300 es

precision highp float;

out vec4 finalColor;

in vec2 tex_coords;
in vec2 world_pos;

uniform sampler2D tex;
uniform float radius;
uniform float pxRange;
uniform vec2 center;

vec4 foreground  = vec4(0.847,0.871,0.914,1.0);
vec4 background1 = vec4(0.231,0.259,0.322,1.0);
vec4 background2 = vec4(0.180,0.204,0.251,1.0);

float opSmoothIntersection( float d1, float d2, float k ) {
    float h = clamp( 0.5 - 0.5*(d2-d1)/k, 0.0, 1.0 );
    return mix( d2, d1, h ) + k*h*(1.0-h);
}

void main() {
    float sd = (texture(tex, tex_coords).r - 0.5) * 10.0;

    float f = length(world_pos - center) - radius;

    float final_opacity = opSmoothIntersection(sd, -(abs(f * pxRange) - 0.1 * pxRange), 12.0);
    final_opacity = abs(final_opacity) - 0.25;

    finalColor = mix(mix(background1, background2, smoothstep(-0.1, 0.1, f)), foreground, 1.0 - clamp(final_opacity, 0.0, 1.0));

}