#version 430 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 vertexColor;

layout(location = 0) uniform mat4 transformation;

smooth out vec4 color;
// noperspective out vec4 color;

void main()
{
    // mat4 transformation = mat4(1.0);

    // transformation[0][0] = 1.2; // a
    // transformation[1][0] = 0.4; // b
    // transformation[3][0] = 0.2; // c
    // transformation[0][1] = 0.4; // d
    // transformation[1][1] = 1.2; // e
    // transformation[3][1] = 0.2; // f

    vec4 vertexPosition = vec4(-position.x, -position.y, position.z, 1.0);

    gl_Position = transformation * vertexPosition;

    color = vertexColor;
}