#version 430 core

out vec4 color;

void main()
{
    vec2 p = gl_FragCoord.xy;

    float squareSize = 22.0;

    vec2 uv = p / squareSize;

    uv.x += 0.7 * sin(uv.y * 0.7);
    uv.y += 0.7 * sin(uv.x * 0.8);

    uv.x += 0.25 * sin(uv.y * 1.7);
    uv.y += 0.25 * sin(uv.x * 1.5);


    float checker = mod(floor(uv.x) + floor(uv.y), 2.0);

    if (checker < 1.0)
    {
        color = vec4(0.0, 0.0, 0.0, 1.0);
    }
    else
    {
        color = vec4(1.0, 1.0, 1.0, 1.0);
    }
}