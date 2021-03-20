#version 450


layout ( location = 0) in vec2 v_tex_coords;
layout (location = 0 ) out vec4 f_color;

// put these together to make the first valuee for the texture function
layout(set = 0,binding = 0) uniform texture2D t_diffuse; // thihs is our texture vieew
layout(set = 0, binding = 1) uniform sampler s_diffuse;// thtis is the sampler we created

void main () {
    vec4 res =texture(sampler2D(t_diffuse,s_diffuse),v_tex_coords); 
    f_color = res;
}