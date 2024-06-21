#version 430 core

in vec3 world_position;
in vec3 world_normal;
out vec4 FragColor;
uniform vec3 light_direction;

void main()
{
    vec3 color = vec3(0.09, 0.4, 0.63);
    vec3 color_shadow = vec3(0.31, 0.65, 0.48);
    float light = (dot(light_direction, normalize(world_normal)) + 1.) * 0.5 ;
    FragColor = vec4(color * light + color_shadow * (1 - light), 1.);
}