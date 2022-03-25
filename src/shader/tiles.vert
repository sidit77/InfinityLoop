#version 300 es

layout(location = 0) in mat3 model;
layout(location = 3) in uint texId;

const vec2 vertex_positions[4] = vec2[4](
    vec2(-1., -1.),
    vec2( 1., -1.),
    vec2(-1.,  1.),
    vec2( 1.,  1.)
);


uniform mat3 camera;

out vec3 tex_coords;

void main() {
    vec2 vertex_position = vertex_positions[gl_VertexID];
    tex_coords = vec3((vertex_position + vec2(1., 1.)) * 0.5, float(texId));
    vec3 position = camera * model * vec3(vertex_position, 1);
    gl_Position = vec4(position.xy / position.z, 0, 1);

}