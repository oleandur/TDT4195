#version 430 core

smooth in vec4 color;
// noperspective in vec4 color;

out vec4 fragColor;

void main()
{
    fragColor = color;
}