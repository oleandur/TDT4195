#version 430 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 vertexColor;
layout(location = 2) in vec3 normal;

layout(location = 0) uniform mat4 transformation;

smooth out vec4 color;
smooth out vec3 vertexNormal;

void main()
{

    vec4 vertexPosition = vec4(position.x, position.y, position.z, 1.0);

    gl_Position = transformation * vertexPosition;

    color = vertexColor;

    vertexNormal = normal;
}