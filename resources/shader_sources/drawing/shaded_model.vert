#version 430 core

uniform mat4 view;
uniform mat4 projection;
uniform mat4 model;
uniform vec3 chunk_scale_factor;
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNorm;


out vec3 chunk_space_position;
out vec3 chunk_space_normal;
out vec2 clip_position;
// out vec3 world_position;
out vec3 world_normal;

void main() 
{
    vec4 pos = projection * view * model * vec4(aPos, 1.);
    clip_position = pos.xy / pos.w;
    gl_Position = pos;
    world_normal = normalize(transpose(inverse(mat3(model))) * aNorm);
    chunk_space_position = aPos * chunk_scale_factor;
    chunk_space_normal = aNorm;
}