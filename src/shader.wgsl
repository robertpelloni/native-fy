// Vertex shader
struct Globals {
    screen_size: vec2<f32>,
};

struct NodeData {
    pos: vec2<f32>,
    size: vec2<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var<storage, read> nodes: array<NodeData>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @builtin(instance_index) instance_idx: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    let node = nodes[model.instance_idx];

    // Scale unit quad (0,0 to 1,1) by node size
    let pos = model.position.xy * node.size + node.pos;

    // Convert to normalized device coordinates (-1 to 1)
    // wgpu uses 0,0 at top-left for surface, but NDC is -1,1 to 1,-1 (y up)
    // Wait, wgpu NDC is -1,1 at top-left to 1,-1 at bottom-right?
    // Standard is -1,1 top-left, but y is down in screen space.
    // NDC: x from -1 to 1, y from 1 to -1.

    let ndc_x = (pos.x / globals.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (pos.y / globals.screen_size.y) * 2.0;

    var out: VertexOutput;
    out.color = node.color;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
