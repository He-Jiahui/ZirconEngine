use std::fs;
use std::path::Path;

use crate::core::framework::render::{
    RenderStats, ShaderPassType, ShaderVariantMissReport, ShaderVariantPrewarmDimensionCount,
    ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport, ShaderVariantPrewarmRequest,
    ShaderVariantRuntimeDimensionCount, ShadingModelId, GEOMETRY_SOURCE_ID_STATIC_MESH,
    SHADING_MODEL_ID_STANDARD_PBR,
};

use super::case::RegistryShaderCase;
use super::manifest::{raw_wgsl_hash, REGISTRY_MATERIAL_PASS_TYPES};

pub(super) fn assert_registry_material_pass_prewarm_written(
    manifest: &ShaderVariantPrewarmManifest,
    report: &ShaderVariantPrewarmReport,
    case: RegistryShaderCase,
) {
    assert_registry_material_pass_prewarm_written_for_shading_model(
        manifest,
        report,
        case,
        SHADING_MODEL_ID_STANDARD_PBR,
    );
}

pub(super) fn assert_registry_material_pass_prewarm_written_for_shading_model(
    manifest: &ShaderVariantPrewarmManifest,
    report: &ShaderVariantPrewarmReport,
    case: RegistryShaderCase,
    expected_shading_model: ShadingModelId,
) {
    for pass_type in REGISTRY_MATERIAL_PASS_TYPES {
        let label = case.source_label_for_pass(pass_type);
        let request = manifest
            .variants
            .iter()
            .find(|request| request.source_label.as_str() == label.as_str())
            .unwrap_or_else(|| panic!("registry material-pass prewarm request for {label}"));
        assert_registry_material_pass_request_key(request, case, pass_type, expected_shading_model);
        let written = report
            .written_variants
            .iter()
            .find(|variant| variant.source_label.as_str() == label.as_str())
            .unwrap_or_else(|| panic!("registry material-pass written variant for {label}"));
        assert!(
            written
                .canonical_string
                .contains(&case.shader_id().to_string()),
            "written cache key should include registry shader id for {label}; canonical={}",
            written.canonical_string
        );
        assert!(
            written
                .canonical_string
                .contains(&format!("|revision={}", case.revision)),
            "written cache key should include registry revision for {label}; canonical={}",
            written.canonical_string
        );
        assert!(
            written
                .canonical_string
                .contains(&format!("|shading={}|", expected_shading_model.value())),
            "written cache key should include shading model {} for {label}; canonical={}",
            expected_shading_model.value(),
            written.canonical_string
        );
    }
}

fn assert_registry_material_pass_request_key(
    request: &ShaderVariantPrewarmRequest,
    case: RegistryShaderCase,
    pass_type: ShaderPassType,
    expected_shading_model: ShadingModelId,
) {
    assert_eq!(request.key.material_shader, case.shader_id());
    assert_eq!(request.key.material_revision, case.revision);
    assert_eq!(request.key.pass_type, pass_type);
    assert_eq!(request.key.shading_model, expected_shading_model);
    assert_eq!(request.source_label, case.source_label_for_pass(pass_type));
    assert!(
        request
            .include_content_hashes
            .contains(&raw_wgsl_hash(&request.wgsl_source)),
        "template request should retain the final WGSL source hash for {}",
        request.source_label
    );
}

pub(super) fn assert_registry_material_pass_first_frame_shader_cache_hit(
    stats: &RenderStats,
    case: RegistryShaderCase,
    prewarm_report: &ShaderVariantPrewarmReport,
) {
    assert_registry_material_pass_first_frame_shader_cache_hit_for_shading_model(
        stats,
        case,
        prewarm_report,
        SHADING_MODEL_ID_STANDARD_PBR,
        "StandardPBR shading model",
    );
}

pub(super) fn assert_registry_material_pass_first_frame_shader_cache_hit_for_shading_model(
    stats: &RenderStats,
    case: RegistryShaderCase,
    prewarm_report: &ShaderVariantPrewarmReport,
    expected_shading_model: ShadingModelId,
    expected_shading_model_label: &str,
) {
    let report = &stats.last_shader_variant_miss_report;
    assert!(
        stats.last_mesh_opaque_draw_count >= 1,
        "first frame should draw the registry material mesh for {}; stats={stats:?}",
        case.locator
    );
    assert!(
        stats.last_mesh_shadow_caster_draw_count >= 1,
        "first frame should draw the registry material shadow caster for {}; stats={stats:?}",
        case.locator
    );
    assert_eq!(
        stats.last_mesh_taa_reactive_mask_command_count, 1,
        "first frame should build the TAA reactive registry material mask for {}; stats={stats:?}",
        case.locator
    );
    assert_executed_registry_material_pass_executors(stats, case);
    assert!(
        report.request_count >= 4,
        "first frame should request material pass variants for {}; report={report:?}",
        case.locator
    );
    assert_eq!(
        report.compile_miss_count, 0,
        "first frame must not compile-miss registry material pass variants for {}; report={report:?}",
        case.locator
    );
    assert_eq!(report.disk_write_count, 0);
    assert_eq!(report.disk_error_count, 0);
    assert_runtime_dimension_disk_hit(
        report.dimension_summary.pass_types.get("depth_prepass"),
        case,
        "depth-prepass pass",
    );
    assert_runtime_dimension_disk_hit(
        report.dimension_summary.pass_types.get("gbuffer"),
        case,
        "gbuffer pass",
    );
    assert_runtime_dimension_requested_without_miss(
        report.dimension_summary.pass_types.get("shadow"),
        case,
        "shadow pass",
    );
    assert_runtime_dimension_requested_without_miss(
        report.dimension_summary.pass_types.get("taa_reactive_mask"),
        case,
        "TAA reactive mask pass",
    );
    assert_registry_material_pass_runtime_dimensions(
        prewarm_report,
        report,
        case,
        true,
        expected_shading_model,
        expected_shading_model_label,
    );
}

pub(super) fn assert_registry_material_pass_velocity_frame_shader_cache_hit(
    stats: &RenderStats,
    case: RegistryShaderCase,
    prewarm_report: &ShaderVariantPrewarmReport,
) {
    assert_registry_material_pass_velocity_frame_shader_cache_hit_for_shading_model(
        stats,
        case,
        prewarm_report,
        SHADING_MODEL_ID_STANDARD_PBR,
        "StandardPBR shading model",
    );
}

pub(super) fn assert_registry_material_pass_velocity_frame_shader_cache_hit_for_shading_model(
    stats: &RenderStats,
    case: RegistryShaderCase,
    prewarm_report: &ShaderVariantPrewarmReport,
    expected_shading_model: ShadingModelId,
    expected_shading_model_label: &str,
) {
    let report = &stats.last_shader_variant_miss_report;
    assert!(
        stats.last_mesh_opaque_draw_count >= 1,
        "velocity frame should continue drawing the registry material mesh for {}; stats={stats:?}",
        case.locator
    );
    assert_eq!(
        stats.last_mesh_taa_reactive_mask_command_count, 1,
        "velocity frame should build the TAA reactive registry material mask for {}; stats={stats:?}",
        case.locator
    );
    assert!(
        stats.last_mesh_previous_velocity_transform_draw_count >= 1,
        "velocity frame should use a previous velocity transform for the registry material mesh on {}; stats={stats:?}",
        case.locator
    );
    assert_eq!(
        stats.last_mesh_missing_velocity_transform_draw_count, 0,
        "velocity frame should not miss previous transforms for the registry material mesh on {}; stats={stats:?}",
        case.locator
    );
    assert_executed_registry_material_pass_executors(stats, case);
    assert_registry_material_pass_velocity_executor(stats, case);
    assert!(
        report.request_count >= 5,
        "velocity frame should request repeat material pass variants for {}; report={report:?}",
        case.locator
    );
    assert_eq!(
        report.compile_miss_count, 0,
        "velocity frame must not compile-miss registry material pass variants for {}; report={report:?}",
        case.locator
    );
    assert_eq!(report.disk_write_count, 0);
    assert_eq!(report.disk_error_count, 0);
    assert_runtime_dimension_requested_without_miss(
        report.dimension_summary.pass_types.get("shadow"),
        case,
        "shadow pass",
    );
    assert_runtime_dimension_requested_without_miss(
        report.dimension_summary.pass_types.get("taa_reactive_mask"),
        case,
        "TAA reactive mask pass",
    );
    assert_runtime_dimension_requested_without_miss(
        report.dimension_summary.pass_types.get("velocity"),
        case,
        "velocity pass",
    );
    assert_registry_material_pass_runtime_dimensions(
        prewarm_report,
        report,
        case,
        false,
        expected_shading_model,
        expected_shading_model_label,
    );
}

fn assert_executed_registry_material_pass_executors(stats: &RenderStats, case: RegistryShaderCase) {
    for executor_id in [
        "deferred.depth-prepass",
        "deferred.gbuffer",
        "shadow.atlas",
        "lighting.light-grid",
        "lighting.deferred",
        "post.output-transfer",
    ] {
        assert!(
            stats
                .last_graph_executed_executor_ids
                .iter()
                .any(|executor| executor == executor_id),
            "registry material pass should execute `{executor_id}` for {}; executed={:?}",
            case.locator,
            stats.last_graph_executed_executor_ids
        );
    }
}

fn assert_registry_material_pass_velocity_executor(stats: &RenderStats, case: RegistryShaderCase) {
    assert!(
        stats
            .last_graph_executed_executor_ids
            .iter()
        .any(|executor| executor == "temporal.velocity-object"),
        "registry material pass should execute `temporal.velocity-object` for {}; executed={:?}; anti_alias={:?}; post_process_nodes={:?}",
        case.locator,
        stats.last_graph_executed_executor_ids,
        stats.last_anti_alias_fallback,
        stats.last_post_process_graph_executed_nodes
    );
}

pub(super) fn assert_registry_material_pass_prewarm_dimensions_written(
    report: &ShaderVariantPrewarmReport,
) {
    assert_registry_material_pass_prewarm_dimensions_written_for_shading_model(
        report,
        SHADING_MODEL_ID_STANDARD_PBR,
        "prewarm StandardPBR shading model",
    );
}

pub(super) fn assert_registry_material_pass_prewarm_dimensions_written_for_shading_model(
    report: &ShaderVariantPrewarmReport,
    expected_shading_model: ShadingModelId,
    expected_shading_model_label: &str,
) {
    let static_geometry = GEOMETRY_SOURCE_ID_STATIC_MESH.value().to_string();
    let shading_model = expected_shading_model.value().to_string();

    for pass_type in REGISTRY_MATERIAL_PASS_TYPES {
        assert_prewarm_dimension_written(
            report.dimension_summary.pass_types.get(pass_type.token()),
            &format!("prewarm {} pass", pass_type.token()),
        );
    }
    assert_prewarm_dimension_written(
        report
            .dimension_summary
            .geometry_source_ids
            .get(&static_geometry),
        "prewarm static geometry source",
    );
    assert_prewarm_dimension_written(
        report
            .dimension_summary
            .shading_model_ids
            .get(&shading_model),
        expected_shading_model_label,
    );
    assert_prewarm_dimension_written(
        report.dimension_summary.quality_tiers.get("medium"),
        "prewarm medium quality tier",
    );
}

fn assert_registry_material_pass_runtime_dimensions(
    prewarm_report: &ShaderVariantPrewarmReport,
    runtime_report: &ShaderVariantMissReport,
    case: RegistryShaderCase,
    require_disk_hit: bool,
    expected_shading_model: ShadingModelId,
    expected_shading_model_label: &str,
) {
    assert_registry_material_pass_prewarm_dimensions_written_for_shading_model(
        prewarm_report,
        expected_shading_model,
        expected_shading_model_label,
    );
    let static_geometry = GEOMETRY_SOURCE_ID_STATIC_MESH.value().to_string();
    let shading_model = expected_shading_model.value().to_string();

    for (count, label) in [
        (
            runtime_report
                .dimension_summary
                .geometry_source_ids
                .get(&static_geometry),
            "static geometry source",
        ),
        (
            runtime_report
                .dimension_summary
                .shading_model_ids
                .get(&shading_model),
            expected_shading_model_label,
        ),
        (
            runtime_report.dimension_summary.quality_tiers.get("medium"),
            "medium quality tier",
        ),
    ] {
        if require_disk_hit {
            assert_runtime_dimension_disk_hit(count, case, label);
        } else {
            assert_runtime_dimension_requested_without_miss(count, case, label);
        }
    }
}

fn assert_prewarm_dimension_written(
    count: Option<&ShaderVariantPrewarmDimensionCount>,
    label: &str,
) {
    let count = count.unwrap_or_else(|| panic!("{label} should be present in prewarm report"));
    assert!(
        count.written_count >= 1,
        "{label} should include at least one written prewarm variant; count={count:?}"
    );
}

fn assert_runtime_dimension_disk_hit(
    count: Option<&ShaderVariantRuntimeDimensionCount>,
    case: RegistryShaderCase,
    dimension_label: &str,
) {
    let count = count.unwrap_or_else(|| {
        panic!(
            "runtime should report {dimension_label} dimension for {}",
            case.locator
        )
    });
    assert!(
        count.disk_hit_count >= 1,
        "runtime should disk-hit staged cache for {dimension_label} on {}; count={count:?}",
        case.locator
    );
    assert_eq!(count.compile_miss_count, 0);
}

fn assert_runtime_dimension_requested_without_miss(
    count: Option<&ShaderVariantRuntimeDimensionCount>,
    case: RegistryShaderCase,
    dimension_label: &str,
) {
    let count = count.unwrap_or_else(|| {
        panic!(
            "runtime should report {dimension_label} dimension for {}",
            case.locator
        )
    });
    assert!(
        count.request_count >= 1,
        "runtime should request {dimension_label} on {}; count={count:?}",
        case.locator
    );
    assert_eq!(count.compile_miss_count, 0);
}

pub(super) fn assert_runtime_shader_cache_root_empty(runtime_root: &Path, label: &str) {
    let file_count = recursive_file_count(runtime_root);
    assert_eq!(
        file_count,
        0,
        "{label}; runtime_root={} file_count={file_count}",
        runtime_root.display()
    );
}

fn recursive_file_count(root: &Path) -> usize {
    if !root.exists() {
        return 0;
    }

    let mut file_count = 0;
    let mut pending_dirs = vec![root.to_path_buf()];
    while let Some(dir) = pending_dirs.pop() {
        for entry in read_dir_entries(&dir) {
            let file_type = entry
                .file_type()
                .unwrap_or_else(|error| panic!("read file type for {:?}: {error}", entry.path()));
            if file_type.is_dir() {
                pending_dirs.push(entry.path());
            } else if file_type.is_file() {
                file_count += 1;
            }
        }
    }
    file_count
}

fn read_dir_entries(dir: &Path) -> Vec<fs::DirEntry> {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("read shader cache directory {dir:?}: {error}"))
        .map(|entry| entry.unwrap_or_else(|error| panic!("read shader cache entry: {error}")))
        .collect::<Vec<_>>()
}
