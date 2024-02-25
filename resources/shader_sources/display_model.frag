#version 430 core

in vec3 var_vert_pos;
in vec3 var_normal;
out vec4 FragColor;
uniform vec3 light_direction;

void main()
{
    vec3 color = vec3(1., 0., 0.);
    float light = max(dot(light_direction, normalize(var_normal)), 0.);
    FragColor = vec4(color * light, 1.);
}