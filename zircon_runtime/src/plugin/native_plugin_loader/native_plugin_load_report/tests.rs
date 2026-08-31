use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::{
    PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleKind, PluginModuleManifest,
    PluginPackageManifest,
};

use super::{manifests::merge_package_manifest, NativePluginLoadReport};
use crate::plugin::native_plugin_loader::{
    LoadedNativePlugin, NativePluginBehaviorValidationReport, NativePluginCandidate,
    NativePluginDescriptor, NativePluginEntryReport, NativePluginLoadProjection,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
};

#[test]
fn load_report_collections_are_owned_by_the_report_module() {
    let report_source = include_str!("../native_plugin_load_report.rs").replace("\r\n", "\n");
    for field in ["discovered", "loaded", "diagnostics", "projection"] {
        assert!(
            !report_source.lines().any(|line| {
                let line = line.trim_start();
                line.starts_with("pub(") && line.contains(&format!("{field}:"))
            }),
            "NativePluginLoadReport::{field} must remain owner-private"
        );
    }

    let authority_source = include_str!("../discover/authority.rs").replace("\r\n", "\n");
    assert!(
        !authority_source.contains("discovered: snapshot.candidates().to_vec()"),
        "discovery authority must construct reports through the report owner"
    );
    assert!(
        !authority_source.contains("report.diagnostics.push("),
        "discovery authority must mutate reports through the report owner"
    );
}

#[test]
fn diagnostic_only_report_preserves_private_projection_initialization() {
    let report = NativePluginLoadReport::diagnostic_only("collector I/O lane is unavailable");

    assert!(report.discovered().is_empty());
    assert!(report.loaded().is_empty());
    assert_eq!(
        report.diagnostics(),
        ["collector I/O lane is unavailable".to_owned()]
    );
    assert!(report.has_failures());

    let first_projection = report.projection() as *const NativePluginLoadProjection;
    let second_projection = report.projection() as *const NativePluginLoadProjection;
    assert_eq!(first_projection, second_projection);
}

#[test]
fn report_owner_mutation_refreshes_a_frozen_projection() {
    let mut report = NativePluginLoadReport::diagnostic_only(
        "native plugin refreshable: collector I/O lane is unavailable",
    );

    assert_eq!(
        report.diagnostics_for_plugin("refreshable"),
        ["native plugin refreshable: collector I/O lane is unavailable".to_owned()]
    );

    report.push_diagnostic("native plugin refreshable: ABI negotiation failed");

    assert_eq!(
        report.diagnostics_for_plugin("refreshable"),
        [
            "native plugin refreshable: ABI negotiation failed".to_owned(),
            "native plugin refreshable: collector I/O lane is unavailable".to_owned(),
        ]
    );
}

#[test]
fn report_container_mutations_refresh_the_frozen_projection_generation() {
    let mut report = projection_fixture(1, 0, 1);
    let plugin_id = "projection_0000";

    assert!(report.projection().is_loaded(plugin_id));
    assert_eq!(report.projection().package_manifests().len(), 1);

    let loaded = report.take_loaded();
    assert_eq!(loaded.len(), 1);
    assert!(!report.projection().is_loaded(plugin_id));
    assert_eq!(report.projection().package_manifests().len(), 1);

    let discovered = report.take_discovered();
    assert_eq!(discovered.len(), 1);
    assert!(report.projection().package_manifests().is_empty());

    report.restore_discovered(discovered);
    report.push_loaded(loaded.into_iter().next().expect("one loaded plugin"));

    let refreshed = report.projection();
    assert!(refreshed.is_loaded(plugin_id));
    assert_eq!(refreshed.package_manifests().len(), 1);
    assert!(!refreshed.diagnostics_for_plugin(plugin_id).is_empty());
}

#[test]
fn native_report_resolves_plugin_shader_module_text_before_runtime_registration() {
    let package_root = std::env::temp_dir().join(format!(
        "zircon_native_shader_module_projection_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should follow the epoch")
            .as_nanos()
    ));
    let shader_path = package_root.join("shaders").join("fixture.wgsl");
    fs::create_dir_all(shader_path.parent().expect("shader parent"))
        .expect("create package shader directory");
    fs::write(
        &shader_path,
        "fn fixture_plugin_lighting() -> vec3f { return vec3f(0.1, 0.2, 0.3); }",
    )
    .expect("write plugin shader source");
    let manifest = PluginPackageManifest::new("shader_fixture", "Shader Fixture")
        .with_runtime_module(PluginModuleManifest::runtime(
            "shader_fixture.runtime",
            "zircon_plugin_shader_fixture_runtime",
        ))
        .with_shader_module("zircon_fixture::lighting", "shaders/fixture.wgsl");
    let report = NativePluginLoadReport {
        discovered: vec![NativePluginCandidate {
            plugin_id: "shader_fixture".to_string(),
            package_manifest: manifest,
            manifest_path: package_root.join("plugin.toml"),
            library_path: package_root.join("native").join("shader_fixture.dll"),
        }],
        ..Default::default()
    };

    let registrations = report.runtime_plugin_registration_reports();

    assert_eq!(registrations.len(), 1);
    assert!(registrations[0].diagnostics.is_empty());
    let sources = registrations[0].extensions.shader_module_sources();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].owner_id, "shader_fixture");
    assert_eq!(sources[0].import_path, "zircon_fixture::lighting");
    assert!(sources[0].source.contains("fixture_plugin_lighting"));
    assert_eq!(
        sources[0].content_hash,
        blake3::hash(sources[0].source.as_bytes())
            .to_hex()
            .to_string()
    );
    let _ = fs::remove_dir_all(package_root);
}

#[test]
fn native_report_defers_missing_shader_module_diagnostics_until_runtime_registration() {
    let package_root = std::env::temp_dir().join(format!(
        "zircon_native_shader_module_missing_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should follow the epoch")
            .as_nanos()
    ));
    let manifest = PluginPackageManifest::new("shader_missing", "Shader Missing")
        .with_runtime_module(PluginModuleManifest::runtime(
            "shader_missing.runtime",
            "zircon_plugin_shader_missing_runtime",
        ))
        .with_shader_module("zircon_missing::lighting", "shaders/missing.wgsl");
    let report = NativePluginLoadReport {
        discovered: vec![NativePluginCandidate {
            plugin_id: "shader_missing".to_string(),
            package_manifest: manifest,
            manifest_path: package_root.join("plugin.toml"),
            library_path: package_root.join("native").join("shader_missing.dll"),
        }],
        ..Default::default()
    };

    assert!(report.diagnostics_for_plugin("shader_missing").is_empty());
    let registrations = report.runtime_plugin_registration_reports();

    assert_eq!(registrations.len(), 1);
    assert!(registrations[0]
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("cannot resolve source")));
}

#[test]
fn native_report_rejects_shader_module_catalogs_above_the_package_budget() {
    let mut manifest = PluginPackageManifest::new("shader_budget", "Shader Budget")
        .with_runtime_module(PluginModuleManifest::runtime(
            "shader_budget.runtime",
            "zircon_plugin_shader_budget_runtime",
        ));
    for index in 0..65 {
        manifest = manifest.with_shader_module(
            format!("zircon_budget::module_{index}"),
            format!("shaders/module_{index}.wgsl"),
        );
    }
    let report = NativePluginLoadReport {
        discovered: vec![NativePluginCandidate {
            plugin_id: "shader_budget".to_string(),
            package_manifest: manifest,
            manifest_path: std::env::temp_dir()
                .join("zircon_native_shader_module_budget")
                .join("plugin.toml"),
            library_path: std::env::temp_dir()
                .join("zircon_native_shader_module_budget")
                .join("native")
                .join("shader_budget.dll"),
        }],
        ..Default::default()
    };

    let registrations = report.runtime_plugin_registration_reports();

    assert_eq!(registrations.len(), 1);
    assert!(registrations[0]
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("above the 64 module limit")));
}

#[test]
fn native_report_rejects_shader_module_sources_above_the_total_byte_budget() {
    const MODULE_BYTES: usize = 4 * 1024 * 1024;

    let package_root = std::env::temp_dir().join(format!(
        "zircon_native_shader_module_total_budget_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should follow the epoch")
            .as_nanos()
    ));
    let shader_directory = package_root.join("shaders");
    fs::create_dir_all(&shader_directory).expect("create package shader directory");
    let mut manifest = PluginPackageManifest::new("shader_total_budget", "Shader Total Budget")
        .with_runtime_module(PluginModuleManifest::runtime(
            "shader_total_budget.runtime",
            "zircon_plugin_shader_total_budget_runtime",
        ));
    for index in 0..5 {
        let relative_path = format!("shaders/module_{index}.wgsl");
        fs::write(
            shader_directory.join(format!("module_{index}.wgsl")),
            vec![b'x'; MODULE_BYTES],
        )
        .expect("write bounded fixture shader source");
        manifest = manifest.with_shader_module(
            format!("zircon_total_budget::module_{index}"),
            relative_path,
        );
    }
    let report = NativePluginLoadReport {
        discovered: vec![NativePluginCandidate {
            plugin_id: "shader_total_budget".to_string(),
            package_manifest: manifest,
            manifest_path: package_root.join("plugin.toml"),
            library_path: package_root.join("native").join("shader_total_budget.dll"),
        }],
        ..Default::default()
    };

    let registrations = report.runtime_plugin_registration_reports();

    let rejected_for_total_budget = registrations
        .first()
        .into_iter()
        .flat_map(|registration| registration.diagnostics.iter())
        .any(|diagnostic| diagnostic.contains("package shader-module budget"));
    let registration_count = registrations.len();
    let _ = fs::remove_dir_all(package_root);
    assert_eq!(registration_count, 1);
    assert!(rejected_for_total_budget);
}

#[test]
fn consuming_a_mixed_report_as_discovery_preserves_the_report() {
    let mut report = projection_fixture(1, 0, 0);
    report.push_diagnostic("native plugin projection_0000: discovery handoff diagnostic");

    let report = report
        .try_into_discovered()
        .expect_err("a report containing loaded plugins is not discovery-only");

    assert_eq!(report.discovered().len(), 1);
    assert_eq!(report.loaded().len(), 1);
    assert_eq!(
        report.diagnostics(),
        ["native plugin projection_0000: discovery handoff diagnostic".to_owned()]
    );
}

#[test]
fn native_manifest_merge_preserves_runtime_and_editor_entry_modules() {
    let mut manifests = BTreeMap::new();
    manifests.insert(
        "split_native".to_string(),
        PluginPackageManifest::new("split_native", "Split Native")
            .with_capability("runtime.plugin.split_native")
            .with_runtime_module(PluginModuleManifest::runtime(
                "split_native.runtime",
                "zircon_plugin_split_native_runtime",
            )),
    );

    merge_package_manifest(
        &mut manifests,
        PluginPackageManifest::new("split_native", "Split Native")
            .with_capability("editor.extension.split_native")
            .with_editor_module(
                PluginModuleManifest::editor(
                    "split_native.editor",
                    "zircon_plugin_split_native_editor",
                )
                .with_capabilities(["editor.extension.split_native".to_string()]),
            ),
    );

    let manifest = manifests.get("split_native").unwrap();
    assert!(manifest
        .modules
        .iter()
        .any(|module| module.kind == PluginModuleKind::Runtime));
    assert!(manifest
        .modules
        .iter()
        .any(|module| module.kind == PluginModuleKind::Editor));
    assert_eq!(manifest.modules.len(), 2);
    assert_eq!(
        manifest.capabilities,
        vec![
            "runtime.plugin.split_native".to_string(),
            "editor.extension.split_native".to_string()
        ]
    );
}

#[test]
fn native_manifest_merge_preserves_optional_feature_declarations() {
    let mut manifests = BTreeMap::new();
    manifests.insert(
        "split_native".to_string(),
        PluginPackageManifest::new("split_native", "Split Native").with_optional_feature(
            PluginFeatureBundleManifest::new(
                "split_native.runtime_tools",
                "Runtime Tools",
                "split_native",
            )
            .with_dependency(PluginFeatureDependency::primary(
                "split_native",
                "runtime.plugin.split_native",
            )),
        ),
    );

    merge_package_manifest(
        &mut manifests,
        PluginPackageManifest::new("split_native", "Split Native").with_optional_feature(
            PluginFeatureBundleManifest::new(
                "split_native.editor_tools",
                "Editor Tools",
                "split_native",
            )
            .with_dependency(PluginFeatureDependency::primary(
                "split_native",
                "runtime.plugin.split_native",
            )),
        ),
    );

    let manifest = manifests.get("split_native").unwrap();
    let feature_ids = manifest
        .optional_features
        .iter()
        .map(|feature| feature.id.as_str())
        .collect::<Vec<_>>();
    assert!(feature_ids.contains(&"split_native.runtime_tools"));
    assert!(feature_ids.contains(&"split_native.editor_tools"));
}

#[test]
fn projection_builder_uses_a_linear_package_index_and_sorts_once_at_output() {
    let source = include_str!("manifests.rs");
    let builder = source
        .split_once("struct ManifestProjectionBuilder")
        .expect("manifest projection builder should exist")
        .1
        .split_once("struct ManifestAccumulator")
        .expect("manifest accumulator should follow the builder")
        .0;

    assert!(builder.contains("manifest_indices: HashMap<String, usize>"));
    assert!(!builder.contains("manifests: BTreeMap"));
    assert_eq!(builder.matches("sort_by").count(), 1);
}

#[test]
fn native_load_report_projects_optional_features_as_runtime_feature_registrations() {
    let feature = PluginFeatureBundleManifest::new(
        "split_native.runtime_tools",
        "Runtime Tools",
        "split_native",
    )
    .with_dependency(PluginFeatureDependency::primary(
        "split_native",
        "runtime.plugin.split_native",
    ))
    .with_capability("runtime.feature.split_native.runtime_tools")
    .with_runtime_module(
        PluginModuleManifest::runtime(
            "split_native.runtime_tools.runtime",
            "zircon_plugin_split_native_runtime_tools_runtime",
        )
        .with_capabilities(["runtime.feature.split_native.runtime_tools"]),
    )
    .with_editor_module(PluginModuleManifest::editor(
        "split_native.runtime_tools.editor",
        "zircon_plugin_split_native_runtime_tools_editor",
    ));
    let report = NativePluginLoadReport {
        discovered: vec![NativePluginCandidate {
            plugin_id: "split_native".to_string(),
            package_manifest: PluginPackageManifest::new("split_native", "Split Native")
                .with_capability("runtime.plugin.split_native")
                .with_runtime_module(
                    PluginModuleManifest::runtime(
                        "split_native.runtime",
                        "zircon_plugin_split_native_runtime",
                    )
                    .with_capabilities(["runtime.plugin.split_native"]),
                )
                .with_optional_feature(feature.clone()),
            manifest_path: PathBuf::from("split_native/plugin.toml"),
            library_path: PathBuf::from("split_native/native/libsplit_native.so"),
        }],
        loaded: Vec::new(),
        diagnostics: Vec::new(),
        ..Default::default()
    };

    let feature_reports = report.runtime_plugin_feature_registration_reports();

    assert_eq!(feature_reports.len(), 1);
    assert_eq!(feature_reports[0].manifest, feature);
    assert_eq!(
        feature_reports[0]
            .project_selection
            .runtime_crate
            .as_deref(),
        Some("zircon_plugin_split_native_runtime_tools_runtime")
    );
    assert_eq!(feature_reports[0].extensions.modules().len(), 1);
    assert_eq!(
        feature_reports[0].extensions.modules()[0].name,
        "split_native.runtime_tools.runtime"
    );
}

#[test]
fn native_load_report_projects_feature_extension_packages_as_runtime_feature_registrations() {
    let feature = PluginFeatureBundleManifest::new(
        "sound.timeline_animation_track",
        "Sound Timeline Animation Track",
        "sound",
    )
    .with_dependency(PluginFeatureDependency::primary(
        "sound",
        "runtime.plugin.sound",
    ))
    .with_capability("runtime.feature.sound.timeline_animation_track")
    .with_runtime_module(
        PluginModuleManifest::runtime(
            "sound.timeline_animation_track.runtime",
            "zircon_plugin_sound_timeline_animation_runtime",
        )
        .with_capabilities(["runtime.feature.sound.timeline_animation_track"]),
    );
    let report = NativePluginLoadReport {
        discovered: vec![NativePluginCandidate {
            plugin_id: "sound_timeline_animation_track".to_string(),
            package_manifest: PluginPackageManifest::new(
                "sound_timeline_animation_track",
                "Sound Timeline Animation Track Provider",
            )
            .as_feature_extension()
            .with_feature_extension(feature.clone()),
            manifest_path: PathBuf::from("sound_timeline_animation_track/plugin.toml"),
            library_path: PathBuf::from(
                "sound_timeline_animation_track/native/libsound_timeline_animation_track.so",
            ),
        }],
        loaded: Vec::new(),
        diagnostics: Vec::new(),
        ..Default::default()
    };

    assert!(report.runtime_plugin_registration_reports().is_empty());

    let feature_reports = report.runtime_plugin_feature_registration_reports();

    assert_eq!(feature_reports.len(), 1);
    assert_eq!(feature_reports[0].manifest, feature);
    assert_eq!(
        feature_reports[0].provider_package_id.as_deref(),
        Some("sound_timeline_animation_track")
    );
    assert_eq!(
        feature_reports[0]
            .project_selection
            .provider_package_id
            .as_deref(),
        Some("sound_timeline_animation_track")
    );
}

#[test]
fn native_feature_extension_registers_package_shader_modules_with_its_runtime_feature() {
    let package_root = std::env::temp_dir().join(format!(
        "zircon_native_feature_extension_shader_module_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should follow the epoch")
            .as_nanos()
    ));
    let shader_path = package_root.join("shaders").join("feature.wgsl");
    fs::create_dir_all(shader_path.parent().expect("shader parent"))
        .expect("create package shader directory");
    fs::write(
        &shader_path,
        "fn feature_extension_lighting() -> vec3f { return vec3f(0.2, 0.4, 0.6); }",
    )
    .expect("write feature-extension shader source");
    let feature = PluginFeatureBundleManifest::new(
        "sound.timeline_animation_track",
        "Sound Timeline Animation Track",
        "sound",
    )
    .with_runtime_module(PluginModuleManifest::runtime(
        "sound.timeline_animation_track.runtime",
        "zircon_plugin_sound_timeline_animation_runtime",
    ));
    let report = NativePluginLoadReport {
        discovered: vec![NativePluginCandidate {
            plugin_id: "sound_timeline_animation_track".to_string(),
            package_manifest: PluginPackageManifest::new(
                "sound_timeline_animation_track",
                "Sound Timeline Animation Track Provider",
            )
            .as_feature_extension()
            .with_feature_extension(feature)
            .with_shader_module("zircon_fixture::feature_extension", "shaders/feature.wgsl"),
            manifest_path: package_root.join("plugin.toml"),
            library_path: package_root
                .join("native")
                .join("sound_timeline_animation_track.dll"),
        }],
        ..Default::default()
    };

    let feature_reports = report.runtime_plugin_feature_registration_reports();

    assert_eq!(feature_reports.len(), 1);
    assert!(feature_reports[0].diagnostics.is_empty());
    assert_eq!(
        feature_reports[0].extensions.shader_module_sources().len(),
        1,
        "feature-extension package shader sources must reach the shared runtime owner"
    );
    let source = &feature_reports[0].extensions.shader_module_sources()[0];
    assert_eq!(source.owner_id, "sound_timeline_animation_track");
    assert_eq!(source.import_path, "zircon_fixture::feature_extension");
    assert!(source.source.contains("feature_extension_lighting"));
    let _ = fs::remove_dir_all(package_root);
}

#[test]
fn native_optional_feature_registers_package_shader_modules_without_a_root_runtime_module() {
    let package_root = std::env::temp_dir().join(format!(
        "zircon_native_optional_feature_shader_module_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should follow the epoch")
            .as_nanos()
    ));
    let shader_path = package_root.join("shaders").join("optional_feature.wgsl");
    fs::create_dir_all(shader_path.parent().expect("shader parent"))
        .expect("create package shader directory");
    fs::write(
        &shader_path,
        "fn optional_feature_lighting() -> vec3f { return vec3f(0.1, 0.3, 0.5); }",
    )
    .expect("write optional-feature shader source");
    let feature = PluginFeatureBundleManifest::new(
        "optional_shader_fixture.runtime",
        "Optional Shader Fixture",
        "optional_shader_fixture",
    )
    .with_runtime_module(PluginModuleManifest::runtime(
        "optional_shader_fixture.runtime.module",
        "zircon_plugin_optional_shader_fixture_runtime",
    ));
    let report = NativePluginLoadReport {
        discovered: vec![NativePluginCandidate {
            plugin_id: "optional_shader_fixture".to_string(),
            package_manifest: PluginPackageManifest::new(
                "optional_shader_fixture",
                "Optional Shader Fixture",
            )
            .with_optional_feature(feature)
            .with_shader_module(
                "zircon_fixture::optional_feature",
                "shaders/optional_feature.wgsl",
            ),
            manifest_path: package_root.join("plugin.toml"),
            library_path: package_root
                .join("native")
                .join("optional_shader_fixture.dll"),
        }],
        ..Default::default()
    };

    assert!(report.runtime_plugin_registration_reports().is_empty());
    let feature_reports = report.runtime_plugin_feature_registration_reports();

    assert_eq!(feature_reports.len(), 1);
    assert!(feature_reports[0].diagnostics.is_empty());
    let sources = feature_reports[0].extensions.shader_module_sources();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].owner_id, "optional_shader_fixture");
    assert_eq!(sources[0].import_path, "zircon_fixture::optional_feature");
    assert!(sources[0].source.contains("optional_feature_lighting"));
    let _ = fs::remove_dir_all(package_root);
}

#[test]
fn projection_preserves_descriptor_runtime_editor_precedence_as_json_bytes() {
    let plugin_id = "projection_merge_contract";
    let discovered = stage_manifest(plugin_id, "Discovered", "1.0.0", "discovered");
    let descriptor = stage_manifest(plugin_id, "Descriptor", "2.0.0", "descriptor");
    let runtime = stage_manifest(plugin_id, "Runtime", "3.0.0", "runtime").with_runtime_module(
        PluginModuleManifest::runtime(
            format!("{plugin_id}.runtime"),
            format!("zircon_plugin_{plugin_id}_runtime"),
        ),
    );
    let editor = stage_manifest(plugin_id, "Editor", "4.0.0", "editor").with_editor_module(
        PluginModuleManifest::editor(
            format!("{plugin_id}.editor"),
            format!("zircon_plugin_{plugin_id}_editor"),
        ),
    );
    let report = NativePluginLoadReport {
        discovered: vec![candidate(discovered.clone())],
        loaded: vec![loaded_projection_plugin(
            plugin_id,
            descriptor.clone(),
            runtime.clone(),
            editor.clone(),
        )],
        diagnostics: Vec::new(),
        ..Default::default()
    };

    let mut expected = discovered;
    expected.version = editor.version.clone();
    expected.display_name = editor.display_name.clone();
    expected.description = editor.description.clone();
    expected.capabilities.extend([
        format!("runtime.plugin.{plugin_id}.descriptor"),
        format!("runtime.plugin.{plugin_id}.runtime"),
        format!("runtime.plugin.{plugin_id}.editor"),
    ]);
    expected.modules.extend(runtime.modules);
    expected.modules.extend(editor.modules);

    let projected_bytes =
        serde_json::to_vec(report.projection().package_manifests()).expect("projected JSON");
    let expected_bytes = serde_json::to_vec(&vec![expected]).expect("expected JSON");
    assert_eq!(projected_bytes, expected_bytes);
}

#[test]
fn projection_preserves_registration_and_diagnostic_outputs_as_json_bytes() {
    let report = projection_fixture(2, 2, 4);
    let projection = report.projection();
    let registrations = projection.runtime_plugin_registration_reports();
    let features = projection.runtime_plugin_feature_registration_reports();
    let registration_signature = registrations
        .iter()
        .map(|registration| {
            serde_json::json!({
                "package": registration.package_manifest.id,
                "manifest_modules": registration
                    .package_manifest
                    .modules
                    .iter()
                    .map(|module| module.name.as_str())
                    .collect::<Vec<_>>(),
                "selection": registration.project_selection.id,
                "runtime_crate": registration.project_selection.runtime_crate,
                "extension_modules": registration
                    .extensions
                    .modules()
                    .iter()
                    .map(|module| module.name.as_str())
                    .collect::<Vec<_>>(),
                "diagnostics": registration.diagnostics,
            })
        })
        .collect::<Vec<_>>();
    let feature_signature = features
        .iter()
        .map(|feature| {
            serde_json::json!({
                "feature": feature.manifest.id,
                "provider": feature.provider_package_id,
                "selection": feature.project_selection.id,
                "runtime_crate": feature.project_selection.runtime_crate,
                "extension_modules": feature
                    .extensions
                    .modules()
                    .iter()
                    .map(|module| module.name.as_str())
                    .collect::<Vec<_>>(),
                "diagnostics": feature.diagnostics,
            })
        })
        .collect::<Vec<_>>();
    let diagnostic_signature = projection
        .package_manifests()
        .iter()
        .map(|manifest| {
            serde_json::json!({
                "package": manifest.id,
                "all": projection.diagnostics_for_plugin(&manifest.id),
                "runtime": projection.runtime_diagnostics_for_plugin(&manifest.id),
                "editor": projection.editor_diagnostics_for_plugin(&manifest.id),
            })
        })
        .collect::<Vec<_>>();

    assert_eq!(
        serde_json::to_vec(&registration_signature).expect("registration signature JSON"),
        br#"[{"diagnostics":["native plugin projection_0000: Runtime entry diagnostic","native plugin projection_0000: diagnostic 00000","native plugin projection_0000: diagnostic 00002","native plugin projection_0000: native plugin projection_0000 runtime behavior is missing"],"extension_modules":["projection_0000.runtime"],"manifest_modules":["projection_0000.runtime"],"package":"projection_0000","runtime_crate":"zircon_plugin_projection_0000_runtime","selection":"projection_0000"},{"diagnostics":["native plugin projection_0001: Runtime entry diagnostic","native plugin projection_0001: diagnostic 00001","native plugin projection_0001: diagnostic 00003","native plugin projection_0001: native plugin projection_0001 runtime behavior is missing"],"extension_modules":["projection_0001.runtime"],"manifest_modules":["projection_0001.runtime"],"package":"projection_0001","runtime_crate":"zircon_plugin_projection_0001_runtime","selection":"projection_0001"}]"#
    );
    assert_eq!(
        serde_json::to_vec(&feature_signature).expect("feature signature JSON"),
        br#"[{"diagnostics":["native plugin projection_0000: Runtime entry diagnostic","native plugin projection_0000: diagnostic 00000","native plugin projection_0000: diagnostic 00002","native plugin projection_0000: native plugin projection_0000 runtime behavior is missing"],"extension_modules":["projection_0000.feature_00.runtime"],"feature":"projection_0000.feature_00","provider":null,"runtime_crate":"zircon_plugin_projection_0000_feature_00_runtime","selection":"projection_0000.feature_00"},{"diagnostics":["native plugin projection_0000: Runtime entry diagnostic","native plugin projection_0000: diagnostic 00000","native plugin projection_0000: diagnostic 00002","native plugin projection_0000: native plugin projection_0000 runtime behavior is missing"],"extension_modules":["projection_0000.feature_01.runtime"],"feature":"projection_0000.feature_01","provider":null,"runtime_crate":"zircon_plugin_projection_0000_feature_01_runtime","selection":"projection_0000.feature_01"},{"diagnostics":["native plugin projection_0001: Runtime entry diagnostic","native plugin projection_0001: diagnostic 00001","native plugin projection_0001: diagnostic 00003","native plugin projection_0001: native plugin projection_0001 runtime behavior is missing"],"extension_modules":["projection_0001.feature_00.runtime"],"feature":"projection_0001.feature_00","provider":null,"runtime_crate":"zircon_plugin_projection_0001_feature_00_runtime","selection":"projection_0001.feature_00"},{"diagnostics":["native plugin projection_0001: Runtime entry diagnostic","native plugin projection_0001: diagnostic 00001","native plugin projection_0001: diagnostic 00003","native plugin projection_0001: native plugin projection_0001 runtime behavior is missing"],"extension_modules":["projection_0001.feature_01.runtime"],"feature":"projection_0001.feature_01","provider":null,"runtime_crate":"zircon_plugin_projection_0001_feature_01_runtime","selection":"projection_0001.feature_01"}]"#
    );
    assert_eq!(
        serde_json::to_vec(&diagnostic_signature).expect("diagnostic signature JSON"),
        br#"[{"all":["native plugin projection_0000: Editor entry diagnostic","native plugin projection_0000: Runtime entry diagnostic","native plugin projection_0000: diagnostic 00000","native plugin projection_0000: diagnostic 00002","native plugin projection_0000: native plugin projection_0000 editor behavior is missing","native plugin projection_0000: native plugin projection_0000 runtime behavior is missing"],"editor":["native plugin projection_0000: Editor entry diagnostic","native plugin projection_0000: diagnostic 00000","native plugin projection_0000: diagnostic 00002","native plugin projection_0000: native plugin projection_0000 editor behavior is missing"],"package":"projection_0000","runtime":["native plugin projection_0000: Runtime entry diagnostic","native plugin projection_0000: diagnostic 00000","native plugin projection_0000: diagnostic 00002","native plugin projection_0000: native plugin projection_0000 runtime behavior is missing"]},{"all":["native plugin projection_0001: Editor entry diagnostic","native plugin projection_0001: Runtime entry diagnostic","native plugin projection_0001: diagnostic 00001","native plugin projection_0001: diagnostic 00003","native plugin projection_0001: native plugin projection_0001 editor behavior is missing","native plugin projection_0001: native plugin projection_0001 runtime behavior is missing"],"editor":["native plugin projection_0001: Editor entry diagnostic","native plugin projection_0001: diagnostic 00001","native plugin projection_0001: diagnostic 00003","native plugin projection_0001: native plugin projection_0001 editor behavior is missing"],"package":"projection_0001","runtime":["native plugin projection_0001: Runtime entry diagnostic","native plugin projection_0001: diagnostic 00001","native plugin projection_0001: diagnostic 00003","native plugin projection_0001: native plugin projection_0001 runtime behavior is missing"]}]"#
    );
}

#[test]
fn native_load_projection_preserves_order_and_projection_statistics() {
    for package_count in [1, 100, 1_000] {
        for feature_count in [0, 10] {
            let report = projection_fixture(package_count, feature_count, 10_000);
            let projection = report.projection();
            let registrations = projection.runtime_plugin_registration_reports();
            let features = projection.runtime_plugin_feature_registration_reports();
            let projected_diagnostic_count = projection
                .package_manifests()
                .iter()
                .map(|package| {
                    assert!(projection.is_loaded(&package.id));
                    assert!(projection.has_descriptor(&package.id));
                    projection.diagnostics_for_plugin(&package.id).len()
                })
                .sum::<usize>();
            let stats = projection.stats();

            assert_eq!(stats.projection_builds, 1);
            assert_eq!(stats.manifest_sources_scanned, package_count * 4);
            assert_eq!(stats.manifest_package_index_lookups, package_count * 4);
            assert_eq!(stats.packages_projected, package_count);
            assert_eq!(stats.features_projected, package_count * feature_count);
            assert_eq!(stats.loaded_plugins_scanned, package_count);
            assert_eq!(stats.raw_diagnostics_scanned, 10_000);
            assert_eq!(registrations.len(), package_count);
            assert_eq!(features.len(), package_count * feature_count);
            assert_eq!(projected_diagnostic_count, 10_000 + package_count * 4);
            assert!(projection.descriptor_diagnostics().is_empty());
            assert_eq!(projection.entry_diagnostics().len(), package_count * 4);

            let package_ids = projection
                .package_manifests()
                .iter()
                .map(|manifest| manifest.id.as_str())
                .collect::<Vec<_>>();
            assert!(package_ids.windows(2).all(|ids| ids[0] < ids[1]));
        }
    }
}

#[test]
fn public_load_report_getters_share_one_frozen_projection() {
    let report = projection_fixture(100, 10, 10_000);
    let first_projection = report.projection() as *const NativePluginLoadProjection;

    assert_eq!(report.package_manifests().len(), 100);
    assert_eq!(report.runtime_plugin_registration_reports().len(), 100);
    assert_eq!(
        report.runtime_plugin_feature_registration_reports().len(),
        1_000
    );
    assert!(report.entry_diagnostics().len() >= 100);
    assert!(report.descriptor_diagnostics().is_empty());
    assert!(!report
        .diagnostics_for_runtime_plugin("projection_0000")
        .is_empty());
    assert!(!report
        .diagnostics_for_editor_plugin("projection_0000")
        .is_empty());
    assert!(!report.diagnostics_for_plugin("projection_0000").is_empty());

    let projection = report.projection();
    assert_eq!(
        first_projection,
        projection as *const NativePluginLoadProjection
    );
    assert_eq!(projection.stats().projection_builds, 1);
    assert_eq!(projection.stats().manifest_sources_scanned, 400);
    assert_eq!(projection.stats().raw_diagnostics_scanned, 10_000);
}

#[test]
fn live_host_builds_one_projection_per_native_report() {
    let loading = include_str!("../native_plugin_live_host/loading.rs");
    let body = loading
        .split_once("fn load_reported_plugins_result")
        .expect("load result function")
        .1
        .split_once("pub(super) fn lock_loaded_native_plugins")
        .expect("loading helpers follow load result")
        .0;

    assert_eq!(body.matches("report.projection()").count(), 1);
    assert!(!body.contains("report.runtime_plugin_registration_reports()"));
    assert!(!body.contains("report.runtime_plugin_feature_registration_reports()"));

    let lifecycle = include_str!("../native_plugin_live_host/lifecycle.rs");
    let hot_reload = lifecycle
        .split_once("fn hot_reload_reported_plugin_result")
        .expect("hot reload result function")
        .1
        .split_once("pub(super) fn load_for_module_kind")
        .expect("module-kind loader follows hot reload")
        .0;
    assert_eq!(hot_reload.matches("report.projection()").count(), 1);
    assert!(hot_reload.contains("load_projected_report_diagnostics(&report, &projection)"));
    assert!(hot_reload.contains("projected_diagnostics_for_plugin("));
    assert!(!hot_reload.contains("report.diagnostics_for_"));
}

fn projection_fixture(
    package_count: usize,
    feature_count: usize,
    diagnostic_count: usize,
) -> NativePluginLoadReport {
    let discovered = (0..package_count)
        .rev()
        .map(|package_index| {
            let plugin_id = format!("projection_{package_index:04}");
            let package_capability = format!("runtime.plugin.{plugin_id}");
            let mut manifest = PluginPackageManifest::new(&plugin_id, &plugin_id)
                .with_capability(package_capability.clone())
                .with_runtime_module(valid_projection_runtime_module(
                    &plugin_id,
                    &package_capability,
                ));
            for feature_index in 0..feature_count {
                let feature_id = format!("{plugin_id}.feature_{feature_index:02}");
                let feature_capability = format!("runtime.feature.{feature_id}");
                manifest = manifest.with_optional_feature(
                    PluginFeatureBundleManifest::new(&feature_id, &feature_id, &plugin_id)
                        .with_dependency(PluginFeatureDependency::primary(
                            &plugin_id,
                            &package_capability,
                        ))
                        .with_capability(feature_capability.clone())
                        .with_runtime_module(
                            PluginModuleManifest::runtime(
                                format!("{feature_id}.runtime"),
                                format!(
                                    "zircon_plugin_{plugin_id}_feature_{feature_index:02}_runtime"
                                ),
                            )
                            .with_target_modes([RuntimeTargetMode::ClientRuntime])
                            .with_capabilities([feature_capability]),
                        ),
                );
            }
            NativePluginCandidate {
                plugin_id: plugin_id.clone(),
                package_manifest: manifest,
                manifest_path: PathBuf::from(format!("{plugin_id}/plugin.toml")),
                library_path: PathBuf::from(format!("{plugin_id}/native/plugin.dll")),
            }
        })
        .collect();
    let loaded = (0..package_count)
        .map(|package_index| {
            let plugin_id = format!("projection_{package_index:04}");
            loaded_projection_plugin(
                &plugin_id,
                stage_manifest(&plugin_id, &plugin_id, "0.2.0", "descriptor"),
                stage_manifest(&plugin_id, &plugin_id, "0.3.0", "runtime")
                    .with_capability(format!("runtime.plugin.{plugin_id}"))
                    .with_runtime_module(valid_projection_runtime_module(
                        &plugin_id,
                        &format!("runtime.plugin.{plugin_id}"),
                    )),
                stage_manifest(&plugin_id, &plugin_id, "0.4.0", "editor").with_editor_module(
                    PluginModuleManifest::editor(
                        format!("{plugin_id}.editor"),
                        format!("zircon_plugin_{plugin_id}_editor"),
                    ),
                ),
            )
        })
        .collect();
    let diagnostics = (0..diagnostic_count)
        .map(|diagnostic_index| {
            let plugin_index = diagnostic_index % package_count;
            format!("native plugin projection_{plugin_index:04}: diagnostic {diagnostic_index:05}")
        })
        .collect();
    NativePluginLoadReport {
        discovered,
        loaded,
        diagnostics,
        ..Default::default()
    }
}

fn valid_projection_runtime_module(plugin_id: &str, capability: &str) -> PluginModuleManifest {
    PluginModuleManifest::runtime(
        format!("{plugin_id}.runtime"),
        format!("zircon_plugin_{plugin_id}_runtime"),
    )
    .with_target_modes([RuntimeTargetMode::ClientRuntime])
    .with_capabilities([capability.to_string()])
}

fn stage_manifest(
    plugin_id: &str,
    display_name: &str,
    version: &str,
    stage: &str,
) -> PluginPackageManifest {
    let mut manifest = PluginPackageManifest::new(plugin_id, display_name)
        .with_capability(format!("runtime.plugin.{plugin_id}.{stage}"));
    manifest.version = version.to_string();
    manifest.description = format!("{stage} description");
    manifest
}

fn candidate(package_manifest: PluginPackageManifest) -> NativePluginCandidate {
    let plugin_id = package_manifest.id.clone();
    NativePluginCandidate {
        plugin_id: plugin_id.clone(),
        package_manifest,
        manifest_path: PathBuf::from(format!("{plugin_id}/plugin.toml")),
        library_path: PathBuf::from(format!("{plugin_id}/native/plugin.dll")),
    }
}

fn loaded_projection_plugin(
    plugin_id: &str,
    descriptor_manifest: PluginPackageManifest,
    runtime_manifest: PluginPackageManifest,
    editor_manifest: PluginPackageManifest,
) -> LoadedNativePlugin {
    LoadedNativePlugin {
        plugin_id: plugin_id.to_string(),
        library_path: PathBuf::from(format!("{plugin_id}.test.dll")),
        descriptor: Some(NativePluginDescriptor {
            abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            plugin_id: plugin_id.to_string(),
            package_manifest: Some(descriptor_manifest),
            runtime_entry_name: None,
            editor_entry_name: None,
            requested_capabilities: Vec::new(),
        }),
        runtime_entry_report: Some(entry_report(
            plugin_id,
            PluginModuleKind::Runtime,
            runtime_manifest,
        )),
        editor_entry_report: Some(entry_report(
            plugin_id,
            PluginModuleKind::Editor,
            editor_manifest,
        )),
        library: LoadedNativePlugin::stable_library(this_process_library()),
    }
}

fn entry_report(
    plugin_id: &str,
    module_kind: PluginModuleKind,
    package_manifest: PluginPackageManifest,
) -> NativePluginEntryReport {
    NativePluginEntryReport {
        plugin_id: plugin_id.to_string(),
        module_kind,
        package_manifest: Some(package_manifest),
        diagnostics: vec![format!("{module_kind:?} entry diagnostic")],
        negotiated_capabilities: Vec::new(),
        missing_required_capabilities: Vec::new(),
        denied_capabilities: Vec::new(),
        bridge_method_bindings: Vec::new(),
        editor_contribution_batch: None,
        behavior: None,
        behavior_validation: NativePluginBehaviorValidationReport::from_behavior(
            plugin_id,
            module_kind,
            ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            None,
        ),
    }
}

fn this_process_library() -> libloading::Library {
    #[cfg(unix)]
    {
        libloading::os::unix::Library::this().into()
    }
    #[cfg(windows)]
    {
        libloading::os::windows::Library::this()
            .expect("current process library handle should be available")
            .into()
    }
}
