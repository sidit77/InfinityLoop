#version 300 es

out vec2 tex_coords;

const vec2 vertex_positions[4] = vec2[4](
    vec2(-1., -1.),
    vec2( 1., -1.),
    vec2(-1.,  1.),
    vec2( 1.,  1.)
);

void main() {
    vec2 position = vertex_positions[gl_VertexID];
    gl_Position = vec4(position, 0, 1);
    tex_coords = (position + vec2(1., 1.)) * 0.5;
}