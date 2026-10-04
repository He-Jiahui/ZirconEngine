use super::*;

#[test]
fn stage_interface_accepts_downstream_vector_subtypes_and_unused_vertex_outputs() {
    let reflection = reflect(
        r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) unused_id: u32,
}

@vertex
fn vs_main() -> VertexOutput {
    return VertexOutput(vec4<f32>(0.0), vec4<f32>(1.0), 7u);
}

@fragment
fn fs_main(@location(0) color: vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(color, 1.0);
}
"#,
    );

    reflection
        .validate_vertex_fragment_stage_interface("vs_main", "fs_main")
        .expect("a narrower fragment input and unused vertex outputs are legal");
}

#[test]
fn stage_interface_rejects_missing_vertex_output_location() {
    let reflection = reflect(
        r#"
@vertex
fn vs_main() -> @builtin(position) vec4<f32> {
    return vec4<f32>(0.0);
}

@fragment
fn fs_main(@location(2) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}
"#,
    );

    let error = reflection
        .validate_vertex_fragment_stage_interface("vs_main", "fs_main")
        .expect_err("the fragment input has no matching vertex output");

    assert!(error.contains("@location(2)"), "unexpected error: {error}");
}

#[test]
fn stage_interface_rejects_incompatible_numeric_types() {
    let reflection = reflect(
        r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) value: vec4<f32>,
}

@vertex
fn vs_main() -> VertexOutput {
    return VertexOutput(vec4<f32>(0.0), vec4<f32>(1.0));
}

@fragment
fn fs_main(
    @location(0) @interpolate(flat) value: vec3<u32>,
) -> @location(0) vec4<f32> {
    _ = value;
    return vec4<f32>(1.0);
}
"#,
    );

    let error = reflection
        .validate_vertex_fragment_stage_interface("vs_main", "fs_main")
        .expect_err("the scalar kinds are incompatible");

    assert!(error.contains("numeric type"), "unexpected error: {error}");
}

#[test]
fn stage_interface_rejects_interpolation_mismatch() {
    let reflection = reflect(
        r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) color: vec4<f32>,
}

@vertex
fn vs_main() -> VertexOutput {
    return VertexOutput(vec4<f32>(0.0), vec4<f32>(1.0));
}

@fragment
fn fs_main(
    @location(0) @interpolate(perspective) color: vec4<f32>,
) -> @location(0) vec4<f32> {
    return color;
}
"#,
    );

    let error = reflection
        .validate_vertex_fragment_stage_interface("vs_main", "fs_main")
        .expect_err("the interpolation modes are incompatible");

    assert!(error.contains("interpolation"), "unexpected error: {error}");
}

#[test]
fn stage_interface_rejects_sampling_mismatch() {
    let reflection = reflect(
        r#"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(perspective, centroid) color: vec4<f32>,
}

@vertex
fn vs_main() -> VertexOutput {
    return VertexOutput(vec4<f32>(0.0), vec4<f32>(1.0));
}

@fragment
fn fs_main(
    @location(0) @interpolate(perspective, center) color: vec4<f32>,
) -> @location(0) vec4<f32> {
    return color;
}
"#,
    );

    let error = reflection
        .validate_vertex_fragment_stage_interface("vs_main", "fs_main")
        .expect_err("the sampling modes are incompatible");

    assert!(error.contains("sampling"), "unexpected error: {error}");
}

#[test]
fn vertex_input_interface_matches_wgpu_scalar_kind_semantics() {
    let reflection = reflect(
        r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) joints: vec4<u32>,
}

@vertex
fn vs_main(input: VertexInput) -> @builtin(position) vec4<f32> {
    _ = input.joints;
    return vec4<f32>(input.position, 1.0);
}
"#,
    );

    reflection
        .validate_vertex_input_stage_interface("vs_main", |location| match location {
            // WGPU validates only the scalar kind for vertex attributes. A narrower
            // provided vector is legal because the driver supplies default components.
            0 => Some(ShaderVertexInputScalarKind::Float),
            1 => Some(ShaderVertexInputScalarKind::Uint),
            _ => None,
        })
        .expect("the production vertex declaration provides both scalar kinds");
}

#[test]
fn vertex_input_interface_rejects_missing_location() {
    let reflection = reflect(
        r#"
@vertex
fn vs_main(@location(3) joints: vec4<u32>) -> @builtin(position) vec4<f32> {
    _ = joints;
    return vec4<f32>(0.0);
}
"#,
    );

    let error = reflection
        .validate_vertex_input_stage_interface("vs_main", |_| None)
        .expect_err("the vertex declaration does not provide location 3");

    assert!(error.contains("@location(3)"), "unexpected error: {error}");
    assert!(error.contains("not provided"), "unexpected error: {error}");
}

#[test]
fn vertex_input_interface_rejects_wrong_scalar_kind() {
    let reflection = reflect(
        r#"
@vertex
fn vs_main(@location(3) joints: vec4<u32>) -> @builtin(position) vec4<f32> {
    _ = joints;
    return vec4<f32>(0.0);
}
"#,
    );

    let error = reflection
        .validate_vertex_input_stage_interface("vs_main", |_| {
            Some(ShaderVertexInputScalarKind::Float)
        })
        .expect_err("the shader requires uint but the vertex declaration provides float");

    assert!(error.contains("@location(3)"), "unexpected error: {error}");
    assert!(error.contains("Uint"), "unexpected error: {error}");
    assert!(error.contains("Float"), "unexpected error: {error}");
}

#[test]
fn fragment_output_interface_accepts_shader_vectors_wider_than_the_target() {
    let reflection = reflect(
        r#"
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0);
}
"#,
    );

    reflection
        .validate_fragment_output_stage_interface("fs_main", |location| {
            (location == 0).then(|| {
                ShaderFragmentOutputNumericType::new(ShaderFragmentOutputScalarKind::Float, 4, 2)
                    .expect("two-component target type")
            })
        })
        .expect("an RG float target is covered by a vec4 float shader output");
}

#[test]
fn fragment_output_interface_rejects_incompatible_target_scalar_kind() {
    let reflection = reflect(
        r#"
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0);
}
"#,
    );

    let error = reflection
        .validate_fragment_output_stage_interface("fs_main", |location| {
            (location == 0).then(|| {
                ShaderFragmentOutputNumericType::new(ShaderFragmentOutputScalarKind::Uint, 4, 1)
                    .expect("single-component uint target type")
            })
        })
        .expect_err("a uint target is not covered by a float shader output");

    assert!(error.contains("@location(0)"), "unexpected error: {error}");
    assert!(error.contains("Uint"), "unexpected error: {error}");
    assert!(error.contains("Float"), "unexpected error: {error}");
}

#[test]
fn fragment_output_interface_ignores_unpaired_shader_outputs_and_targets() {
    let reflection = reflect(
        r#"
struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(2) auxiliary: vec2<f32>,
}

@fragment
fn fs_main() -> FragmentOutput {
    return FragmentOutput(vec4<f32>(1.0), vec2<f32>(0.0));
}
"#,
    );

    reflection
        .validate_fragment_output_stage_interface("fs_main", |location| {
            (location == 1).then(|| {
                ShaderFragmentOutputNumericType::new(ShaderFragmentOutputScalarKind::Float, 4, 4)
                    .expect("four-component float target type")
            })
        })
        .expect("WGPU ignores outputs without targets and targets without outputs");
}

fn reflect(source: &str) -> ShaderTemplateReflection {
    let module = naga::front::wgsl::parse_str(source).expect("WGSL parse");
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    let module_info = validator.validate(&module).expect("WGSL validation");
    reflect_validated_shader_module(&module, &module_info)
}
