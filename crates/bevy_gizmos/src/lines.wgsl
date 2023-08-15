// TODO use common view binding
#import bevy_render::view View

@group(0) @binding(0)
var<uniform> view: View;


struct LineGizmoUniform {
    line_width: f32,
    depth_bias: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _padding: vec2<f32>,
#endif
}

@group(1) @binding(0)
var<uniform> line_gizmo: LineGizmoUniform;

struct VertexInput {
    @location(0) position_a: vec3<f32>,
    @location(1) position_b: vec3<f32>,
    @location(2) color_a: vec4<f32>,
    @location(3) color_b: vec4<f32>,
    @builtin(vertex_index) index: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(vertex: VertexInput) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2(0., -0.5),
        vec2(1., -0.5),
        vec2(1., 0.5),
        vec2(0., -0.5),
        vec2(1., 0.5),
        vec2(0., 0.5)
    );
    let position = positions[vertex.index];

    var color = mix(vertex.color_a, vertex.color_b, position.x);

    var line_width = line_gizmo.line_width;

#ifdef PERSPECTIVE
    line_width /= clip.w;
#endif

    // Line thinness fade from https://acegikmo.com/shapes/docs/#anti-aliasing
    if line_width < 1. {
        color.a *= line_width;
        line_width = 1.;
    }

    // algorithm based on https://wwwtyro.net/2019/11/18/instanced-lines.html
    let clip_a = view.view_proj * vec4(vertex.position_a, 1.);
    let clip_b = view.view_proj * vec4(vertex.position_b, 1.);
    let clip = mix(clip_a, clip_b, position.x);

    let screen_a = clip_to_screen(clip_a);
    let screen_b = clip_to_screen(clip_b);

    let x_basis = normalize(screen_b - screen_a);
    let y_basis = vec2(-x_basis.y, x_basis.x);

    let screen_offset = y_basis * position.y * line_width;

    let resolution = view.viewport.zw;
    let offset = screen_offset / resolution;

    // TODO: Fix depth :\

    // var depth: f32;
    // if line_gizmo.depth_bias >= 0. {
    //     depth = clip.z * (1. - line_gizmo.depth_bias);
    // } else {
    //     let epsilon = 4.88e-04;
    //     // depth * (clip.w / depth)^-depth_bias. So that when -depth_bias is 1.0, this is equal to clip.w
    //     // and when equal to 0.0, it is exactly equal to depth.
    //     // the epsilon is here to prevent the depth from exceeding clip.w when -depth_bias = 1.0 
    //     // clip.w represents the near plane in homogeneous clip space in bevy, having a depth
    //     // of this value means nothing can be in front of this
    //     // The reason this uses an exponential function is that it makes it much easier for the 
    //     // user to chose a value that is convenient for them
    //     depth = clip.z * exp2(-line_gizmo.depth_bias * log2(clip.w / clip.z - epsilon));
    // }

    var clip_position = vec4(clip.xy + offset, 1., 1.);

    return VertexOutput(clip_position, color);
}

fn clip_to_screen(clip: vec4<f32>) -> vec2<f32> {
    return (clip.xy / clip.w * 0.5 + 0.5) * view.viewport.zw;
}

fn screen_to_clip(screen: vec2<f32>, clip: vec4<f32>) -> vec2<f32> {
    return (screen / view.viewport.zw - 0.5) / 0.5 * clip.w;
}

struct FragmentInput {
    @location(0) color: vec4<f32>,
};

struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

@fragment
fn fragment(in: FragmentInput) -> FragmentOutput {
    return FragmentOutput(in.color);
}
