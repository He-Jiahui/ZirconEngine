use super::*;

const STATUS: &str =
    "render_plan08_runtime_custom_id_staged_fallback_lookup_static_passed_cargo_deferred";
const PRODUCT_PASS_STATUS: &str =
    "render_plan08_build_tool_product_base_pass_acceptance_contract_python_passed_cargo_deferred";
const QUALITY_GEOMETRY_STATUS: &str =
    "render_plan08_build_tool_cache_quality_geometry_identity_contract_python_passed_cargo_deferred";
const DIMENSION_COMBINATION_STATUS: &str =
    "render_plan08_build_tool_cache_dimension_combination_contract_python_passed_cargo_deferred";
const CUSTOM_ID_COMBINATION_STATUS: &str =
    "render_plan08_build_tool_cache_custom_id_combination_contract_python_passed_cargo_deferred";
const SOURCE_LABEL_PROVENANCE_STATUS: &str =
    "render_plan08_build_tool_cache_source_label_provenance_contract_python_passed_cargo_deferred";
const WRITTEN_VARIANT_UNIQUENESS_STATUS: &str =
    "render_plan08_build_tool_written_variant_uniqueness_contract_python_passed_cargo_deferred";
const CACHE_METADATA_FIELD_TYPE_STATUS: &str =
    "render_plan08_build_tool_cache_metadata_field_type_contract_python_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_cache_artifact_contract_is_wired() {
    let build = read_repo("tools/zircon_build.py");
    let prewarm_report =
        read_repo("zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs");
    let prewarm_write_path =
        read_repo("zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs");
    let cache_contract = read_repo("tools/zircon_build_shader_prewarm_cache_artifacts.py");
    let written_variants_helper =
        read_repo("tools/zircon_build_shader_prewarm_written_variants.py");
    let build_prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let cache_contract_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_cache_contract.py");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "build helper validates staged shader cache artifacts after successful prewarm",
        &build,
        &[
            "validate_shader_prewarm_cache_artifact_contract",
            "config.shader_prewarm_cache_root",
            "report_path=config.shader_prewarm_report_path",
            "expected_pass_types=_PRODUCT_BASE_MESH_PASS_TYPES",
            "expected_quality_tiers=config.shader_quality_tiers",
            "expected_geometry_sources=config.shader_geometry_sources",
            "expected_geometry_source_ids=shader_geometry_source_id_specs(config)",
            "expected_shading_model_ids=shader_shading_model_id_specs(config)",
        ],
    );
    assert_contains_all(
        "rust report records exact written cache variants",
        &(prewarm_report + &prewarm_write_path),
        &[
            "pub written_variants: Vec<ShaderVariantPrewarmWrittenVariant>",
            "pub struct ShaderVariantPrewarmWrittenVariant",
            "pub fn record_written_cache_entry(",
            "cache_hash: cache_hash.into()",
            "canonical_string: canonical_string.into()",
            "report.record_written_cache_entry(",
            "disk_key.hash",
            "disk_key.canonical_string",
            "render_shader_variant_prewarm_custom_ids_survive_disk_lookup",
            "variant_key_for_custom_ids(4, 16)",
            "GeometrySourceId::new(geometry_source)",
            "ShadingModelId::new(shading_model)",
            ".contains(\"|geometry=4|\")",
            ".contains(\"|shading=16|\")",
            "ShaderVariantCacheDisk::new(&root).lookup(&disk_key)",
            "render_shader_variant_prewarm_custom_ids_hit_staged_fallback_root",
            "ShaderVariantCacheDisk::new(&runtime_root).lookup(&disk_key)",
            "ShaderVariantCacheDisk::with_fallback_roots(&runtime_root, [&staged_root])",
            "fallback cache lookup must not create or write the runtime cache root",
        ],
    );
    assert_contains_all(
        "cache artifact contract helper checks written report count against disk pairs",
        &cache_contract,
        &[
            "def validate_shader_prewarm_cache_artifact_contract(",
            "expected_pass_types: Sequence[str] = ()",
            "expected_quality_tiers: Sequence[str] = ()",
            "expected_geometry_sources: Sequence[str] = ()",
            "expected_geometry_source_ids: Sequence[str] = ()",
            "expected_shading_model_ids: Sequence[str] = ()",
            "written_count = _count_value(report, \"written\")",
            "reported_written_variants(report)",
            "validate_reported_variants",
            "_validate_expected_written_variant_dimensions",
            "_validate_expected_written_pass_types",
            "_validate_expected_written_quality_tiers",
            "_validate_expected_written_geometry_sources",
            "_validate_expected_written_variant_combinations",
            "_validate_expected_written_custom_id_combinations",
            "_custom_id_combination_matches",
            "_custom_id_combination_label",
            "validate_written_variant_source_labels",
            "_validate_expected_written_dimension",
            "_expected_shader_id_records",
            "parse_shader_id_record",
            "_canonical_has_dimension_value",
            "_canonical_dimension_values",
            "canonical_field=\"pass\"",
            "canonical_field=\"quality\"",
            "canonical_field=\"geometry\"",
            "canonical_field=\"shading\"",
            "missing requested shader pass types",
            "missing requested shader quality tiers",
            "missing requested shader geometry sources",
            "missing requested shader variant combinations",
            "missing requested shader custom id combinations",
            "written variant source labels missing source provenance",
            "missing requested {label} ids",
            "def _geometry_source_dimension_id(",
            "_SHADER_VARIANT_CACHE_SCHEMA_VERSION = 1",
            "validate_cache_hash_shape",
            "cache hash shape mismatch",
            "_validate_runtime_cache_layout",
            "_runtime_cache_wgsl_artifact_path",
            "runtime cache layout",
            "cache metadata schema mismatch",
            "invalid field types: schema_version",
            "created_unix_seconds",
            "isinstance(metadata.get(field), bool)",
            "missing reported cache variants",
            "reported cache variant mismatch",
            "cache_hash",
            "_shader_cache_artifact_pairs",
            "def _validate_cache_metadata(",
            "def _hash_from_wgsl_artifact(",
            "missing metadata",
            "invalid cache metadata",
            "cache metadata hash mismatch",
            "cache artifacts do not cover written variants",
            "canonical_string",
            "schema_version",
            ".wgsl.zst",
            ".meta",
        ],
    );
    assert_contains_all(
        "shared written variant helper owns cache identity parsing and uniqueness",
        &written_variants_helper,
        &[
            "class ReportedWrittenVariant",
            "def reported_written_variants(",
            "def validate_unique_written_variant_identity(",
            "def validate_cache_hash_shape(",
            "def validate_written_variant_source_labels(",
            "def _source_provenance_labels(",
            "_BLAKE3_HEX_LENGTH = 64",
            "source_label: str | None",
            "duplicate written cache variant identity",
            "cache_hash=",
            "canonical_string=",
        ],
    );
    assert_contains_all(
        "python regressions cover artifact contract and build handoff",
        &(build_prewarm_tests + &cache_contract_tests),
        &[
            "test_prewarm_shaders_validates_wgpu_report_after_success",
            "test_validate_cache_artifact_contract_requires_written_cache_pairs",
            "test_validate_cache_artifact_contract_rejects_orphan_wgsl_artifacts",
            "test_validate_cache_artifact_contract_rejects_invalid_metadata",
            "test_validate_cache_artifact_contract_rejects_invalid_metadata_field_types",
            "test_validate_cache_artifact_contract_rejects_metadata_hash_mismatch",
            "test_validate_cache_artifact_contract_requires_report_written_variants",
            "test_validate_cache_artifact_contract_rejects_partial_written_variant_report",
            "test_validate_cache_artifact_contract_rejects_wrong_canonical_variant",
            "test_validate_cache_artifact_contract_rejects_non_runtime_cache_layout",
            "test_validate_cache_artifact_contract_rejects_schema_version_mismatch",
            "test_validate_cache_artifact_contract_rejects_non_blake3_hex_cache_hash",
            "test_validate_cache_artifact_contract_requires_requested_pass_types",
            "test_validate_cache_artifact_contract_accepts_requested_pass_types",
            "test_validate_cache_artifact_contract_requires_requested_quality_tiers",
            "test_validate_cache_artifact_contract_requires_requested_geometry_sources",
            "test_validate_cache_artifact_contract_accepts_requested_quality_and_geometry",
            "test_validate_cache_artifact_contract_requires_requested_dimension_combinations",
            "test_validate_cache_artifact_contract_requires_requested_custom_id_combinations",
            "test_validate_cache_artifact_contract_requires_written_variant_source_labels_in_provenance",
            "test_validate_cache_artifact_contract_rejects_duplicate_written_variant_identity",
            "test_validate_cache_artifact_contract_requires_requested_custom_ids",
            "test_validate_cache_artifact_contract_requires_requested_shading_ids",
            "test_validate_cache_artifact_contract_accepts_requested_custom_ids",
            "test_validate_cache_artifact_contract_accepts_written_cache_pairs",
        ],
    );

    for (path, source) in [
        (
            "tools/zircon_build_shader_prewarm_cache_artifacts.py",
            cache_contract.as_str(),
        ),
        (
            "tools/zircon_build_shader_prewarm_written_variants.py",
            written_variants_helper.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm_cache_contract.py",
            cache_contract_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_cache_artifact_contract.rs",
            include_str!("shader_prewarm_cache_artifact_contract.rs"),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("build tool doc", build_tool_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("session note", session.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Build-tool shader prewarm cache artifact contract",
                "Prewarm report cache identity contract",
                "Prewarm cache runtime layout contract",
                "Prewarm cache hash shape contract",
                "Prewarm cache custom id correlation contract",
                "Runtime prewarm custom id cache lookup contract",
                "Runtime custom id staged fallback lookup contract",
                STATUS,
                "Build-tool product Base pass acceptance contract",
                PRODUCT_PASS_STATUS,
                "Build-tool cache quality/geometry identity contract",
                QUALITY_GEOMETRY_STATUS,
                "Build-tool cache dimension combination contract",
                DIMENSION_COMBINATION_STATUS,
                "Build-tool cache custom id combination contract",
                CUSTOM_ID_COMBINATION_STATUS,
                "Build-tool cache source-label provenance correlation contract",
                SOURCE_LABEL_PROVENANCE_STATUS,
                "Build-tool written variant uniqueness contract",
                WRITTEN_VARIANT_UNIQUENESS_STATUS,
                "Build-tool cache metadata field type contract",
                CACHE_METADATA_FIELD_TYPE_STATUS,
                "render_shader_variant_prewarm_custom_ids_survive_disk_lookup",
                "render_shader_variant_prewarm_custom_ids_hit_staged_fallback_root",
                "test_validate_cache_artifact_contract_requires_written_cache_pairs",
                "test_validate_cache_artifact_contract_rejects_invalid_metadata_field_types",
                "test_validate_cache_artifact_contract_rejects_wrong_canonical_variant",
                "test_validate_cache_artifact_contract_rejects_non_runtime_cache_layout",
                "test_validate_cache_artifact_contract_rejects_non_blake3_hex_cache_hash",
                "test_validate_cache_artifact_contract_requires_requested_pass_types",
                "test_validate_cache_artifact_contract_accepts_requested_pass_types",
                "test_validate_cache_artifact_contract_requires_requested_quality_tiers",
                "test_validate_cache_artifact_contract_requires_requested_geometry_sources",
                "test_validate_cache_artifact_contract_accepts_requested_quality_and_geometry",
                "test_validate_cache_artifact_contract_requires_requested_dimension_combinations",
                "test_validate_cache_artifact_contract_requires_requested_custom_id_combinations",
                "test_validate_cache_artifact_contract_requires_written_variant_source_labels_in_provenance",
                "test_validate_cache_artifact_contract_rejects_duplicate_written_variant_identity",
                "test_acceptance_contract_rejects_duplicate_written_variant_identity",
                "tools/zircon_build_shader_prewarm_written_variants.py",
                "duplicate written cache variant identity",
                "test_validate_cache_artifact_contract_requires_requested_custom_ids",
                "test_validate_cache_artifact_contract_requires_requested_shading_ids",
                "test_validate_cache_artifact_contract_accepts_requested_custom_ids",
                "runtime_15_shader_prewarm_cache_artifact_contract_is_wired",
            ],
        );
    }
}
