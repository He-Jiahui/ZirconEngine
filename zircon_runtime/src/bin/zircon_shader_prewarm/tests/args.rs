use std::ffi::OsString;

use zircon_runtime::core::framework::render::{
    GeometrySourceId, GEOMETRY_SOURCE_ID_MORPHED_MESH, GEOMETRY_SOURCE_ID_SKINNED_MESH,
    GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
    GEOMETRY_SOURCE_PLUGIN_ID_START, SHADING_MODEL_PLUGIN_ID_START,
};

use super::super::error::ShaderPrewarmArgsError;
use super::parse;

#[test]
fn shader_prewarm_args_default_to_static_geometry_source() {
    let args = parse(["--asset-root", "assets"].into_iter().map(OsString::from))
        .unwrap()
        .unwrap();

    assert_eq!(args.geometry_sources, vec![GEOMETRY_SOURCE_ID_STATIC_MESH]);
}

#[test]
fn shader_prewarm_args_expand_all_builtin_geometry_sources() {
    let args = parse(
        ["--asset-root", "assets", "--geometry-source", "all"]
            .into_iter()
            .map(OsString::from),
    )
    .unwrap()
    .unwrap();

    assert_eq!(
        args.geometry_sources,
        vec![
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
            GEOMETRY_SOURCE_ID_MORPHED_MESH,
            GEOMETRY_SOURCE_ID_SKINNED_MORPHED_MESH,
        ]
    );
}

#[test]
fn shader_prewarm_args_parse_custom_shading_model_plugin_ids() {
    let args = parse(
        [
            "--asset-root",
            "assets",
            "--shading-model-id",
            "custom:Subsurface=16",
            "--shading-model-id",
            "toon=17",
        ]
        .into_iter()
        .map(OsString::from),
    )
    .unwrap()
    .unwrap();

    assert_eq!(
        args.shading_model_ids
            .get("custom:subsurface")
            .copied()
            .unwrap()
            .value(),
        SHADING_MODEL_PLUGIN_ID_START
    );
    assert_eq!(
        args.shading_model_ids
            .get("custom:toon")
            .copied()
            .unwrap()
            .value(),
        SHADING_MODEL_PLUGIN_ID_START + 1
    );
}

#[test]
fn shader_prewarm_args_parse_custom_geometry_source_plugin_ids() {
    let args = parse(
        [
            "--asset-root",
            "assets",
            "--geometry-source-id",
            "custom:GpuDriven=4",
            "--geometry-source-id",
            "foliage=5",
        ]
        .into_iter()
        .map(OsString::from),
    )
    .unwrap()
    .unwrap();

    assert_eq!(
        args.geometry_source_ids
            .get("custom:gpudriven")
            .copied()
            .unwrap()
            .value(),
        GEOMETRY_SOURCE_PLUGIN_ID_START
    );
    assert_eq!(
        args.geometry_source_ids
            .get("custom:foliage")
            .copied()
            .unwrap()
            .value(),
        GEOMETRY_SOURCE_PLUGIN_ID_START + 1
    );
    assert_eq!(
        args.geometry_sources,
        vec![
            GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START),
            GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START + 1),
        ]
    );
}

#[test]
fn shader_prewarm_args_parse_resource_registry_path() {
    let args = parse(
        [
            "--asset-root",
            "assets",
            "--resource-registry",
            "Project/.zircon/cache/resources.json",
        ]
        .into_iter()
        .map(OsString::from),
    )
    .unwrap()
    .unwrap();

    assert_eq!(
        args.resource_registry.unwrap(),
        std::path::PathBuf::from("Project/.zircon/cache/resources.json")
    );
}

#[test]
fn shader_prewarm_args_parse_shader_permutation_registry_path() {
    let args = parse(
        [
            "--asset-root",
            "assets",
            "--shader-permutation-registry",
            "Project/.zircon/cache/shader_permutation_registry.json",
        ]
        .into_iter()
        .map(OsString::from),
    )
    .unwrap()
    .unwrap();

    assert_eq!(
        args.permutation_registries,
        vec![std::path::PathBuf::from(
            "Project/.zircon/cache/shader_permutation_registry.json"
        )]
    );
}

#[test]
fn shader_prewarm_args_parse_export_resource_registry_path() {
    let args = parse(
        [
            "--asset-root",
            "assets",
            "--export-resource-registry",
            "cache/shader_resource_records.json",
        ]
        .into_iter()
        .map(OsString::from),
    )
    .unwrap()
    .unwrap();

    assert_eq!(
        args.export_resource_registry.unwrap(),
        std::path::PathBuf::from("cache/shader_resource_records.json")
    );
}

#[test]
fn shader_prewarm_args_parse_wgpu_module_validation_flag() {
    let args = parse(
        ["--asset-root", "assets", "--validate-wgpu-modules"]
            .into_iter()
            .map(OsString::from),
    )
    .unwrap()
    .unwrap();

    assert!(args.validate_wgpu_modules);
}

#[test]
fn shader_prewarm_args_parse_wgpu_pipeline_validation_flag() {
    let args = parse(
        ["--asset-root", "assets", "--validate-wgpu-pipelines"]
            .into_iter()
            .map(OsString::from),
    )
    .unwrap()
    .unwrap();

    assert!(args.validate_wgpu_pipelines);
}

#[test]
fn shader_prewarm_args_reject_builtin_shading_model_id_range() {
    let error = parse(
        [
            "--asset-root",
            "assets",
            "--shading-model-id",
            "custom:subsurface=2",
        ]
        .into_iter()
        .map(OsString::from),
    )
    .unwrap_err();

    assert!(matches!(error, ShaderPrewarmArgsError::Usage(_)));
    assert!(error
        .to_string()
        .contains("plugin shading model ids must be >= 16"));
}

#[test]
fn shader_prewarm_args_reject_builtin_geometry_source_id_range() {
    let error = parse(
        [
            "--asset-root",
            "assets",
            "--geometry-source-id",
            "custom:gpu-driven=3",
        ]
        .into_iter()
        .map(OsString::from),
    )
    .unwrap_err();

    assert!(matches!(error, ShaderPrewarmArgsError::Usage(_)));
    assert!(error
        .to_string()
        .contains("plugin geometry source ids must be >= 4"));
}

#[test]
fn shader_prewarm_args_missing_value_reports_typed_usage_error() {
    let error = parse(["--asset-root"].into_iter().map(OsString::from)).unwrap_err();

    assert!(matches!(error, ShaderPrewarmArgsError::Usage(_)));
    assert!(error.to_string().contains("missing value for --asset-root"));
}
