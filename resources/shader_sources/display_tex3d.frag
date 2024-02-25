#version 430 core

in vec2 var_uv;
out vec4 FragColor;
uniform sampler3D image;
uniform float slice;

void main()
{
    vec4 value = texture(image, vec3(var_uv.xy, slice));
    FragColor = vec4(value.x, 0., 0., 1.);
}