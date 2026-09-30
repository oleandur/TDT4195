#version 430 core

smooth in vec4 color;
smooth in vec3 vertexNormal;
smooth in vec3 fragmentPosition;

layout(location = 2) uniform vec3 cameraPosition;

out vec4 fragColor;

void main()
{
    vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));

    vec3 normal = normalize(vertexNormal);

    // ambient
    float ambientStrength = 0.15;
    vec3 ambient = ambientStrength * color.rgb;

    // diffuse (Lambert)

    float diffuseStrength = max(dot(normal, -lightDirection), 0.0);

    vec3 diffuse = diffuseStrength * color.rgb;

    // specular

    vec3 viewDirection = normalize(cameraPosition - fragmentPosition);

    vec3 reflectedDirection = reflect(lightDirection, normal);

    float specularStrength = 0.5;
    float shininess = 32.0;

    float specularAmount = 0.0;

    if (diffuseStrength > 0.0) {
        specularAmount = pow(max(dot(viewDirection, reflectedDirection), 0.0), shininess);
    }

    vec3 specular = specularStrength *specularAmount *vec3(1.0);

    // Final Phong color
    vec3 finalColor = ambient + diffuse + specular;


    fragColor = vec4(finalColor, color.a);
}