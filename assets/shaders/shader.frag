#version 450

layout(location = 0) in vec3 frag_colour;
layout(location = 1) in vec2 frag_tex_coord;

layout(location = 0) out vec4 out_colour;

layout(binding = 1) uniform sampler2D tex_sampler;

layout(push_constant) uniform PushConstants
{
    layout(offset = 64) float opacity;
} pcs;

void main() 
{
    out_colour = vec4(texture(tex_sampler, frag_tex_coord).rgb, pcs.opacity);
}