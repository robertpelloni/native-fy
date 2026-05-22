// Vertex shader
struct Globals {
    screen_size: vec2<f32>,
};

struct NodeData {
    pos: vec2<f32>,
    size: vec2<f32>,
    color: vec4<f32>,
    mode: u32, // 0 = Color, 1 = Texture
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<storage, read> nodes: array<NodeData>;
@group(0) @binding(2) var t_diffuse: texture_2d<f32>;
@group(0) @binding(3) var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @builtin(instance_index) instance_idx: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) @interpolate(flat) mode: u32,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let node = nodes[model.instance_idx];

    // Scale unit quad (0,0 to 1,1) by node size
    let pos = model.position.xy * node.size + node.pos;

    let ndc_x = (pos.x / globals.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (pos.y / globals.screen_size.y) * 2.0;

    var out: VertexOutput;
    out.color = node.color;
    out.uv = model.position.xy;
    out.mode = node.mode;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.mode == 1u) {
        return textureSample(t_diffuse, s_diffuse, in.uv) * in.color;
    }
    return in.color;
}
