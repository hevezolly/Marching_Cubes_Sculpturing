#version 430 core

uniform mat4 view;
uniform mat4 projection;
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 uv;
out vec3 var_vert_pos;
out vec2 var_uv;

void main()
{
    vec4 position = projection * view * vec4(aPos.xyz, 1.0);
    var_vert_pos = position.xyz;
    var_uv = uv;
    gl_Position = position;
}