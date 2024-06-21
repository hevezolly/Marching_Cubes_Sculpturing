#version 430 core


uniform mat4 view;
uniform mat4 projection;
uniform mat4 model;
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNorm;
out vec3 world_position;
out vec3 world_normal;

void main()
{
    vec4 wp = model * vec4(aPos.xyz, 1.0);
    world_position = wp.xyz;
    world_normal = normalize(transpose(inverse(mat3(model))) * aNorm);
    gl_Position = projection * view * wp;
}