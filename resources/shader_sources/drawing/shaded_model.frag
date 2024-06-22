#version 430 core

in vec3 chunk_space_position;
in vec3 chunk_space_normal;
// in vec3 world_position;
in vec3 world_normal;
in vec2 clip_position;

uniform sampler3D scalar_field;
uniform vec3 light_direction;
uniform ivec3 field_chunk_size_diff;
uniform float surface_level;
uniform float ao_max_dist;
uniform float ao_falloff;
uniform float ao_upper_edge;

out vec4 FragColor;

vec3 to_texture_space(vec3 chunk_pos, vec3 tex_dim) {
    vec3 chunk_dim = tex_dim - field_chunk_size_diff;

    return ((chunk_pos - vec3(0.5)) * (chunk_dim)) / 
           (tex_dim) + vec3(0.5);
}

float inv_lerp(float a, float b, float v) {
    return (v - a) / (b - a);
}

#define PI 3.14159265358979323846

float random(vec2 st)
{
    return fract(sin(dot(st.xy, vec2(12.9898,78.233))) * 43758.5453123);
}

uint part1by1 (uint x) {
    x = (x & 0x0000ffffu);
    x = ((x ^ (x << 8u)) & 0x00ff00ffu);
    x = ((x ^ (x << 4u)) & 0x0f0f0f0fu);
    x = ((x ^ (x << 2u)) & 0x33333333u);
    x = ((x ^ (x << 1u)) & 0x55555555u);
    return x;
}
    
uint compact1by1 (uint x) {
    x = (x & 0x55555555u);
    x = ((x ^ (x >> 1u)) & 0x33333333u);
    x = ((x ^ (x >> 2u)) & 0x0f0f0f0fu);
    x = ((x ^ (x >> 4u)) & 0x00ff00ffu);
    x = ((x ^ (x >> 8u)) & 0x0000ffffu);
    return x;
}
    
uint pack_morton2x16(uvec2 v) {
	return part1by1(v.x) | (part1by1(v.y) << 1);
}

uvec2 unpack_morton2x16(uint p) {
    return uvec2(compact1by1(p), compact1by1(p >> 1));
}

uint inverse_gray32(uint n) {
    n = n ^ (n >> 1);
    n = n ^ (n >> 2);
    n = n ^ (n >> 4);
    n = n ^ (n >> 8);
    n = n ^ (n >> 16);
    return n;
}

// https://www.shadertoy.com/view/llGcDm
int hilbert( ivec2 p, int level )
{
    int d = 0;
    for( int k=0; k<level; k++ )
    {
        int n = level-k-1;
        ivec2 r = (p>>n)&1;
        d += ((3*r.x)^r.y) << (2*n);
    	if (r.y == 0) { if (r.x == 1) { p = (1<<n)-1-p; } p = p.yx; }
    }
    return d;
}

// https://www.shadertoy.com/view/llGcDm
ivec2 ihilbert( int i, int level )
{
    ivec2 p = ivec2(0,0);
    for( int k=0; k<level; k++ )
    {
        ivec2 r = ivec2( i>>1, i^(i>>1) ) & 1;
        if (r.y==0) { if(r.x==1) { p = (1<<k) - 1 - p; } p = p.yx; }
        p += r<<k;
        i >>= 2;
    }
    return p;
}

// knuth's multiplicative hash function (fixed point R1)
uint kmhf(uint x) {
    return 0x80000000u + 2654435789u * x;
}

uint kmhf_inv(uint x) {
    return (x - 0x80000000u) * 827988741u;
}

// mapping each pixel to a hilbert curve index, then taking a value from the Roberts R1 quasirandom sequence for it
uint hilbert_r1_blue_noise(uvec2 p) {
    #if 1
    uint x = uint(hilbert( ivec2(p), 17 )) % (1u << 17u);
    #else
    //p = p ^ (p >> 1);
    uint x = pack_morton2x16( p ) % (1u << 17u);    
    //x = x ^ (x >> 1);
    x = inverse_gray32(x);
    #endif
    #if 0
    // based on http://extremelearning.com.au/unreasonable-effectiveness-of-quasirandom-sequences/
    const float phi = 2.0/(sqrt(5.0)+1.0);
	return fract(0.5+phi*float(x));
    #else
    x = kmhf(x);
    return x;
    #endif
}

// mapping each pixel to a hilbert curve index, then taking a value from the Roberts R1 quasirandom sequence for it
float hilbert_r1_blue_noisef(uvec2 p) {
    uint x = hilbert_r1_blue_noise(p);
    #if 0
    return float(x >> 24) / 256.0;
    #else
    return float(x) / 4294967296.0;
    #endif
}

float rand(vec2 co){return fract(sin(dot(co.xy ,vec2(12.9898,78.233))) * 43758.5453);}
float rand (vec2 co, float l) {return rand(vec2(rand(co), l));}
float rand (vec2 co, float l, float t) {return rand(vec2(rand(co, l), t));}

float perlin(vec2 p, float dim, float time) {
	vec2 pos = floor(p * dim);
	vec2 posx = pos + vec2(1.0, 0.0);
	vec2 posy = pos + vec2(0.0, 1.0);
	vec2 posxy = pos + vec2(1.0);
	
	float c = rand(pos, dim, time);
	float cx = rand(posx, dim, time);
	float cy = rand(posy, dim, time);
	float cxy = rand(posxy, dim, time);
	
	vec2 d = fract(p * dim);
	d = -0.5 * cos(d * PI) + 0.5;
	
	float ccx = mix(c, cx, d.x);
	float cycxy = mix(cy, cxy, d.x);
	float center = mix(ccx, cycxy, d.y);
	
	return center * 2.0 - 1.0;
}


vec3 random_hemisphere(vec3 dir, float seed, float seed2, float seed3) {
    float u1 = mix(random(vec2(seed + 1.)), seed2, 0.1);
    float u2 = mix(random(vec2(seed + 2.)), seed3, 0.1);

    float theta = acos(u1);
    float phi = 2 * PI * u2;

    float sin_theta = sin(theta);
    float x = cos(phi) * sin_theta;
    float y = sin(phi) * sin_theta;
    float z = u1;

    vec3 result = vec3(x, y, z);

    float proj = dot(dir, result);
    if (proj < 0)
        result -= 2 * dir * proj;

    return normalize(result); 

    // vec3 local_space = vec3(x, y, sqrt(max(0., 1 - u1)));

    // float sign = sign(dir.z);
    // sign = 1;
    // float a = -1.0 / (sign + dir.z);
    // float b = dir.x * dir.y * a;
    // vec3 tangent = vec3(1.0 + sign * dir.x * dir.x * a, sign * b, -sign * dir.x);
    // vec3 bitangent = vec3(b, sign + dir.y * dir.y * a, -dir.y);

    // return local_space.x * tangent + local_space.y * bitangent + local_space.z * dir;
}

vec3 rotate_vector(vec3 a, vec3 b, float angle) {
    vec3 a_par_b  = dot(a, b) * b / dot(b, b);

    vec3 a_perp_b = a - a_par_b;
    vec3 w = cross(b, a_perp_b);

    float x1 = cos(angle) / length(a_perp_b);
    float x2 = sin(angle) / length(w);

    vec3 a_perp_b_rotated = length(a_perp_b) * (x1 * a_perp_b + x2 * w);
    return a_perp_b_rotated + a_par_b;
} 


float sample_field(vec3 tex_position, vec3 tex_dim) {
    // float lod = inv_lerp(tex_dim.x, tex_dim.x * 0.5, tex_dim.x - float(field_chunk_size_diff.x));
    return texture(scalar_field, tex_position).x - surface_level;
}

float ambient_occlusion(vec3 position, vec3 normal) {
    vec3 tex_dim = vec3(textureSize(scalar_field, 0).xyz);

    float ao = 0.;

    const int rays = 48;
    const int ray_steps = 4;

    const float ao_upper_edge = 0.07;
    // const float ao_falloff = 0.9;

    float step_len = ao_max_dist / ray_steps;

    float seed = hilbert_r1_blue_noisef(uvec2(gl_FragCoord));
    float seed2 = hilbert_r1_blue_noisef(uvec2(gl_FragCoord + 10));

    // float seed = random(position.xy);
    // float seed2 = random(position.yz);

    for (int i = 1; i <= rays; i++) {
        vec3 dir = random_hemisphere(normal, i, seed, seed2);
        // vec3 dir = random_hemisphere(normal, i);

        float min_dist = ao_max_dist;

        float weight = (dot(normal, dir) + 1. ) * 0.5;

        for (int step = 1; step <= ray_steps; step++) {
            
            float l = step * step_len;
            // float l = ao_max_dist * random(vec2(i, i ));
            vec3 p = to_texture_space(position + l * dir, tex_dim);

            float sample_val = max(0.,sample_field(p, tex_dim));
            if (sample_val < ao_upper_edge) {
                min_dist = min(min_dist, sample_val / ao_upper_edge);
            }
            

            // min_dist = min(max(l - sample_val, 0.), min_dist);
        }

        ao += (ao_max_dist - min_dist) * weight / ao_max_dist * ao_falloff;

        // ao += max(l - max(sample_value, 0.), 0.) / 
        //     ao_max_dist * ao_falloff * weight;
    }

    return clamp(1. - ao / float(rays), 0., 1.);
}

void main() {
    // FragColor = vec4(chunk_space_normal, 1.);

    vec3 color = vec3(0.31, 0.65, 0.48);
    float light = (dot(light_direction, normalize(world_normal)) + 1.) * 0.5 ;

    // float seed = random(vec2(chunk_space_position.x, chunk_space_position.y) * chunk_space_position.z);
    
    // float seed = 1.;

    float ambient_occlusion = ambient_occlusion(chunk_space_position, chunk_space_normal);
    
    vec3 color_shadow = vec3(0.09, 0.4, 0.63);
    vec3 lighted = color * light + color_shadow * (1 - light);
    
    FragColor = vec4(lighted * ambient_occlusion, 1.);
    // FragColor = vec4(ambient_occlusion.xxx, 1.);
    // FragColor = vec4(seed.xxx, 1.);
}
