#version 450

layout(constant_id = 5) const int index = 2;

struct S {
    vec3 val1;
    bool val2[5];
};

layout(set = 0, binding = 0) uniform sampler2D u_texture;

layout(set = 0, binding = 1) uniform Block {
    S u_data;
} block;

layout(location = 0) in vec2 v_texcoords;
layout(location = 0) out vec4 f_color;

void main() {
    if (block.u_data.val2[index]) {
        f_color = texture(u_texture, v_texcoords);
    } else {
        f_color = vec4(1.0);
    }
}
