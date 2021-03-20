#version 450

// old method
//const vec2 positions[3] = vec2[3](
//    vec2(0.0,0.5),
//    vec2(-0.5,-0.5),
//    vec2(0.5,-0.5)
//);

layout (location = 0) in vec3 a_position;
layout (location = 1) in vec2 a_tex_coords;

layout (location = 0) out vec2 v_tex_coords;

void main () {
    v_tex_coords = a_tex_coords;
    gl_Position = vec4(a_position,1.0);
}