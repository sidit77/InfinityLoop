#version 300 es

out vec2 tex_coords;
out vec2 world_pos;

const vec2 vertex_positions[4] = vec2[4](
    vec2(-1., -1.),
    vec2( 1., -1.),
    vec2(-1.,  1.),
    vec2( 1.,  1.)
);

uniform mat3 inv_camera;

void main() {
    vec2 position = vertex_positions[gl_VertexID];
    gl_Position = vec4(position, 0, 1);
    tex_coords = (position + vec2(1., 1.)) * 0.5;
    vec3 tmp = inv_camera * vec3(position, 1);
    world_pos = tmp.xy / tmp.z;
}