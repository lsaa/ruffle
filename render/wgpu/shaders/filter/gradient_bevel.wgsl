struct Filter {
    strength: f32,
    bevel_type: u32,
    knockout: u32,
    composite_source: u32,
}

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> filter_args: Filter;
@group(0) @binding(3) var blurred: texture_2d<f32>;
@group(0) @binding(4) var gradient: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) source_uv: vec2<f32>,
    @location(1) blur_uv_left: vec2<f32>,
    @location(2) blur_uv_right: vec2<f32>,
};

struct VertexInput {
    /// The position of the vertex in texture space (topleft 0,0, bottomright 1,1)
    @location(0) position: vec2<f32>,

    /// The coordinate of the source texture to sample in texture space (topleft 0,0, bottomright 1,1)
    @location(1) source_uv: vec2<f32>,

    /// The coordinate of the blur texture to sample in texture space (topleft 0,0, bottomright 1,1)
    @location(2) blur_uv_left: vec2<f32>,

    /// The coordinate of the blur texture to sample in texture space (topleft 0,0, bottomright 1,1)
    @location(3) blur_uv_right: vec2<f32>,
};

@vertex
fn main_vertex(in: VertexInput) -> VertexOutput {
    // Convert texture space (topleft 0,0 to bottomright 1,1) to render space (topleft -1,1 to bottomright 1,-1)
    let pos = vec4<f32>((in.position.x * 2.0 - 1.0), (1.0 - in.position.y * 2.0), 0.0, 1.0);
    return VertexOutput(pos, in.source_uv, in.blur_uv_left, in.blur_uv_right);
}

@fragment
fn main_fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let knockout = filter_args.knockout > 0u;
    let composite_source = filter_args.composite_source > 0u;
    var blur_left = textureSample(blurred, texture_sampler, in.blur_uv_left).a;
    var blur_right = textureSample(blurred, texture_sampler, in.blur_uv_right).a;
    let dest = textureSample(texture, texture_sampler, in.source_uv);

    let outer = filter_args.bevel_type == 0u || filter_args.bevel_type == 2u;
    let inner = filter_args.bevel_type == 1u || filter_args.bevel_type == 2u;

    if (in.blur_uv_left.x < 0.0 || in.blur_uv_left.x > 1.0 || in.blur_uv_left.y < 0.0 || in.blur_uv_left.y > 1.0) {
        blur_left = 0.0;
    }
    if (in.blur_uv_right.x < 0.0 || in.blur_uv_right.x > 1.0 || in.blur_uv_right.y < 0.0 || in.blur_uv_right.y > 1.0) {
        blur_right = 0.0;
    }

    // Signed alpha difference maps to the 256 gradient entries, with zero at 128.
    let index = i32(clamp(128.0 + (blur_left - blur_right) * filter_args.strength * 127.5, 0.0, 255.0));
    let color = textureLoad(gradient, vec2<i32>(index, 0), 0);
    let glow = vec4<f32>(color.rgb * color.a, color.a);

    if (inner && outer) {
        if (knockout || !composite_source) {
            return glow;
        } else {
            return dest - dest * glow.a + glow;
        }
    } else if (inner) {
        if (knockout || !composite_source) {
            return glow * dest.a;
        } else {
            return glow * dest.a + dest * (1.0 - glow.a);
        }
    } else {
        if (knockout || !composite_source) {
            return glow - glow * dest.a;
        } else {
            return dest + glow - glow * dest.a;
        }
    }
}
