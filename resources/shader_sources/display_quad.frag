#version 430 core

uniform sampler3D volume;
uniform float slice;
in vec3 var_vert_pos;
in vec2 var_uv;
out vec4 FragColor;

void main()
{ 
    FragColor = vec4(texture(volume, vec3(var_uv.xy, slice)).xyz, 1.);
}