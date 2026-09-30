#version 430 core

smooth in vec4 color;
smooth in vec3 vertexNormal;

out vec4 fragColor;

void main()
{
    vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));

    float brightness = max(0.0, dot(normalize(vertexNormal), -lightDirection));

    fragColor = vec4(color.rgb * brightness, color.a);
}