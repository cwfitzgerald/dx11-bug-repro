#version 450

layout(location = 0) in vec2 tex_coords;

layout(location = 0) out vec4 color;

layout(set = 0, binding = 0, rgba8) readonly uniform image2D image;

void main() {
    vec2 img_color = imageLoad(image, ivec2(0, 0)).rg;
    color = vec4(tex_coords.x > img_color.r ? 1.0 : 0.0, img_color.g > tex_coords.y ? 1.0 : 0.0, 0.0, 1.0);
}