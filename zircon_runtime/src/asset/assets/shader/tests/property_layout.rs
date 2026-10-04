use std::collections::BTreeMap;

use super::*;

#[test]
fn render_shader_property_packing_backfills_vec3_w_with_scalar() {
    let properties = vec![
        property("base_color", MaterialPropertyKind::Color),
        property("normal", MaterialPropertyKind::Vec3),
        property("roughness", MaterialPropertyKind::Float),
        property("uv_scale", MaterialPropertyKind::Vec2),
        property("flags", MaterialPropertyKind::UInt),
        property("enabled", MaterialPropertyKind::Bool),
    ];

    let artifact = generate_material_artifact(&properties, &[], &[]);

    assert_eq!(artifact.property_layout.f32_slot_count, 3);
    assert_eq!(artifact.property_layout.u32_slot_count, 1);
    assert_eq!(artifact.property_layout.packed_size, 64);
    assert_slot(&artifact.property_layout, "base_color", 0, 0, 4);
    assert_slot(&artifact.property_layout, "normal", 1, 0, 3);
    assert_slot(&artifact.property_layout, "roughness", 1, 3, 1);
    assert_slot(&artifact.property_layout, "uv_scale", 2, 0, 2);
    assert_slot(&artifact.property_layout, "flags", 0, 0, 1);
    assert_slot(&artifact.property_layout, "enabled", 0, 1, 1);
}

#[test]
fn render_shader_property_layout_hash_is_deterministic() {
    let properties = vec![
        property("gain", MaterialPropertyKind::Float),
        property("tint", MaterialPropertyKind::Vec4),
    ];

    let first = generate_material_artifact(&properties, &[], &[]);
    let second = generate_material_artifact(&properties, &[], &[]);

    assert_ne!(first.property_layout.layout_hash, 0);
    assert_eq!(
        first.property_layout.layout_hash,
        second.property_layout.layout_hash
    );
}

#[test]
fn render_shader_property_layout_hash_includes_texture_dimension() {
    let two_dimensional =
        generate_material_artifact(&[], &[], &[texture_slot("input", "texture_2d", false)]);
    let cube =
        generate_material_artifact(&[], &[], &[texture_slot("input", "texture_cube", false)]);

    assert_ne!(
        two_dimensional.property_layout.layout_hash,
        cube.property_layout.layout_hash
    );
}

#[test]
#[cfg(feature = "graphics")]
fn render_shader_generated_module_naga_accepts_full_type_surface_layout() {
    let properties = vec![
        property("base_color", MaterialPropertyKind::Color),
        property("roughness", MaterialPropertyKind::Float),
        property("uv_scale", MaterialPropertyKind::Vec2),
        property("normal", MaterialPropertyKind::Vec3),
        property("layer", MaterialPropertyKind::Int),
        property("flags", MaterialPropertyKind::UInt),
        property("enabled", MaterialPropertyKind::Bool),
    ];
    let texture_slots = vec![
        texture_slot("profile", "texture_1d", false),
        texture_slot("base_color", "texture_2d", true),
        texture_slot("decals", "texture_2d_array", false),
        texture_slot("environment", "texture_cube", false),
        texture_slot("environment_probes", "texture_cube_array", false),
        texture_slot("volume", "texture_3d", false),
    ];

    let artifact = generate_material_artifact(&properties, &[], &texture_slots);

    assert!(artifact.wgsl_source.contains("fn zr_mat_base_color_st()"));
    assert!(artifact.wgsl_source.contains("fn zr_uv_base_color"));
    assert!(artifact
        .wgsl_source
        .contains("fn zr_sample_profile(coord: f32)"));
    assert!(artifact
        .wgsl_source
        .contains("fn zr_sample_base_color(uv: vec2<f32>)"));
    assert!(artifact
        .wgsl_source
        .contains("fn zr_sample_decals(uv: vec2<f32>, layer: i32)"));
    assert!(artifact
        .wgsl_source
        .contains("fn zr_sample_environment(direction: vec3<f32>)"));
    assert!(artifact.wgsl_source.contains("texture_cube_array<f32>"));
    assert!(artifact
        .wgsl_source
        .contains("fn zr_sample_environment_probes(direction: vec3<f32>, layer: i32)"));
    assert!(artifact
        .wgsl_source
        .contains("fn zr_sample_volume(position: vec3<f32>)"));
    naga::front::wgsl::parse_str(&artifact.wgsl_source).unwrap();
}

#[test]
fn render_shader_option_table_packs_bool_and_enum_bits() {
    let options = vec![
        option("detail_layer", "bool", Some(toml::Value::Boolean(true)), ""),
        option(
            "detail_mode",
            "enum",
            Some(toml::Value::String("triplanar".to_string())),
            "off,uv,triplanar",
        ),
    ];

    let artifact = generate_material_artifact(&[], &options, &[]);

    assert_eq!(artifact.option_table.options[0].bit_offset, 0);
    assert_eq!(artifact.option_table.options[0].bit_width, 1);
    assert_eq!(artifact.option_table.options[0].default_bits, 1);
    assert_eq!(artifact.option_table.options[1].bit_offset, 1);
    assert_eq!(artifact.option_table.options[1].bit_width, 2);
    assert_eq!(artifact.option_table.options[1].default_bits, 2);
    assert_eq!(artifact.option_table.total_bits, 3);
}

#[test]
fn borrowed_shader_token_contract_property() {
    let bool_option = option("enabled", "  BoOlEaN ", None, "");
    let enum_option = option("mode", " EnUm ", None, "off,quality");
    let unknown_option = option("fallback", " custom ", None, "");

    assert_eq!(
        option_kind_and_values(&bool_option),
        (MaterialOptionKind::Bool, Vec::new())
    );
    assert_eq!(
        option_kind_and_values(&enum_option),
        (
            MaterialOptionKind::Enum,
            vec!["off".to_string(), "quality".to_string()]
        )
    );
    assert_eq!(
        option_kind_and_values(&unknown_option),
        (MaterialOptionKind::Bool, Vec::new()),
        "unknown option kinds retain the bool fallback"
    );
}

#[test]
#[ignore = "release performance gate"]
fn borrowed_shader_token_performance_release_property() {
    use std::hint::black_box;
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 21;
    const LOOKUPS_PER_SAMPLE: usize = 80_000;

    fn legacy_sample() -> u128 {
        let started = Instant::now();
        let mut matched = 0_u64;
        for _ in 0..LOOKUPS_PER_SAMPLE {
            for token in [" bool ", "BOOLEAN", " Enum ", "custom"] {
                let kind = match black_box(token).trim().to_ascii_lowercase().as_str() {
                    "enum" => MaterialOptionKind::Enum,
                    _ => MaterialOptionKind::Bool,
                };
                matched += u64::from(kind == MaterialOptionKind::Enum);
            }
        }
        black_box(matched);
        started.elapsed().as_nanos()
    }

    fn borrowed_sample() -> u128 {
        let started = Instant::now();
        let mut matched = 0_u64;
        for _ in 0..LOOKUPS_PER_SAMPLE {
            for token in [" bool ", "BOOLEAN", " Enum ", "custom"] {
                let kind = material_option_kind_from_token(black_box(token));
                matched += u64::from(kind == MaterialOptionKind::Enum);
            }
        }
        black_box(matched);
        started.elapsed().as_nanos()
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
            (legacy_sample(), borrowed_sample())
        } else {
            let optimized_ns = borrowed_sample();
            (legacy_sample(), optimized_ns)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
    }

    let nearest_rank_p95 = |samples: &[u128]| {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    };
    let csv = |samples: &[u128]| {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    };
    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_material_option_token_dispatch sample_pairs={SAMPLE_PAIRS} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25 legacy_allocations_per_sample={} optimized_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
        csv(&legacy_samples),
        csv(&optimized_samples),
        LOOKUPS_PER_SAMPLE * 4,
    );
    assert!(
        improvement_percent >= 25,
        "borrowed material option matching must improve P95 by at least 25%"
    );
}

fn property(name: &str, kind: MaterialPropertyKind) -> ShaderMaterialPropertyAsset {
    ShaderMaterialPropertyAsset {
        name: name.to_string(),
        kind,
        required: false,
        default: None,
        editor: BTreeMap::new(),
    }
}

fn option(
    name: &str,
    kind: &str,
    default: Option<toml::Value>,
    enum_values: &str,
) -> ShaderOptionAsset {
    let mut editor = BTreeMap::new();
    if !enum_values.is_empty() {
        editor.insert("values".to_string(), enum_values.to_string());
    }
    ShaderOptionAsset {
        name: name.to_string(),
        kind: kind.to_string(),
        default,
        editor,
    }
}

fn texture_slot(name: &str, kind: &str, st: bool) -> ShaderTextureSlotAsset {
    ShaderTextureSlotAsset {
        name: name.to_string(),
        kind: kind.to_string(),
        required: false,
        default: None,
        sampler: None,
        group: None,
        label: None,
        option: None,
        st,
        editor: BTreeMap::new(),
    }
}

fn assert_slot(
    layout: &MaterialPropertyLayout,
    name: &str,
    slot: u16,
    component: u8,
    component_count: u8,
) {
    let property = layout
        .properties
        .iter()
        .find(|property| property.name == name)
        .unwrap();
    assert_eq!(property.slot, slot);
    assert_eq!(property.component, component);
    assert_eq!(property.component_count, component_count);
}
