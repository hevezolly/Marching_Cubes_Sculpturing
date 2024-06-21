#version 430 core

in vec3 chunk_space_position;
in vec3 chunk_space_normal;
// in vec3 world_position;
in vec3 world_normal;

uniform sampler3D scalar_field;
uniform vec3 light_direction;
uniform ivec3 field_chunk_size_diff;
uniform float surface_level;
uniform float ao_max_dist;
uniform float ao_falloff;

out vec4 FragColor;

vec3 to_texture_space(vec3 chunk_pos, vec3 tex_dim) {
    vec3 chunk_dim = tex_dim - field_chunk_size_diff;

    return ((chunk_pos - vec3(0.5)) * (chunk_dim)) / 
           (tex_dim) + vec3(0.5);
}

float inv_lerp(float a, float b, float v) {
    return (v - a) / (b - a);
}

#define PI (3.14159265)

float random(vec2 st)
{
    return fract(sin(dot(st.xy, vec2(12.9898,78.233))) * 43758.5453123);
}

vec3 random_hemisphere(vec3 dir, float seed) {
    float s = random(vec2(seed + 1.)) * PI * 2.;
    float t = random(vec2(seed + 2.)) * 2 - 1.;
    vec3 rand_sphere = vec3(sin(s), cos(s), t) / sqrt(1.0 + t * t);

    return rand_sphere * sign(dot(dir, rand_sphere));
}



float sample_field(vec3 tex_position, vec3 tex_dim) {
    // float lod = inv_lerp(tex_dim.x, tex_dim.x * 0.5, tex_dim.x - float(field_chunk_size_diff.x));
    return texture(scalar_field, tex_position).x - surface_level;
}

float ambient_occlusion(vec3 position, vec3 normal) {
    vec3 tex_dim = vec3(textureSize(scalar_field, 0).xyz);

    float ao = 0.;

    const int iters = 64;
    const float iters_inv = 1. / float(iters);
    const float rad = 1. - 1. * iters_inv;

    for (int i = 1; i <= iters; i++) {
        float l = ao_max_dist * random(vec2(i, i));
        vec3 direction = normalize(normal + random_hemisphere(normal, l) * rad);
        vec3 p = to_texture_space(position + direction * l, tex_dim);


        ao += max(l - max(sample_field(p, tex_dim), 0.), 0.) / 
            ao_max_dist * ao_falloff;
    }

    return clamp(1. - ao / float(iters), 0., 1.);
}

void main() {
    // FragColor = vec4(chunk_space_normal, 1.);

    vec3 color = vec3(0.31, 0.65, 0.48);
    float light = (dot(light_direction, normalize(world_normal)) + 1.) * 0.5 ;

    float ambient_occlusion = ambient_occlusion(chunk_space_position, chunk_space_normal);
    
    vec3 color_shadow = vec3(0.09, 0.4, 0.63);
    vec3 lighted = color * light + color_shadow * (1 - light);
    
    FragColor = vec4(lighted * ambient_occlusion, 1.);
}
