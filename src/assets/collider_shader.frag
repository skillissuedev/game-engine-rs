#version 330

uniform bool sensor;

out vec4 color;

void main() {
    if (sensor == false) {
        color = vec4(0.48, 0.61, 0.96, 0.4);
    } else {
        color = vec4(0.71, 0.98, 0.3, 0.4);
    }
}

