#version 430 core

uniform mat4 view;
uniform mat4 projection;
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 uv;
out vec2 var_uv;

void main()
{
    var_uv = uv;
    gl_Position = projection * view * vec4(aPos, 1.0);
}
