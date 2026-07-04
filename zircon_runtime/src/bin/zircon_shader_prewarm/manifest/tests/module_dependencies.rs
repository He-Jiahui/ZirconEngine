use std::collections::BTreeMap;
use std::fs;

use zircon_runtime::core::framework::render::{ShaderQualityTier, GEOMETRY_SOURCE_ID_STATIC_MESH};

use super::super::{asset_root_manifest, asset_root_manifest_with_resource_registry_revisions};

#[test]
fn shader_prewarm_asset_root_manifest_tracks_imported_include_module_revisions() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_include_revision_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("shaders/materials/hero")).unwrap();
    fs::create_dir_all(root.join("shaders/materials/observer")).unwrap();
    fs::create_dir_all(root.join("shaders/includes/noise")).unwrap();
    fs::write(
        root.join("shaders/materials/hero/hero.zshader"),
        r#"version = 2
kind = "surface"
wgsl_files = ["surface.wgsl"]
shading_model = "standard_pbr"

[[imports]]
source = "project::includes::noise"
"#,
    )
    .unwrap();
    fs::write(
        root.join("shaders/materials/hero/surface.wgsl"),
        "#include <project::includes::noise>\nfn hero_surface() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("shaders/materials/observer/observer.zshader"),
        r#"version = 2
kind = "surface"
wgsl_files = ["surface.wgsl"]
shading_model = "standard_pbr"
"#,
    )
    .unwrap();
    fs::write(
        root.join("shaders/materials/observer/surface.wgsl"),
        "fn observer_surface() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("shaders/includes/noise/noise.zshader"),
        r#"version = 2
kind = "include"
import_path = "project::includes::noise"
wgsl_files = ["noise.wgsl"]
"#,
    )
    .unwrap();
    fs::write(
        root.join("shaders/includes/noise/noise.wgsl"),
        "fn noise_value() -> f32 { return 0.25; }\n",
    )
    .unwrap();

    let first = asset_root_manifest(&root).unwrap();
    let hero_first = first
        .variants
        .iter()
        .find(|request| request.source_label.ends_with("hero/hero.zshader"))
        .expect("hero surface request");
    let observer_first = first
        .variants
        .iter()
        .find(|request| request.source_label.ends_with("observer/observer.zshader"))
        .expect("observer surface request");
    assert!(
        hero_first.include_content_hashes.len() > observer_first.include_content_hashes.len(),
        "imported include content hashes must be attached to the referencing surface"
    );

    fs::write(
        root.join("shaders/includes/noise/noise.wgsl"),
        "fn noise_value() -> f32 { return 0.75; }\n",
    )
    .unwrap();
    let second = asset_root_manifest(&root).unwrap();
    let hero_second = second
        .variants
        .iter()
        .find(|request| request.source_label.ends_with("hero/hero.zshader"))
        .expect("hero surface request after include edit");
    let observer_second = second
        .variants
        .iter()
        .find(|request| request.source_label.ends_with("observer/observer.zshader"))
        .expect("observer surface request after include edit");

    assert_ne!(
        hero_first.key.material_revision, hero_second.key.material_revision,
        "referencing shader must change revision when an imported include module changes"
    );
    assert_eq!(
        observer_first.key.material_revision, observer_second.key.material_revision,
        "non-referencing shader must keep its revision when an unrelated include module changes"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_asset_root_manifest_tracks_registry_shader_module_revisions() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_registry_include_revision_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("shaders/materials/hero")).unwrap();
    fs::create_dir_all(root.join("shaders/materials/observer")).unwrap();
    fs::write(
        root.join("shaders/materials/hero/hero.zshader"),
        r#"version = 2
kind = "surface"
wgsl_files = ["surface.wgsl"]
shading_model = "standard_pbr"

[[imports]]
source = "custom::toon::noise"
"#,
    )
    .unwrap();
    fs::write(
        root.join("shaders/materials/hero/surface.wgsl"),
        "#include <custom::toon::noise>\nfn hero_surface() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("shaders/materials/observer/observer.zshader"),
        r#"version = 2
kind = "surface"
wgsl_files = ["surface.wgsl"]
shading_model = "standard_pbr"
"#,
    )
    .unwrap();
    fs::write(
        root.join("shaders/materials/observer/surface.wgsl"),
        "fn observer_surface() {}\n",
    )
    .unwrap();

    let mut first_modules = BTreeMap::new();
    first_modules.insert(
        "custom::toon::noise".to_string(),
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
    );
    let first = asset_root_manifest_with_resource_registry_revisions(
        &root,
        &[ShaderQualityTier::Medium],
        &[GEOMETRY_SOURCE_ID_STATIC_MESH],
        &BTreeMap::new(),
        &BTreeMap::new(),
        &first_modules,
        None,
    )
    .unwrap();
    let hero_first = first
        .variants
        .iter()
        .find(|request| request.source_label.ends_with("hero/hero.zshader"))
        .expect("hero surface request");
    let observer_first = first
        .variants
        .iter()
        .find(|request| request.source_label.ends_with("observer/observer.zshader"))
        .expect("observer surface request");
    assert!(
        hero_first.include_content_hashes.len() > observer_first.include_content_hashes.len(),
        "registry shader module hash must be attached to the referencing surface"
    );

    let mut second_modules = BTreeMap::new();
    second_modules.insert(
        "custom::toon::noise".to_string(),
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
    );
    let second = asset_root_manifest_with_resource_registry_revisions(
        &root,
        &[ShaderQualityTier::Medium],
        &[GEOMETRY_SOURCE_ID_STATIC_MESH],
        &BTreeMap::new(),
        &BTreeMap::new(),
        &second_modules,
        None,
    )
    .unwrap();
    let hero_second = second
        .variants
        .iter()
        .find(|request| request.source_label.ends_with("hero/hero.zshader"))
        .expect("hero surface request after registry module edit");
    let observer_second = second
        .variants
        .iter()
        .find(|request| request.source_label.ends_with("observer/observer.zshader"))
        .expect("observer surface request after registry module edit");

    assert_ne!(
        hero_first.key.material_revision, hero_second.key.material_revision,
        "referencing shader must change revision when an external shader module changes"
    );
    assert_eq!(
        observer_first.key.material_revision, observer_second.key.material_revision,
        "non-referencing shader must keep its revision when an external shader module changes"
    );
    let _ = fs::remove_dir_all(root);
}
