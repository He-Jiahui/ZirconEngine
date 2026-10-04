use super::{
    viewer_project_assets_are_ready_for_fixture, write_viewer_material,
    write_viewer_project_assets_for_fixture, AssetMetaDocument, AssetReference, MaterialAsset,
    ReferenceResolutionError, SceneAsset, SPHERE_RINGS, SPHERE_SEGMENTS,
    VIEWER_PROJECT_ASSET_PATHS, VIEWER_PROJECT_ASSET_ROOT, VIEWER_PROJECT_SOURCE_PATHS,
};
use zircon_runtime::asset::assets::ZMaterialDocument;

use crate::material_fixture::ViewerMaterialFixture;
use crate::work_paths::viewer_test_artifact_root;

#[test]
fn viewer_mirror_mesh_stays_within_its_startup_triangle_budget() {
    assert_eq!(SPHERE_RINGS * SPHERE_SEGMENTS * 2, 16_384);
}

#[test]
fn viewer_project_reuse_requires_a_completed_asset_tree() {
    let root = viewer_test_artifact_root("asset-ready");
    let asset_root = root.join(VIEWER_PROJECT_ASSET_ROOT);
    std::fs::create_dir_all(&root).expect("test cache root should be created");

    assert!(!viewer_project_assets_are_ready_for_fixture(
        &asset_root,
        ViewerMaterialFixture::MetalMirror,
    ));
    for relative_path in VIEWER_PROJECT_ASSET_PATHS {
        let path = asset_root.join(relative_path);
        std::fs::create_dir_all(path.parent().expect("asset path should have a parent"))
            .expect("test asset parent should be created");
        std::fs::write(path, "fixture\n").expect("test asset should be written");
    }
    assert!(
        !viewer_project_assets_are_ready_for_fixture(
            &asset_root,
            ViewerMaterialFixture::MetalMirror,
        ),
        "file presence alone must not reuse a stale material fixture"
    );

    write_viewer_project_assets_for_fixture(&asset_root, ViewerMaterialFixture::MetalMirror)
        .expect("a stale project tree should be replaced");
    assert!(viewer_project_assets_are_ready_for_fixture(
        &asset_root,
        ViewerMaterialFixture::MetalMirror,
    ));

    std::fs::remove_file(asset_root.join(VIEWER_PROJECT_ASSET_PATHS[1]))
        .expect("test asset should be removable");
    assert!(
        !viewer_project_assets_are_ready_for_fixture(
            &asset_root,
            ViewerMaterialFixture::MetalMirror,
        ),
        "a partial cache must regenerate the viewer project assets"
    );

    std::fs::remove_dir_all(&root).expect("test cache root should be removed");
}

#[test]
fn viewer_project_replaces_a_complete_tree_when_its_material_fixture_changes() {
    let root = viewer_test_artifact_root("fixture-bound-project-assets");
    let asset_root = root.join(VIEWER_PROJECT_ASSET_ROOT);

    let mirror_report =
        write_viewer_project_assets_for_fixture(&asset_root, ViewerMaterialFixture::MetalMirror)
            .expect("mirror fixture should publish a complete project tree");
    assert_eq!(mirror_report.mesh_generation_samples(), 1);
    assert!(super::viewer_project_assets_are_ready_for_fixture(
        &asset_root,
        ViewerMaterialFixture::MetalMirror,
    ));
    assert!(
        !super::viewer_project_assets_are_ready_for_fixture(
            &asset_root,
            ViewerMaterialFixture::DielectricIor,
        ),
        "a complete mirror tree must not satisfy the dielectric IOR fixture"
    );

    let ior_report =
        write_viewer_project_assets_for_fixture(&asset_root, ViewerMaterialFixture::DielectricIor)
            .expect("a stale complete mirror tree should be replaced by the IOR fixture");
    assert_eq!(ior_report.mesh_generation_samples(), 1);
    assert!(super::viewer_project_assets_are_ready_for_fixture(
        &asset_root,
        ViewerMaterialFixture::DielectricIor,
    ));
    assert!(
        !super::viewer_project_assets_are_ready_for_fixture(
            &asset_root,
            ViewerMaterialFixture::MetalMirror,
        ),
        "the replacement tree must no longer satisfy the mirror fixture"
    );

    std::fs::remove_dir_all(&root).expect("test cache root should be removed");
}

#[test]
fn viewer_project_publishes_stable_sidecars_and_scene_references_without_preopening() {
    let root = viewer_test_artifact_root("project-assets");
    let asset_root = root.join(VIEWER_PROJECT_ASSET_ROOT);
    std::fs::create_dir_all(&root).expect("test cache root should be created");

    let cold_report =
        write_viewer_project_assets_for_fixture(&asset_root, ViewerMaterialFixture::MetalMirror)
            .expect("viewer project assets should publish without opening a project manager");
    assert!(viewer_project_assets_are_ready_for_fixture(
        &asset_root,
        ViewerMaterialFixture::MetalMirror,
    ));
    assert_eq!(cold_report.mesh_generation_samples(), 1);
    assert_eq!(cold_report.filesystem_writes(), 6);
    assert!(cold_report.serialized_source_bytes() > 0);

    let references =
        super::ViewerProjectAssetReferences::versioned(ViewerMaterialFixture::MetalMirror)
            .expect("viewer references should be well-formed");
    let model_meta =
        AssetMetaDocument::load(asset_root.join("models/single_pbr_sphere.model.toml.zmeta"))
            .expect("model sidecar should be readable");
    let material_meta =
        AssetMetaDocument::load(asset_root.join("materials/single_metal_sphere.zmaterial.zmeta"))
            .expect("material sidecar should be readable");
    assert_eq!(model_meta.uuid, references.model.uuid);
    assert_eq!(material_meta.uuid, references.material.uuid);

    let scene_source =
        std::fs::read_to_string(asset_root.join("scenes/single_pbr_sphere.scene.toml"))
            .expect("viewer scene should be readable");
    let expected_references = [references.model.clone(), references.material.clone()];
    let scene = SceneAsset::from_project_toml_str(&scene_source, |persisted| {
        let project_reference =
            persisted
                .project_ref()
                .ok_or_else(|| ReferenceResolutionError::Registry {
                    message: "viewer scene should only contain project references".to_string(),
                })?;
        let reference = expected_references
            .iter()
            .find(|reference| reference.uuid == project_reference.guid())
            .cloned()
            .ok_or_else(|| ReferenceResolutionError::Registry {
                message: "viewer scene should retain its stable generated identity".to_string(),
            })?;
        assert_eq!(
            project_reference.path_hint().as_str(),
            format!("{VIEWER_PROJECT_ASSET_ROOT}/{}", reference.locator.path()),
            "viewer scene must preserve the project-relative source path hint"
        );
        Ok(reference)
    })
    .expect("viewer scene should deserialize against its generated sidecars");
    assert_eq!(scene.direct_references(), expected_references.to_vec());

    let warm_report =
        write_viewer_project_assets_for_fixture(&asset_root, ViewerMaterialFixture::MetalMirror)
            .expect("ready viewer project assets should be reused without writes");
    assert_eq!(
        warm_report,
        super::ViewerProjectAssetGenerationReport::reused()
    );

    std::fs::remove_dir_all(&root).expect("test cache root should be removed");
}

#[test]
fn viewer_project_replaces_an_incomplete_versioned_tree_without_deleting_a_ready_one() {
    let root = viewer_test_artifact_root("incomplete-project-assets");
    let asset_root = root.join(VIEWER_PROJECT_ASSET_ROOT);
    std::fs::create_dir_all(&asset_root).expect("incomplete asset root should be created");
    std::fs::write(asset_root.join("stale.partial"), "incomplete\n")
        .expect("incomplete asset marker should be written");

    let report =
        write_viewer_project_assets_for_fixture(&asset_root, ViewerMaterialFixture::MetalMirror)
            .expect("an incomplete versioned tree should be replaced by a complete tree");
    assert!(viewer_project_assets_are_ready_for_fixture(
        &asset_root,
        ViewerMaterialFixture::MetalMirror,
    ));
    assert_eq!(report.mesh_generation_samples(), 1);
    assert_eq!(report.filesystem_writes(), 6);
    assert!(
        !asset_root.join("stale.partial").exists(),
        "the published tree must contain only the current generated artifacts"
    );
    let replaced_roots = std::fs::read_dir(&root)
        .expect("project root should be readable")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(".zpv4-r-"))
        .count();
    assert_eq!(
        replaced_roots, 0,
        "the displaced stale tree should be removed after publication"
    );

    std::fs::remove_dir_all(&root).expect("test cache root should be removed");
}

#[test]
fn viewer_asset_writer_leaves_project_open_and_import_to_the_runtime_manager() {
    const SOURCE: &str = include_str!("../project_assets.rs");
    let writer = SOURCE
        .split_once("pub(crate) fn write_viewer_project_assets")
        .and_then(|(_, source)| source.split_once("fn viewer_asset_staging_root"))
        .map(|(writer, _)| writer)
        .expect("viewer asset writer should retain a bounded implementation");

    assert!(writer.contains("publish_viewer_project_assets"));
    assert!(writer.contains("write_viewer_asset_meta"));
    assert!(writer.contains("ViewerProjectAssetGenerationReport::generated"));
    assert_eq!(VIEWER_PROJECT_SOURCE_PATHS.len(), 3);
    assert!(
        !SOURCE.contains(concat!("remove_dir_all(", "asset_root)")),
        "a competing publication must never delete a completed immutable cache"
    );
    assert!(
        !writer.contains("ProjectManager::open(") && !writer.contains("scan_and_import("),
        "the runtime AssetManager owns the sole project open and import generation"
    );
}

#[test]
fn competing_viewer_project_publish_reuses_the_completed_immutable_asset_tree() {
    let root = viewer_test_artifact_root("project-publish-contention");
    let asset_root = root.join(VIEWER_PROJECT_ASSET_ROOT);
    write_viewer_project_assets_for_fixture(&asset_root, ViewerMaterialFixture::MetalMirror)
        .expect("completed asset root should be published");
    let staging_root = root.join("competing-staging-root");
    std::fs::create_dir_all(&staging_root).expect("competing staging root should be created");
    std::fs::write(staging_root.join("partial.asset"), "partial\n")
        .expect("competing staging payload should be written");

    assert!(
        !super::publish_viewer_project_assets(
            &staging_root,
            &asset_root,
            ViewerMaterialFixture::MetalMirror,
        )
        .expect("a completed immutable asset root should win publication"),
        "a competing publisher must reuse the already-complete versioned tree"
    );
    assert!(viewer_project_assets_are_ready_for_fixture(
        &asset_root,
        ViewerMaterialFixture::MetalMirror,
    ));
    assert!(
        !staging_root.exists(),
        "a losing publisher must discard only its private staging tree"
    );

    std::fs::remove_dir_all(&root).expect("test cache root should be removed");
}

#[test]
fn losing_cold_publication_keeps_its_completed_generation_report() {
    let root = viewer_test_artifact_root("project-generation-report");
    let asset_root = root.join(VIEWER_PROJECT_ASSET_ROOT);
    write_viewer_project_assets_for_fixture(&asset_root, ViewerMaterialFixture::MetalMirror)
        .expect("completed asset root should be published");
    let staging_root = root.join("losing-staging-root");
    std::fs::create_dir_all(&staging_root).expect("losing staging root should be created");
    std::fs::write(staging_root.join("generated.asset"), "generated\n")
        .expect("generated staging payload should be written");

    let report = super::finish_viewer_project_asset_generation(
        &staging_root,
        &asset_root,
        ViewerMaterialFixture::MetalMirror,
        123,
    )
    .expect("a losing cold publication should retain its completed work report");
    assert_eq!(report.mesh_generation_samples(), 1);
    assert_eq!(report.filesystem_writes(), 6);
    assert_eq!(report.serialized_source_bytes(), 123);
    assert!(
        !staging_root.exists(),
        "a losing publication should discard only its private staging tree"
    );
    assert!(viewer_project_assets_are_ready_for_fixture(
        &asset_root,
        ViewerMaterialFixture::MetalMirror,
    ));

    std::fs::remove_dir_all(&root).expect("test cache root should be removed");
}

#[test]
fn viewer_mirror_material_matches_environment_only_prewarm_variant() {
    let root = viewer_test_artifact_root("mirror-material");
    let material_path = root.join("single_metal_sphere.zmaterial");

    write_viewer_material(material_path.clone(), ViewerMaterialFixture::MetalMirror)
        .expect("viewer mirror material should be generated");
    let source =
        std::fs::read_to_string(&material_path).expect("viewer mirror material should be readable");
    let document: toml::Value =
        toml::from_str(&source).expect("viewer mirror material should remain valid TOML");
    let overrides = document
        .get("overrides")
        .and_then(toml::Value::as_table)
        .expect("viewer mirror material should keep its PBR overrides");

    assert!(
        source.contains("builtin://shader/pbr.wgsl"),
        "viewer mirror material must use the builtin PBR shader warmed by the renderer"
    );
    assert_eq!(
        overrides
            .get("lighting_model")
            .and_then(toml::Value::as_str),
        Some("pbr")
    );
    assert_eq!(
        overrides
            .get("receive_shadows")
            .and_then(toml::Value::as_bool),
        Some(false),
        "the viewer must reuse the prewarmed no-shadow-receiver Base variant"
    );
    assert_eq!(
        overrides.get("metallic").and_then(toml::Value::as_float),
        Some(1.0)
    );
    assert_eq!(
        overrides.get("roughness").and_then(toml::Value::as_float),
        Some(0.0)
    );
    assert!(
        document
            .get("textures")
            .and_then(toml::Value::as_table)
            .is_none_or(toml::Table::is_empty),
        "the environment-only viewer prewarms the static no-texture material variant"
    );

    let material = ZMaterialDocument::from_project_toml_str(&source, |reference| {
        reference
            .builtin_locator()
            .cloned()
            .map(AssetReference::from_locator)
            .ok_or_else(|| ReferenceResolutionError::Registry {
                message: "viewer material test expects a builtin shader reference".to_string(),
            })
    })
    .map(MaterialAsset::from_zmaterial_document)
    .expect("viewer mirror material should deserialize into its runtime asset");
    let descriptor = material.standard_material_descriptor();
    assert!(
        !descriptor.receive_shadows,
        "the runtime material descriptor must keep the no-shadow-receiver prewarm key"
    );
    assert!(
        descriptor.base_color_texture.is_none()
            && descriptor.normal_texture.is_none()
            && descriptor.metallic_roughness_texture.is_none()
            && descriptor.occlusion_texture.is_none()
            && descriptor.emissive_texture.is_none(),
        "the runtime material descriptor must retain the prewarm's static no-texture key"
    );

    std::fs::remove_dir_all(&root).expect("test cache root should be removed");
}

#[test]
fn dielectric_ior_fixture_serializes_the_routed_standard_material_input() {
    let root = viewer_test_artifact_root("dielectric-ior-material");
    let material_path = root.join("single_dielectric_sphere.zmaterial");

    write_viewer_material(material_path.clone(), ViewerMaterialFixture::DielectricIor)
        .expect("viewer dielectric material should be generated");
    let source = std::fs::read_to_string(&material_path)
        .expect("viewer dielectric material should be readable");
    let document: toml::Value =
        toml::from_str(&source).expect("viewer dielectric material should remain valid TOML");
    let overrides = document
        .get("overrides")
        .and_then(toml::Value::as_table)
        .expect("viewer dielectric material should keep its PBR overrides");

    assert_eq!(
        overrides.get("ior").and_then(toml::Value::as_float),
        Some(2.0)
    );
    assert_eq!(
        overrides.get("metallic").and_then(toml::Value::as_float),
        Some(0.0)
    );
    assert_eq!(
        overrides.get("roughness").and_then(toml::Value::as_float),
        Some(0.08)
    );

    let material = ZMaterialDocument::from_project_toml_str(&source, |reference| {
        reference
            .builtin_locator()
            .cloned()
            .map(AssetReference::from_locator)
            .ok_or_else(|| ReferenceResolutionError::Registry {
                message: "viewer material test expects a builtin shader reference".to_string(),
            })
    })
    .map(MaterialAsset::from_zmaterial_document)
    .expect("viewer dielectric material should deserialize into its runtime asset");
    let features = material.advanced_pbr_features();
    assert_eq!(features.ior, 2.0);
    assert!(
        (features.dielectric_f0() - (1.0 / 9.0)).abs() <= f32::EPSILON,
        "the fixed IOR fixture must derive its F0 before shader routing"
    );
    assert!(features.uses_dielectric_f0_override());
    assert!(
        features.requires_forward_path(),
        "the fixed IOR fixture must exercise PipelineKey::pbr_ior_override"
    );
    assert!(
        viewer_material_matches_fixture(&material_path, ViewerMaterialFixture::DielectricIor),
        "the reusable viewer-project guard must verify the same derived IOR contract"
    );

    std::fs::remove_dir_all(&root).expect("test cache root should be removed");
}
