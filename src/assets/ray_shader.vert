#version 330

in vec3 position;
in vec3 normal;
in vec2 tex_coords;
in vec4 joints;
in vec4 weights;

uniform mat4 view;
uniform mat4 proj;

void main() {
    gl_Position = proj * view * vec4(position, 1.0);
}
