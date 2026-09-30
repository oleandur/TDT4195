#version 430 core

smooth in vec4 color;
smooth in vec3 vertexNormal;

out vec4 fragColor;

void main()
{
    fragColor = vec4(vertexNormal, 1.0);
}