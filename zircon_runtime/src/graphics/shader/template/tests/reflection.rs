use super::*;
use crate::graphics::shader::{ShaderTextureSampleType, ShaderTextureViewDimension};

const REFLECTION_WGSL: &str = r#"
struct Globals { value: vec4<f32> }
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var color_texture: texture_2d<f32>;
@group(0) @binding(2) var color_sampler: sampler;
@group(0) @binding(3) var<uniform> unused_globals: Globals;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(globals.value.xy, f32(vertex_index), 1.0);
    output.uv = vec2<f32>(0.5, 0.5);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return globals.value + textureSample(color_texture, color_sampler, input.uv);
}
"#;

#[test]
fn reflection_tracks_reachable_resources_and_merges_stage_visibility() {
    let reflection = reflect(REFLECTION_WGSL);

    assert_eq!(reflection.entry_points.len(), 2);
    assert_eq!(reflection.resource_bindings.len(), 3);
    let globals = resource(&reflection, 0);
    assert_eq!(
        globals.identity.resource_type,
        ShaderBindingResourceType::UniformBuffer
    );
    assert_eq!(globals.identity.min_binding_size, Some(16));
    assert!(globals.visibility.contains(naga::ShaderStage::Vertex));
    assert!(globals.visibility.contains(naga::ShaderStage::Fragment));
    let texture = resource(&reflection, 1);
    assert_eq!(
        texture.identity.resource_type,
        ShaderBindingResourceType::SampledTexture {
            view_dimension: ShaderTextureViewDimension::D2,
            sample_type: ShaderTextureSampleType::Float,
            multisampled: false,
        }
    );
    assert_eq!(texture.identity.min_binding_size, None);
    assert!(!texture.visibility.contains(naga::ShaderStage::Vertex));
    assert!(texture.visibility.contains(naga::ShaderStage::Fragment));
    assert_eq!(
        resource(&reflection, 2).identity.resource_type,
        ShaderBindingResourceType::Sampler { comparison: false }
    );
    assert!(reflection
        .resource_bindings
        .iter()
        .all(|resource| resource.identity.binding != 3));
}

#[test]
fn reflection_preserves_storage_access_depth_and_comparison_sampler_classes() {
    let reflection = reflect(
        r#"
@group(0) @binding(0) var<storage, read> source_values: array<u32>;
@group(0) @binding(1) var<storage, read_write> target_values: array<u32>;
@group(0) @binding(2) var shadow_texture: texture_depth_cube_array;
@group(0) @binding(3) var shadow_sampler: sampler_comparison;

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    target_values[0] = source_values[0];
    let visibility = textureSampleCompare(
        shadow_texture,
        shadow_sampler,
        vec3<f32>(0.0, 0.0, 1.0),
        0,
        0.5,
    );
    return vec4<f32>(visibility);
}
"#,
    );

    assert_eq!(
        resource(&reflection, 0).identity.resource_type,
        ShaderBindingResourceType::StorageBuffer { read_only: true }
    );
    assert_eq!(resource(&reflection, 0).identity.min_binding_size, Some(4));
    assert_eq!(
        resource(&reflection, 1).identity.resource_type,
        ShaderBindingResourceType::StorageBuffer { read_only: false }
    );
    assert_eq!(resource(&reflection, 1).identity.min_binding_size, Some(4));
    assert_eq!(
        resource(&reflection, 2).identity.resource_type,
        ShaderBindingResourceType::SampledTexture {
            view_dimension: ShaderTextureViewDimension::CubeArray,
            sample_type: ShaderTextureSampleType::Depth,
            multisampled: false,
        }
    );
    assert_eq!(
        resource(&reflection, 3).identity.resource_type,
        ShaderBindingResourceType::Sampler { comparison: true }
    );
}

#[test]
fn reflection_flattens_stage_io_bindings() {
    let reflection = reflect(REFLECTION_WGSL);
    let vertex = reflection
        .entry_points
        .iter()
        .find(|entry| entry.stage == naga::ShaderStage::Vertex)
        .expect("vertex reflection");
    let fragment = reflection
        .entry_points
        .iter()
        .find(|entry| entry.stage == naga::ShaderStage::Fragment)
        .expect("fragment reflection");

    assert_eq!(vertex.inputs.len(), 1);
    assert_eq!(vertex.outputs.len(), 2);
    assert_eq!(fragment.inputs.len(), 2);
    assert_eq!(fragment.outputs.len(), 1);
}

#[test]
fn reflection_keeps_same_stage_entry_resource_sets_independent() {
    let source = format!(
            "{REFLECTION_WGSL}\n@fragment\nfn fs_alt(input: VertexOutput) -> @location(0) vec4<f32> {{\n    return unused_globals.value + vec4<f32>(input.uv, 0.0, 0.0);\n}}"
        );
    let reflection = reflect(&source);
    let main = reflection
        .entry_points
        .iter()
        .find(|entry| entry.name == "fs_main")
        .expect("main fragment reflection");
    let alternative = reflection
        .entry_points
        .iter()
        .find(|entry| entry.name == "fs_alt")
        .expect("alternative fragment reflection");

    assert_eq!(
        main.resource_bindings
            .iter()
            .map(|resource| resource.binding)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    assert_eq!(
        alternative
            .resource_bindings
            .iter()
            .map(|resource| resource.binding)
            .collect::<Vec<_>>(),
        vec![3]
    );
    assert_ne!(main.resource_layout_hash, alternative.resource_layout_hash);
}

#[test]
fn reflection_hashes_ignore_resource_declaration_order_and_names() {
    let reordered = REFLECTION_WGSL
            .replace(
                "@group(0) @binding(0) var<uniform> globals: Globals;\n@group(0) @binding(1) var color_texture: texture_2d<f32>;",
                "@group(0) @binding(1) var renamed_texture: texture_2d<f32>;\n@group(0) @binding(0) var<uniform> renamed_globals: Globals;",
            )
            .replace("globals.value", "renamed_globals.value")
            .replace("color_texture", "renamed_texture");
    let original = reflect(REFLECTION_WGSL);
    let reordered = reflect(&reordered);

    assert_eq!(
        original.interface_layout_hash,
        reordered.interface_layout_hash
    );
    assert_eq!(
        original.resource_layout_hash,
        reordered.resource_layout_hash
    );
}

#[test]
fn reflection_hashes_change_only_for_the_changed_abi_domain() {
    let original = reflect(REFLECTION_WGSL);
    let interface_changed = reflect(&REFLECTION_WGSL.replace("@location(0) uv", "@location(1) uv"));
    let resource_changed = reflect(&REFLECTION_WGSL.replace(
        "@group(0) @binding(2) var color_sampler",
        "@group(0) @binding(4) var color_sampler",
    ));

    assert_ne!(
        original.interface_layout_hash,
        interface_changed.interface_layout_hash
    );
    assert_eq!(
        original.resource_layout_hash,
        interface_changed.resource_layout_hash
    );
    assert_eq!(
        original.interface_layout_hash,
        resource_changed.interface_layout_hash
    );
    assert_ne!(
        original.resource_layout_hash,
        resource_changed.resource_layout_hash
    );
}

#[test]
fn reflection_resource_hash_includes_buffer_member_names() {
    let original = reflect(REFLECTION_WGSL);
    let renamed_source = REFLECTION_WGSL
        .replace("value: vec4<f32>", "renamed_value: vec4<f32>")
        .replace(".value", ".renamed_value");
    let renamed = reflect(&renamed_source);

    assert_eq!(
        original.interface_layout_hash,
        renamed.interface_layout_hash
    );
    assert_ne!(original.resource_layout_hash, renamed.resource_layout_hash);
}

#[test]
fn reflection_marks_workgroup_override_identity_as_needing_specialization() {
    let specialized = reflect(
        r#"
override workgroup_x: u32 = 8u;

@compute @workgroup_size(workgroup_x, 1, 1)
fn cs_main() {}
"#,
    );
    let literal = reflect(
        r#"
@compute @workgroup_size(8, 1, 1)
fn cs_main() {}
"#,
    );

    assert_eq!(specialized.pipeline_override_count, 1);
    assert!(specialized.interface_requires_specialization);
    assert!(!specialized.resource_layout_requires_specialization);
    assert_eq!(
        specialized.entry_points[0].workgroup_size_overrides,
        [true, false, false]
    );
    assert_eq!(literal.pipeline_override_count, 0);
    assert!(!literal.interface_requires_specialization);
    assert_ne!(
        specialized.interface_layout_hash,
        literal.interface_layout_hash
    );
}

#[test]
fn override_sized_resource_type_requires_specialization_but_is_not_publishable() {
    let module = naga::front::wgsl::parse_str(
        r#"
override value_count: u32 = 4u;
@group(0) @binding(0) var<storage, read> values: array<u32, value_count>;

@compute @workgroup_size(1, 1, 1)
fn cs_main() {
    let first = values[0];
}
"#,
    )
    .expect("override-sized WGSL parse");
    let (_, global) = module
        .global_variables
        .iter()
        .next()
        .expect("override-sized resource global");
    let type_layout =
        shader_type_layout_hash(&module, global.ty, &mut vec![None; module.types.len()]);
    assert!(type_layout.requires_specialization);

    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    assert!(
        validator.validate(&module).is_err(),
        "host-shareable shader resources must be creation-resolved before publication"
    );
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

fn resource(
    reflection: &ShaderTemplateReflection,
    binding: u32,
) -> &ShaderResourceBindingReflection {
    reflection
        .resource_bindings
        .iter()
        .find(|resource| resource.identity.group == 0 && resource.identity.binding == binding)
        .expect("resource binding reflection")
}
