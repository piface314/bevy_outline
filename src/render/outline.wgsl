#import bevy_pbr::{
    mesh_bindings::mesh,
    mesh_functions::get_world_from_local,
    mesh_view_bindings::view,
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@group(2) @binding(0) var<uniform> outline_width: f32;
@group(2) @binding(1) var<uniform> outline_color: vec4<f32>;


struct DoubleReciprocalWindowSize {
    size: vec2<f32>,
};

@group(3) @binding(0)
var<uniform> window_size: DoubleReciprocalWindowSize;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let mvp = view.clip_from_world * get_world_from_local(vertex.instance_index);
    let clip_position = mvp * vec4<f32>(vertex.position, 1.0);
    let clip_normal = mvp * vec4<f32>(vertex.normal, 0.0);
    let extrude_offset = normalize(clip_normal.xy) * outline_width * clip_position.w * window_size.size;
    var out: VertexOutput;
    out.clip_position = vec4<f32>(clip_position.xy + extrude_offset, clip_position.zw);
    return out;
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return outline_color;
}
