//! https://www.w3.org/TR/WGSL/#builtin-functions

// #import bevy_sprite::mesh2d_vertex_output MeshVertexOutput

// taken from https://github.com/bevyengine/bevy/blob/264195ed772e1fc964c34b6f1e64a476805e1766/crates/bevy_sprite/src/mesh2d/mesh2d_vertex_output.wgsl
struct MeshVertexOutput {
    // this is `clip position` when the struct is used as a vertex stage output 
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    // #ifdef VERTEX_TANGENTS
    // @location(3) world_tangent: vec4<f32>,
    // #endif
    // #ifdef VERTEX_COLORS
    // @location(4) color: vec4<f32>,
    // #endif
}

struct CustomMaterial {
    color: vec4<f32>,
    size: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;

@fragment
fn fragment(
    mesh: MeshVertexOutput,
) -> @location(0) vec4<f32> {
    let color = material.color;
    let size = material.size;

    // the size matters because we want the shadow blur to be in physical pixels
    // but uv is relative to the shadow blur
    // let's imagine that we want 100px of blur
    // if size = 1000, then that's 10%
    // more generally
    let lx = 70.0 / size.x;
    var ax: f32;
    if mesh.uv.x <= lx {
      ax = smoothstep(0.0, 1.0, mesh.uv.x / lx);
    } else if 1.0 - mesh.uv.x <= lx {
      ax = smoothstep(0.0, 1.0, (1.0 - mesh.uv.x) / lx);
    } else {
      ax = 1.0;
    }

    let ly = 70.0 / size.y;
    var ay: f32;
    if mesh.uv.y <= ly {
      ay = smoothstep(0.0, 1.0, mesh.uv.y / ly);
    } else if 1.0 - mesh.uv.y <= ly {
      ay = smoothstep(0.0, 1.0, (1.0 - mesh.uv.y) / ly);
    } else {
      ay = 1.0;
    }

    var a = mix(ax * ay, min(ax, ay), 0.3);

    return vec4<f32>(color.rgb, a * color.a);
}