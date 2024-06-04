#version 430 core


uniform mat4 view;
uniform mat4 projection;
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aNorm;
out vec3 var_vert_pos;
out vec3 var_normal;

void main()
{
    vec4 position = projection * view * vec4(aPos.xyz, 1.0);
    var_vert_pos = position.xyz;
    var_normal = aNorm;
    gl_Position = position;
}