#version 430 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 vertexColor;
layout(location = 2) in vec3 normal;

layout(location = 0) uniform mat4 transformation;

layout(location = 1) uniform mat4 modelMatrix;

smooth out vec4 color;
smooth out vec3 vertexNormal;
smooth out vec3 fragmentPosition;

void main()
{

    vec4 vertexPosition = vec4(position, 1.0);

    // position on screen
    gl_Position = transformation * vertexPosition;

    color = vertexColor;

    // normal in world space
    vertexNormal = normalize(mat3(modelMatrix) * normal);

    // vertex position in world space
    fragmentPosition = vec3(modelMatrix * vertexPosition);
}