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

float opSmoothSubtraction( float d1, float d2, float k ) {
    float h = clamp( 0.5 - 0.5*(d2+d1)/k, 0.0, 1.0 );
    return mix( d2, -d1, h ) + k*h*(1.0-h);
}

void main() {
    //float sd = (texture(tex, tex_coords).r - 0.5) * 10.0;
//
    float f = length(world_pos - center) - radius;
//
    //float final_opacity = abs(min(opSmoothSubtraction(sd, (f + 0.1) * pxRange, 12.0), opSmoothSubtraction(sd, -(f - 0.1) * pxRange, 12.0))) - 0.25;
    //final_opacity = abs(final_opacity) - 0.15;
//
    //finalColor = mix(mix(background1, background2, smoothstep(-0.1, 0.1, f)), foreground, 1.0 - clamp(final_opacity, 0.0, 1.0));
    float sd = (texture(tex, tex_coords).r - 0.5) * 10.0;

    float opa_out = abs(sd + 1.5) - 0.3;
    float opa_in = sd + 1.5;
    float mix_fac = smoothstep(-4.0, 4.0, f * pxRange);

    float final_opacity = mix(opa_in, opa_out, mix_fac);


    finalColor = mix(background2, mix(background1, foreground, mix_fac), 1.0 - clamp(final_opacity, 0.0, 1.0));

}