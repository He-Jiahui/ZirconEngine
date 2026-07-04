use std::collections::BTreeMap;
use std::fs;

use zircon_runtime::core::framework::render::{
    ShaderPassType, ShaderQualityTier, GEOMETRY_SOURCE_ID_STATIC_MESH,
};
use zircon_runtime::core::resource::{
    ResourceId, ResourceKind, ResourceLocator, ResourceManager, ResourceRecord, ResourceState,
};

use super::super::{
    asset_root_manifest_with_resource_registry_revisions, merge_manifests,
    resource_registry::{
        shader_resource_records_from_asset_root, shader_resource_records_from_asset_roots,
        shader_resource_records_from_manager, ShaderPrewarmResourceRegistryOverlay,
    },
};

#[test]
fn shader_prewarm_asset_root_manifest_uses_resource_registry_revision_overlay() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_registry_revision_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("shaders")).unwrap();
    fs::write(root.join("shaders/example.wgsl"), "fn example() {}\n").unwrap();
    fs::write(
        root.join("shaders/example.wgsl.zmeta"),
        r#"format_version = 6
uuid = "00000000-0000-0000-0000-000000000045"
url = "res://shaders/example"
asset_kind = "Shader"
unit = "single"
source_hash = "source-hash-registry-fallback"
"#,
    )
    .unwrap();

    let mut record = ResourceRecord::new(
        ResourceId::from_stable_label("registry-shader"),
        ResourceKind::Shader,
        ResourceLocator::parse("res://shaders/example").unwrap(),
    )
    .with_state(ResourceState::Ready);
    record.revision = 77;
    let overlay = ShaderPrewarmResourceRegistryOverlay::from_records([record]);

    let manifest = asset_root_manifest_with_resource_registry_revisions(
        &root,
        &[ShaderQualityTier::Medium],
        &[GEOMETRY_SOURCE_ID_STATIC_MESH],
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
        Some(&overlay),
    )
    .unwrap();

    assert_eq!(manifest.variants.len(), 6);
    assert!(manifest
        .variants
        .iter()
        .all(|request| request.key.material_revision == 77));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_asset_root_exports_shader_resource_records() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_registry_export_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("shaders")).unwrap();
    fs::write(root.join("shaders/example.wgsl"), "fn example() {}\n").unwrap();
    fs::write(
        root.join("shaders/example.wgsl.zmeta"),
        r#"format_version = 6
uuid = "00000000-0000-0000-0000-000000000046"
url = "res://shaders/example"
asset_kind = "Shader"
unit = "single"
source_hash = "source-hash-registry-export"
"#,
    )
    .unwrap();

    let records = shader_resource_records_from_asset_root(&root).unwrap();

    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.kind, ResourceKind::Shader);
    assert_eq!(record.state, ResourceState::Ready);
    assert_eq!(
        record.primary_locator,
        ResourceLocator::parse("res://shaders/example").unwrap()
    );
    assert_ne!(record.revision, 0);

    let overlay = ShaderPrewarmResourceRegistryOverlay::from_records(records.clone());
    let manifest = asset_root_manifest_with_resource_registry_revisions(
        &root,
        &[ShaderQualityTier::Medium],
        &[GEOMETRY_SOURCE_ID_STATIC_MESH],
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
        Some(&overlay),
    )
    .unwrap();

    assert_eq!(manifest.variants.len(), 6);
    assert!(manifest
        .variants
        .iter()
        .all(|request| request.key.material_revision == record.revision));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_project_and_plugin_asset_roots_use_exported_registry_revisions() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_project_plugin_live_export_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let project_root = root.join("project_assets");
    let plugin_root = root.join("plugin_assets");
    write_named_shader_with_meta(
        &project_root,
        "project",
        "00000000-0000-0000-0000-000000000054",
        "res://project/shaders/project",
        "source-hash-project-live-registry",
    );
    write_named_shader_with_meta(
        &plugin_root,
        "plugin",
        "00000000-0000-0000-0000-000000000055",
        "package://virtual_geometry/shaders/plugin",
        "source-hash-plugin-live-registry",
    );

    let records =
        shader_resource_records_from_asset_roots(&[project_root.clone(), plugin_root.clone()])
            .unwrap();
    let overlay = ShaderPrewarmResourceRegistryOverlay::from_records(records.clone());
    let mut manifest = asset_root_manifest_with_resource_registry_revisions(
        &project_root,
        &[ShaderQualityTier::Medium],
        &[GEOMETRY_SOURCE_ID_STATIC_MESH],
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
        Some(&overlay),
    )
    .unwrap();
    manifest = merge_manifests(
        manifest,
        asset_root_manifest_with_resource_registry_revisions(
            &plugin_root,
            &[ShaderQualityTier::Medium],
            &[GEOMETRY_SOURCE_ID_STATIC_MESH],
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            Some(&overlay),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(manifest.variants.len(), 12);
    for record in &records {
        let label = record.primary_locator.to_string();
        let requests = manifest
            .variants
            .iter()
            .filter(|request| request.source_label == label)
            .collect::<Vec<_>>();
        assert_eq!(
            requests.len(),
            6,
            "expected six material passes for {label}"
        );
        assert!(requests
            .iter()
            .all(|request| request.key.material_revision == record.revision));
        assert!(requests
            .iter()
            .all(|request| request.template_revision == "zr-material-template-v1"));
        let forward = requests
            .iter()
            .find(|request| request.key.pass_type == ShaderPassType::Forward)
            .expect("forward project/plugin registry request");
        assert!(forward.wgsl_source.contains("fn vs_main("));
        assert!(forward.wgsl_source.contains("fn fs_main("));
        assert!(forward
            .wgsl_source
            .contains("// include: zr_template_forward.wgsl"));
    }

    let _ = fs::remove_dir_all(root);
}

#[derive(Debug)]
struct ShaderPayload;

#[test]
fn shader_prewarm_resource_registry_overlay_uses_live_resource_manager_shader_revisions() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_live_registry_export_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("shaders")).unwrap();
    fs::write(root.join("shaders/live.wgsl"), "fn live() {}\n").unwrap();
    fs::write(
        root.join("shaders/live.wgsl.zmeta"),
        r#"format_version = 6
uuid = "00000000-0000-0000-0000-000000000047"
url = "res://shaders/live"
asset_kind = "Shader"
unit = "single"
source_hash = "source-hash-live-manager-fallback"
"#,
    )
    .unwrap();

    let manager = ResourceManager::new();
    let live_locator = ResourceLocator::parse("res://shaders/live").unwrap();
    let live_id = ResourceId::from_locator(&live_locator);
    manager.register_ready(
        ResourceRecord::new(live_id, ResourceKind::Shader, live_locator.clone())
            .with_source_hash("live-manager-shader-a"),
        ShaderPayload,
    );
    manager.register_ready(
        ResourceRecord::new(live_id, ResourceKind::Shader, live_locator)
            .with_source_hash("live-manager-shader-b"),
        ShaderPayload,
    );
    let model_locator = ResourceLocator::parse("res://models/mesh.glb").unwrap();
    manager.register_ready(
        ResourceRecord::new(
            ResourceId::from_locator(&model_locator),
            ResourceKind::Model,
            model_locator,
        )
        .with_source_hash("live-manager-model"),
        ShaderPayload,
    );
    let pending_locator = ResourceLocator::parse("res://shaders/pending").unwrap();
    manager.register_record(ResourceRecord::new(
        ResourceId::from_locator(&pending_locator),
        ResourceKind::Shader,
        pending_locator,
    ));

    let records = shader_resource_records_from_manager(&manager);

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, live_id);
    assert_eq!(records[0].kind, ResourceKind::Shader);
    assert_eq!(records[0].state, ResourceState::Ready);
    assert_eq!(records[0].revision, 2);
    let live_revision = records[0].revision;

    let overlay = ShaderPrewarmResourceRegistryOverlay::from_records(records);
    let manifest = asset_root_manifest_with_resource_registry_revisions(
        &root,
        &[ShaderQualityTier::Medium],
        &[GEOMETRY_SOURCE_ID_STATIC_MESH],
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
        Some(&overlay),
    )
    .unwrap();

    assert_eq!(manifest.variants.len(), 6);
    assert!(manifest
        .variants
        .iter()
        .all(|request| request.key.material_revision == live_revision));
    let _ = fs::remove_dir_all(root);
}

fn write_named_shader_with_meta(
    asset_root: &std::path::Path,
    name: &str,
    id: &str,
    locator: &str,
    source_hash: &str,
) {
    fs::create_dir_all(asset_root.join("shaders")).unwrap();
    fs::write(
        asset_root.join("shaders").join(format!("{name}.wgsl")),
        format!("fn {name}() {{}}\n"),
    )
    .unwrap();
    fs::write(
        asset_root
            .join("shaders")
            .join(format!("{name}.wgsl.zmeta")),
        format!(
            r#"format_version = 6
uuid = "{id}"
url = "{locator}"
asset_kind = "Shader"
unit = "single"
source_hash = "{source_hash}"
"#
        ),
    )
    .unwrap();
}
