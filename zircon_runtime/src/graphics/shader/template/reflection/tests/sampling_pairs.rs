use super::*;

const FILTERED_SAMPLE_WGSL: &str = r#"
@group(2) @binding(3) var sampled_texture: texture_2d<f32>;
@group(2) @binding(4) var sampled_sampler: sampler;

fn sample_through_helper(texture: texture_2d<f32>, texture_sampler: sampler) -> vec4<f32> {
    return textureSample(texture, texture_sampler, vec2<f32>(0.5));
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return sample_through_helper(sampled_texture, sampled_sampler);
}
"#;

const NON_SAMPLING_USE_WGSL: &str = r#"
@group(2) @binding(3) var sampled_texture: texture_2d<f32>;
@group(2) @binding(4) var sampled_sampler: sampler;

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    _ = sampled_sampler;
    return textureLoad(sampled_texture, vec2<i32>(0), 0);
}
"#;

#[test]
fn reflection_preserves_entry_sampling_pairs_through_function_arguments() {
    let reflection = reflect(FILTERED_SAMPLE_WGSL);
    let entry = reflection
        .entry_points
        .iter()
        .find(|entry| entry.name == "fs_main")
        .expect("fragment entry reflection");

    assert_eq!(
        entry.sampling_pairs,
        vec![ShaderSamplingPairIdentity {
            texture_group: 2,
            texture_binding: 3,
            sampler_group: 2,
            sampler_binding: 4,
        }]
    );
}

#[test]
fn sampling_pairs_participate_in_the_entry_resource_layout_hash() {
    let sampled = reflect(FILTERED_SAMPLE_WGSL);
    let loaded = reflect(NON_SAMPLING_USE_WGSL);
    let sampled_entry = sampled
        .entry_points
        .iter()
        .find(|entry| entry.name == "fs_main")
        .expect("sampled fragment entry");
    let loaded_entry = loaded
        .entry_points
        .iter()
        .find(|entry| entry.name == "fs_main")
        .expect("loaded fragment entry");

    assert_eq!(
        sampled_entry.resource_bindings,
        loaded_entry.resource_bindings
    );
    assert_ne!(
        sampled_entry.resource_layout_hash,
        loaded_entry.resource_layout_hash
    );
    assert!(loaded_entry.sampling_pairs.is_empty());
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
