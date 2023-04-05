struct VertexInput {
    @builtin(vertex_index) index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

fn index_to_pos(index: u32) -> vec2<f32> {
    switch index {
        case 0u: { return vec2<f32>(-1.0, -1.0); }
        case 1u: { return vec2<f32>(-1.0,  1.0); }
        case 2u: { return vec2<f32>( 1.0, -1.0); }
        case 3u: { return vec2<f32>(-1.0,  1.0); }
        case 4u: { return vec2<f32>( 1.0, -1.0); }
        default: { return vec2<f32>( 1.0,  1.0); }
    }
}

fn index_to_tex(index: u32) -> vec2<f32> {
    switch index {
        case 0u: { return vec2<f32>(0.0, 0.0); }
        case 1u: { return vec2<f32>(0.0, 1.0); }
        case 2u: { return vec2<f32>(1.0, 0.0); }
        case 3u: { return vec2<f32>(0.0, 1.0); }
        case 4u: { return vec2<f32>(1.0, 0.0); }
        default: { return vec2<f32>(1.0, 1.0); }
    }
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(index_to_pos(in.index), 0.0, 1.0);
    out.tex_coords    = index_to_tex(in.index);
    return out;
}

type Complex = vec2<f32>;

fn abs(a: Complex) -> f32 {
    return sqrt(a.x * a.x + a.y * a.y);
}

fn square(a: Complex) -> Complex {
    return Complex(
            a.x * a.x - a.y * a.y,
            2.0 * a.x * a.y,
    );
}

fn colour(c: Complex) -> f32 {
    var z = Complex(0.0, 0.0);
    var n = 255u;

    while (abs(z) < 2.0 && n > 0u) {
        z = square(z) + c;
        n--;
    }
    return f32(n) / 255.0;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let c = (vertex.tex_coords.xy - vec2<f32>(0.75, 0.5)) * 2.0;
    let s = vec3<f32>(colour(c));
    return vec4<f32>(s, 1.0);
}
