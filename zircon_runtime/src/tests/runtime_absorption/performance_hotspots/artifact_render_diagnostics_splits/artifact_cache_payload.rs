#[test]
fn runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed() {
    let cache_root = include_str!("../../../../asset/artifact/cache_payload.rs");
    let cache_json = include_str!("../../../../asset/artifact/cache_payload/json_value.rs");
    let cache_mesh = include_str!("../../../../asset/artifact/cache_payload/mesh.rs");
    let cache_scene = include_str!("../../../../asset/artifact/cache_payload/scene.rs");
    let cache_toml = include_str!("../../../../asset/artifact/cache_payload/toml_value.rs");
    let runtime_04_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    );
    let runtime_07_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_07_archive = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let artifact_doc = include_str!("../../../../../../docs/zircon_runtime/asset/artifact.md");

    for root_anchor in [
        "mod json_value;",
        "mod mesh;",
        "mod scene;",
        "mod toml_value;",
        "pub(super) enum ArtifactCacheAsset",
        "Data(ArtifactCacheDataAsset)",
        "Mesh(ArtifactCacheMeshAsset)",
    ] {
        assert!(
            cache_root.contains(root_anchor),
            "cache_payload.rs should retain artifact wire dispatcher anchor `{root_anchor}`"
        );
    }

    for moved_owner_anchor in [
        "enum ArtifactCacheJsonValue",
        "enum ArtifactCacheTomlValue",
        "struct ArtifactCacheMeshAsset",
        "enum ArtifactCacheMeshAttributeValues",
        "enum ArtifactCacheMeshIndices",
    ] {
        assert!(
            !cache_root.contains(moved_owner_anchor),
            "cache_payload.rs should not reclaim moved cache wire owner `{moved_owner_anchor}`"
        );
    }

    for json_anchor in [
        "pub(super) enum ArtifactCacheJsonValue",
        "pub(super) fn from_json",
        "pub(super) fn into_json",
        "pub(super) fn json_table_to_cache",
        "pub(super) fn cache_table_to_json",
    ] {
        assert!(
            cache_json.contains(json_anchor),
            "cache_payload/json_value.rs should own JSON wire anchor `{json_anchor}`"
        );
    }

    for mesh_anchor in [
        "pub(super) struct ArtifactCacheMeshAsset",
        "pub(super) fn into_asset",
        "enum ArtifactCacheMeshAttributeValues",
        "enum ArtifactCacheMeshIndices",
        "fn mesh_attribute_table_to_cache",
    ] {
        assert!(
            cache_mesh.contains(mesh_anchor),
            "cache_payload/mesh.rs should own mesh wire anchor `{mesh_anchor}`"
        );
    }

    for toml_anchor in [
        "pub(super) type ArtifactCacheTomlTable",
        "pub(super) enum ArtifactCacheTomlValue",
        "pub(super) fn from_toml",
        "pub(super) fn into_toml",
        "pub(super) fn toml_table_like_to_cache",
        "pub(super) fn cache_table_like_to_toml",
    ] {
        assert!(
            cache_toml.contains(toml_anchor),
            "cache_payload/toml_value.rs should own TOML wire anchor `{toml_anchor}`"
        );
    }

    assert!(
        cache_scene.contains(
            "use super::json_value::{cache_table_to_json, json_table_to_cache, ArtifactCacheJsonValue};"
        ),
        "scene cache wire owner should consume JSON conversion through cache_payload/json_value.rs"
    );

    for doc_anchor in [
        "artifact_cache_payload_owner_split_static_passed_cargo_deferred",
        "cache_payload/{json_value,mesh,toml_value}.rs",
        "large_file_hotspot_count = 41",
        "runtime-other = 16",
        "runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed",
        "expected_source_file_count = 22",
    ] {
        assert!(
            runtime_04_plan.contains(doc_anchor)
                || runtime_07_plan.contains(doc_anchor)
                || runtime_07_archive.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || hotspot_doc.contains(doc_anchor)
                || artifact_doc.contains(doc_anchor),
            "artifact cache payload owner split docs should retain `{doc_anchor}`"
        );
    }
}
