struct VertexInput {
    @builtin(vertex_index) index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct Properties {
    center: vec2<f32>,
    zoom: f32,
    _padding: u32,
}

@group(0) @binding(0)
var<uniform> aspect: f32;

@group(1) @binding(0)
var<uniform> properties: Properties;

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
        y_diff = (1.0 / aspect - 1.0) / 2.0;
    }

    // add padding depending on aspect ratio
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

fn square(a: Complex) -> Complex {
    return Complex(
            a.x * a.x - a.y * a.y,
            2.0 * a.x * a.y,
    );
}

const lim = 256u;

fn colour(c: Complex) -> vec3<f32> {
    var z = Complex(0.0, 0.0);
    var n = 0u;
    var abs = length(z);

    while (abs < 2.0 && n < lim) {
        z = square(z) + c;
        abs = length(z);
        n++;
    }

    if (n == lim) {
        return vec3<f32>(0.0);
    }
    else {
        let n = f32(n) / f32(lim);
        let r = abs % 13.1999 / 13.12 + n;
        let g = n - sin(abs / 1.7) / 21.3 + z.y / 66.7;
        let b = n - abs % 0.123 + (sin(n * 12.3) + 1.0) / 1999.1213;
        return vec3<f32>(r, g, b);
    }

}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let c = ((vertex.tex_coords.xy - vec2<f32>(0.5, 0.5)) * properties.zoom + properties.center);
    return vec4<f32>(colour(c), 1.0);
}
