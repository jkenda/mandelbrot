struct VertexInput {
    @builtin(vertex_index) index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> aspect: f32;

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
    var x_diff = 0.0;
    var y_diff = 0.0;
    if aspect > 1.0 {
        x_diff = (aspect - 1.0) / 2.0;
    }
    else if aspect < 1.0 {
        y_diff = (1.0 - aspect) / 2.0;
    }

    let x0 = 0.0 - x_diff;
    let x1 = 1.0 + x_diff;
    let y0 = 0.0 - y_diff;
    let y1 = 1.0 + y_diff;

    switch index {
        case 0u: { return vec2<f32>(x0, y0); }
        case 1u: { return vec2<f32>(x0, y1); }
        case 2u: { return vec2<f32>(x1, y0); }
        case 3u: { return vec2<f32>(x0, y1); }
        case 4u: { return vec2<f32>(x1, y0); }
        default: { return vec2<f32>(x1, y1); }
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
