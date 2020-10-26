#version 450

layout(location = 0) in vec2 tex_coords;

layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) buffer Buffer {
    float location;
};

void main() {
    color = tex_coords.x > location ? vec4(1.0, 0.0, 0.0, 0.0) : vec4(0.0, 1.0, 0.0, 0.0);
}