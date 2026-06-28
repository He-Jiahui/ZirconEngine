//! Linked first-party runtime provider catalog.
//!
//! This crate centralizes the optional Rust implementation fan-out for
//! first-party runtime plugins. `zircon_app` projects profiles and manifests,
//! while this catalog maps selected runtime plugin ids to compiled providers.

use std::collections::HashSet;

use zircon_runtime::builtin::{RuntimePluginId, RuntimeTargetMode};
use zircon_runtime::plugin::{ProjectPluginManifest, RuntimePluginRegistrationReport};

pub fn first_party_runtime_plugin_registrations_for_manifest(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> Vec<RuntimePluginRegistrationReport> {
    let mut seen = HashSet::new();
    manifest
        .enabled_for_target(target_mode)
        .filter_map(|selection| selection.runtime_id())
        .filter(|runtime_id| seen.insert(*runtime_id))
        .filter_map(first_party_registration_for_runtime_plugin)
        .collect()
}

pub fn first_party_registration_for_runtime_plugin(
    id: RuntimePluginId,
) -> Option<RuntimePluginRegistrationReport> {
    match id {
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::Ai => Some(zircon_plugin_ai_runtime::plugin_registration()),
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::Sound => Some(zircon_plugin_sound_runtime::plugin_registration()),
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::Texture => Some(zircon_plugin_texture_runtime::plugin_registration()),
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::Net => Some(zircon_plugin_net_runtime::plugin_registration()),
        #[cfg(feature = "navigation-runtime-plugin")]
        RuntimePluginId::Navigation => {
            Some(zircon_plugin_navigation_runtime::plugin_registration())
        }
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::Particles => Some(zircon_plugin_particles_runtime::plugin_registration()),
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::Animation => Some(zircon_plugin_animation_runtime::plugin_registration()),
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::Rendering => Some(zircon_plugin_rendering_runtime::plugin_registration()),
        #[cfg(feature = "base-runtime-plugins")]
        RuntimePluginId::GltfImporter => {
            Some(zircon_plugin_gltf_importer_runtime::plugin_registration())
        }
        #[cfg(feature = "advanced-render-runtime-plugins")]
        RuntimePluginId::VirtualGeometry => {
            Some(zircon_plugin_virtual_geometry_runtime::plugin_registration())
        }
        #[cfg(feature = "advanced-render-runtime-plugins")]
        RuntimePluginId::HybridGi => Some(zircon_plugin_hybrid_gi_runtime::plugin_registration()),
        #[cfg(feature = "advanced-render-runtime-plugins")]
        RuntimePluginId::Solari => Some(zircon_plugin_solari_runtime::plugin_registration()),
        #[cfg(feature = "zr-vm-language-runtime-plugin")]
        RuntimePluginId::ZrVmLanguage => {
            Some(zircon_plugin_zr_vm_language_runtime::plugin_registration())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[cfg(any(
        feature = "base-runtime-plugins",
        feature = "advanced-render-runtime-plugins",
        feature = "navigation-runtime-plugin",
        feature = "zr-vm-language-runtime-plugin"
    ))]
    use zircon_runtime::plugin::{
        ExportPackagingStrategy, PluginMaturity, PluginModuleKind, PluginPackageManifest,
    };
    use zircon_runtime::plugin::{ProjectPluginManifest, ProjectPluginSelection};

    use super::*;

    const GENERATED_MANIFEST_HEADER: &str =
        "# @generated from Rust descriptor package_manifest(); do not edit by hand.";

    const STATIC_PLUGIN_MANIFESTS: &[(&str, &str)] = &[
        ("ai", include_str!("../../ai/plugin.toml")),
        (
            "animation_graph",
            include_str!("../../animation_graph/plugin.toml"),
        ),
        ("animation", include_str!("../../animation/plugin.toml")),
        (
            "audio_importer",
            include_str!("../../audio_importer/plugin.toml"),
        ),
        (
            "asset_importer.audio",
            include_str!("../../asset_importers/audio/plugin.toml"),
        ),
        (
            "asset_importer.data",
            include_str!("../../asset_importers/data/plugin.toml"),
        ),
        (
            "asset_importer.model",
            include_str!("../../asset_importers/model/plugin.toml"),
        ),
        (
            "asset_importer.shader",
            include_str!("../../asset_importers/shader/plugin.toml"),
        ),
        (
            "asset_importer.texture",
            include_str!("../../asset_importers/texture/plugin.toml"),
        ),
        (
            "editor_build_export_desktop",
            include_str!("../../editor_build_export_desktop/plugin.toml"),
        ),
        (
            "gltf_importer",
            include_str!("../../gltf_importer/plugin.toml"),
        ),
        ("hybrid_gi", include_str!("../../hybrid_gi/plugin.toml")),
        (
            "material_editor",
            include_str!("../../material_editor/plugin.toml"),
        ),
        (
            "native_window_hosting",
            include_str!("../../native_window_hosting/plugin.toml"),
        ),
        ("navigation", include_str!("../../navigation/plugin.toml")),
        ("net", include_str!("../../net/plugin.toml")),
        (
            "obj_importer",
            include_str!("../../obj_importer/plugin.toml"),
        ),
        (
            "opus_importer",
            include_str!("../../opus_importer/plugin.toml"),
        ),
        ("particles", include_str!("../../particles/plugin.toml")),
        ("physics", include_str!("../../physics/plugin.toml")),
        (
            "plugin_sdk_examples",
            include_str!("../../plugin_sdk_examples/plugin.toml"),
        ),
        (
            "prefab_tools",
            include_str!("../../prefab_tools/plugin.toml"),
        ),
        ("rendering", include_str!("../../rendering/plugin.toml")),
        (
            "runtime_diagnostics",
            include_str!("../../runtime_diagnostics/plugin.toml"),
        ),
        (
            "shader_wgsl_importer",
            include_str!("../../shader_wgsl_importer/plugin.toml"),
        ),
        ("solari", include_str!("../../solari/plugin.toml")),
        ("sound", include_str!("../../sound/plugin.toml")),
        ("terrain", include_str!("../../terrain/plugin.toml")),
        (
            "texture_importer",
            include_str!("../../texture_importer/plugin.toml"),
        ),
        ("texture", include_str!("../../texture/plugin.toml")),
        ("tilemap_2d", include_str!("../../tilemap_2d/plugin.toml")),
        (
            "timeline_sequence",
            include_str!("../../timeline_sequence/plugin.toml"),
        ),
        (
            "ui_asset_authoring",
            include_str!("../../ui_asset_authoring/plugin.toml"),
        ),
        (
            "ui_document_importer",
            include_str!("../../ui_document_importer/plugin.toml"),
        ),
        (
            "virtual_geometry",
            include_str!("../../virtual_geometry/plugin.toml"),
        ),
        (
            "zr_vm_language",
            include_str!("../../zr_vm_language/plugin.toml"),
        ),
    ];

    #[derive(Debug, Default)]
    struct ParsedPluginManifest {
        id: String,
        sdk_api_version: String,
        display_name: String,
        description: String,
        version: String,
        category: String,
        supported_targets: Vec<String>,
        supported_platforms: Vec<String>,
        capabilities: Vec<String>,
        maturity: String,
        default_packaging: Vec<String>,
        modules: Vec<ParsedModuleManifest>,
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    struct ParsedModuleManifest {
        name: String,
        kind: String,
        crate_name: String,
        target_modes: Vec<String>,
        capabilities: Vec<String>,
        system_sets: Vec<String>,
        system_anchors: Vec<String>,
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Section {
        Root,
        Module,
        Other,
    }

    #[cfg(not(any(
        feature = "base-runtime-plugins",
        feature = "advanced-render-runtime-plugins",
        feature = "navigation-runtime-plugin",
        feature = "zr-vm-language-runtime-plugin"
    )))]
    #[test]
    fn catalog_without_provider_features_returns_no_registrations() {
        let manifest = ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::Sound,
                true,
                true,
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])],
        };

        assert!(first_party_runtime_plugin_registrations_for_manifest(
            RuntimeTargetMode::ClientRuntime,
            &manifest
        )
        .is_empty());
    }

    #[test]
    fn plugins_12_static_plugin_manifest_is_generated() {
        let mut missing_generated_headers = Vec::new();
        for (package_id, manifest_toml) in STATIC_PLUGIN_MANIFESTS {
            if !manifest_toml.starts_with(GENERATED_MANIFEST_HEADER) {
                missing_generated_headers.push(*package_id);
            }
            let decoded = parse_manifest(package_id, manifest_toml);
            assert_eq!(
                decoded.id, *package_id,
                "{package_id} plugin.toml id drifted"
            );
        }
        assert!(
            missing_generated_headers.is_empty(),
            "static plugin.toml files missing @generated header: {missing_generated_headers:?}"
        );

        #[cfg(any(
            feature = "base-runtime-plugins",
            feature = "advanced-render-runtime-plugins",
            feature = "navigation-runtime-plugin",
            feature = "zr-vm-language-runtime-plugin"
        ))]
        assert_runtime_descriptor_manifests_match_generated_static_manifests();
        assert_native_dynamic_fixture_keeps_single_hand_written_manifest();
    }

    #[cfg(any(
        feature = "base-runtime-plugins",
        feature = "advanced-render-runtime-plugins",
        feature = "navigation-runtime-plugin",
        feature = "zr-vm-language-runtime-plugin"
    ))]
    #[test]
    fn plugins_12_feature_enabled_runtime_descriptor_manifest_parity() {
        assert_runtime_descriptor_manifests_match_generated_static_manifests();
    }

    #[test]
    fn plugins_12_manifest_schema_uniform() {
        let manifests = STATIC_PLUGIN_MANIFESTS
            .iter()
            .copied()
            .chain(std::iter::once((
                "native_dynamic_fixture",
                include_str!("../../native_dynamic_fixture/plugin.toml"),
            )));
        let mut violations = Vec::new();
        for (package_id, manifest_toml) in manifests {
            let manifest = parse_manifest(package_id, manifest_toml);
            collect_manifest_schema_violations(package_id, &manifest, &mut violations);
        }

        assert!(
            violations.is_empty(),
            "plugin manifest schema violations: {violations:?}"
        );
    }

    #[test]
    fn plugins_12_manifest_schema_uniform_audit_report_is_clean() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .and_then(std::path::Path::parent)
            .expect("first_party_runtime_catalog should live under zircon_plugins");
        let audit_script = repo_root.join("tools").join("audit_plugin_structure.py");
        assert!(
            audit_script.exists(),
            "missing plugin structure audit script at {}",
            audit_script.display()
        );

        let output = run_python_audit(&audit_script, repo_root);
        assert!(
            output.status.success(),
            "plugin structure audit failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).expect("audit JSON must be UTF-8");
        for expected_anchor in [
            "\"m1_gate_status\": \"classified-and-clear\"",
            "\"expected_manifest_count\": 37",
            "\"manifest_count\": 37",
            "\"generated_manifest_count\": 36",
            "\"missing_plugin_toml\": 0",
            "\"manifest_schema_violations\": 0",
            "\"generated_manifest_header_violations\": 0",
            "\"m3_t2_runtime_registration_builder_status\": \"runtime-registration-builder-clean\"",
            "\"runtime_registration_builder_violation_count\": 0",
            "\"runtime_registration_builder_roots\": [",
            "\"animation\"",
            "\"physics\"",
            "\"net\"",
        ] {
            assert!(
                stdout.contains(expected_anchor),
                "plugin structure audit JSON missing `{expected_anchor}`\n{stdout}"
            );
        }
    }

    #[test]
    fn plugins_12_capability_single_source_conformance() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .and_then(std::path::Path::parent)
            .expect("first_party_runtime_catalog should live under zircon_plugins");
        let audit_script = repo_root.join("tools").join("audit_plugin_structure.py");

        let output = run_python_audit(&audit_script, repo_root);
        assert!(
            output.status.success(),
            "plugin structure audit failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).expect("audit JSON must be UTF-8");
        for expected_anchor in [
            "\"m4_runtime_capability_gate_status\": \"runtime-capability-single-source-clean\"",
            "\"audited_runtime_root_count\": 15",
            "\"capability_audited_runtime_root_count\": 15",
            "\"capability_source_mismatches\": 0",
            "\"missing_capability_owner_files\": 0",
            "\"missing_runtime_capability_exports\": 0",
            "\"root_capability_mismatches\": 0",
            "\"module_capability_mismatches\": 0",
            "\"lib_capability_literal_sites\": 0",
            "\"sdk_builder_mirror_violations\": 0",
            "\"m4_t2_builder_mirror_gate_status\": \"sdk-builder-mirror-clean\"",
            "\"editor_runtime_mirror_root_count\": 3",
            "\"editor_runtime_mirror_violations\": 0",
            "\"d9_editor_runtime_mirror_gate_status\": \"editor-runtime-mirror-clean\"",
        ] {
            assert!(
                stdout.contains(expected_anchor),
                "plugin capability audit JSON missing `{expected_anchor}`\n{stdout}"
            );
        }
    }

    #[test]
    fn plugins_13_dist_dependency_boundary_clean() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .and_then(std::path::Path::parent)
            .expect("first_party_runtime_catalog should live under zircon_plugins");
        let audit_script = repo_root.join("tools").join("audit_plugin_structure.py");

        let output = run_python_audit(&audit_script, repo_root);
        assert!(
            output.status.success(),
            "plugin structure audit failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).expect("audit JSON must be UTF-8");
        for expected_anchor in [
            "\"dist_capable_plugin_count\": 1",
            "\"dist_capable_plugins\": [",
            "\"native_dynamic_fixture\"",
            "\"distribution_section_violations\": 0",
            "\"dist_dependency_boundary_violations\": 0",
            "\"m1_dist_dependency_boundary_gate_status\": \"dist-boundary-clean\"",
        ] {
            assert!(
                stdout.contains(expected_anchor),
                "plugin standalone distribution audit JSON missing `{expected_anchor}`\n{stdout}"
            );
        }
    }

    #[test]
    fn plugins_12_crate_skeleton_conformance() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .and_then(std::path::Path::parent)
            .expect("first_party_runtime_catalog should live under zircon_plugins");
        let audit_script = repo_root.join("tools").join("audit_plugin_structure.py");

        let output = run_python_audit(&audit_script, repo_root);
        assert!(
            output.status.success(),
            "plugin structure audit failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).expect("audit JSON must be UTF-8");
        for expected_anchor in [
            "\"sample_conformance_status\": \"sample-clean\"",
            "\"sample_expected_count\": 1",
            "\"sample_conforming_count\": 1",
            "\"sample_violation_count\": 0",
            "\"sample_workspace_dependency_status\": \"sample-workspace-deps-clean\"",
            "\"core_workspace_dependency_status\": \"core-workspace-deps-clean\"",
            "\"core_workspace_dependency_count\": 117",
            "\"core_workspace_dependency_violation_count\": 0",
            "\"plugin_sdk_examples\"",
            "\"m2_gate_status\": \"sample-clean-migration-debt-clear\"",
            "\"migration_debt_count\"",
        ] {
            assert!(
                stdout.contains(expected_anchor),
                "plugin skeleton audit JSON missing `{expected_anchor}`\n{stdout}"
            );
        }
    }

    fn run_python_audit(
        audit_script: &std::path::Path,
        repo_root: &std::path::Path,
    ) -> std::process::Output {
        for python in ["python", "python3"] {
            if let Ok(output) = std::process::Command::new(python)
                .arg(audit_script)
                .arg("--json")
                .arg("--repo-root")
                .arg(repo_root)
                .output()
            {
                return output;
            }
        }
        panic!("failed to launch python or python3 for plugin structure audit");
    }

    #[cfg(any(
        feature = "base-runtime-plugins",
        feature = "advanced-render-runtime-plugins",
        feature = "navigation-runtime-plugin",
        feature = "zr-vm-language-runtime-plugin"
    ))]
    fn assert_runtime_descriptor_manifests_match_generated_static_manifests() {
        #[cfg(feature = "base-runtime-plugins")]
        {
            assert_runtime_manifest_matches_descriptor(
                "ai",
                include_str!("../../ai/plugin.toml"),
                zircon_plugin_ai_runtime::package_manifest(),
            );
            assert_runtime_manifest_matches_descriptor(
                "animation",
                include_str!("../../animation/plugin.toml"),
                zircon_plugin_animation_runtime::package_manifest(),
            );
            assert_runtime_manifest_matches_descriptor(
                "gltf_importer",
                include_str!("../../gltf_importer/plugin.toml"),
                zircon_plugin_gltf_importer_runtime::package_manifest(),
            );
            assert_runtime_manifest_matches_descriptor(
                "net",
                include_str!("../../net/plugin.toml"),
                zircon_plugin_net_runtime::package_manifest(),
            );
            assert_runtime_manifest_matches_descriptor(
                "particles",
                include_str!("../../particles/plugin.toml"),
                zircon_plugin_particles_runtime::package_manifest(),
            );
            assert_runtime_manifest_matches_descriptor(
                "rendering",
                include_str!("../../rendering/plugin.toml"),
                zircon_plugin_rendering_runtime::package_manifest(),
            );
            assert_runtime_manifest_matches_descriptor(
                "sound",
                include_str!("../../sound/plugin.toml"),
                zircon_plugin_sound_runtime::package_manifest(),
            );
            assert_runtime_manifest_matches_descriptor(
                "texture",
                include_str!("../../texture/plugin.toml"),
                zircon_plugin_texture_runtime::package_manifest(),
            );
        }
        #[cfg(feature = "advanced-render-runtime-plugins")]
        {
            assert_runtime_manifest_matches_descriptor(
                "hybrid_gi",
                include_str!("../../hybrid_gi/plugin.toml"),
                zircon_plugin_hybrid_gi_runtime::package_manifest(),
            );
            assert_runtime_manifest_matches_descriptor(
                "solari",
                include_str!("../../solari/plugin.toml"),
                zircon_plugin_solari_runtime::package_manifest(),
            );
            assert_runtime_manifest_matches_descriptor(
                "virtual_geometry",
                include_str!("../../virtual_geometry/plugin.toml"),
                zircon_plugin_virtual_geometry_runtime::package_manifest(),
            );
        }
        #[cfg(feature = "navigation-runtime-plugin")]
        assert_runtime_manifest_matches_descriptor(
            "navigation",
            include_str!("../../navigation/plugin.toml"),
            zircon_plugin_navigation_runtime::package_manifest(),
        );
        #[cfg(feature = "zr-vm-language-runtime-plugin")]
        assert_runtime_manifest_matches_descriptor(
            "zr_vm_language",
            include_str!("../../zr_vm_language/plugin.toml"),
            zircon_plugin_zr_vm_language_runtime::package_manifest(),
        );
    }

    #[cfg(any(
        feature = "base-runtime-plugins",
        feature = "advanced-render-runtime-plugins",
        feature = "navigation-runtime-plugin",
        feature = "zr-vm-language-runtime-plugin"
    ))]
    fn assert_runtime_manifest_matches_descriptor(
        package_id: &str,
        manifest_toml: &str,
        descriptor_manifest: PluginPackageManifest,
    ) {
        let actual = parse_manifest(package_id, manifest_toml);
        assert_eq!(actual.id, descriptor_manifest.id, "{package_id} id drift");
        assert_eq!(
            actual.display_name, descriptor_manifest.display_name,
            "{package_id} display name drift"
        );
        assert_eq!(
            actual.category, descriptor_manifest.category,
            "{package_id} category drift"
        );
        assert_eq!(
            actual.sdk_api_version, descriptor_manifest.sdk_api_version,
            "{package_id} SDK API version drift"
        );
        assert_eq!(
            actual.supported_targets,
            target_mode_names(&descriptor_manifest.supported_targets),
            "{package_id} target-mode drift"
        );
        assert_eq!(
            actual.supported_platforms,
            platform_names(&descriptor_manifest.supported_platforms),
            "{package_id} supported-platform drift"
        );
        assert_eq!(
            actual.capabilities, descriptor_manifest.capabilities,
            "{package_id} capability drift"
        );
        assert_eq!(
            actual.maturity,
            maturity_name(descriptor_manifest.maturity),
            "{package_id} maturity drift"
        );
        assert_eq!(
            actual.default_packaging,
            packaging_names(&descriptor_manifest.default_packaging),
            "{package_id} default packaging drift"
        );
        for expected_module in descriptor_manifest
            .modules
            .iter()
            .filter(|module| module.kind == PluginModuleKind::Runtime)
        {
            let actual_module = actual
                .modules
                .iter()
                .find(|module| module.kind == "runtime" && module.name == expected_module.name)
                .unwrap_or_else(|| {
                    panic!(
                        "{package_id} missing runtime module {}",
                        expected_module.name
                    )
                });
            let expected_module = ParsedModuleManifest {
                name: expected_module.name.clone(),
                kind: "runtime".to_string(),
                crate_name: expected_module.crate_name.clone(),
                target_modes: target_mode_names(&expected_module.target_modes),
                capabilities: expected_module.capabilities.clone(),
                system_sets: expected_module.system_sets.clone(),
                system_anchors: expected_module.system_anchors.clone(),
            };
            assert_eq!(
                *actual_module, expected_module,
                "{package_id} runtime module drift"
            );
        }
    }

    fn parse_manifest(package_id: &str, manifest_toml: &str) -> ParsedPluginManifest {
        let mut manifest = ParsedPluginManifest::default();
        let mut section = Section::Root;
        let mut module = None;
        let mut pending_array: Option<(String, String)> = None;

        for raw_line in manifest_toml.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, mut value)) = pending_array.take() {
                value.push(' ');
                value.push_str(line);
                if line.ends_with(']') {
                    parse_manifest_field(
                        package_id,
                        section,
                        &mut manifest,
                        &mut module,
                        &key,
                        &value,
                    );
                } else {
                    pending_array = Some((key, value));
                }
                continue;
            }
            if line == "[[modules]]" {
                if let Some(module) = module.take() {
                    manifest.modules.push(module);
                }
                module = Some(ParsedModuleManifest::default());
                section = Section::Module;
                continue;
            }
            if line.starts_with('[') {
                if let Some(module) = module.take() {
                    manifest.modules.push(module);
                }
                section = Section::Other;
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            if value.starts_with('[') && !value.ends_with(']') {
                pending_array = Some((key.to_string(), value.to_string()));
                continue;
            }
            parse_manifest_field(package_id, section, &mut manifest, &mut module, key, value);
        }
        if let Some(module) = module {
            manifest.modules.push(module);
        }
        manifest
    }

    fn parse_manifest_field(
        package_id: &str,
        section: Section,
        manifest: &mut ParsedPluginManifest,
        module: &mut Option<ParsedModuleManifest>,
        key: &str,
        value: &str,
    ) {
        match section {
            Section::Root => parse_root_manifest_field(manifest, key, value),
            Section::Module => {
                let module = module
                    .as_mut()
                    .unwrap_or_else(|| panic!("{package_id} parser entered module without row"));
                parse_module_manifest_field(module, key, value);
            }
            Section::Other => {}
        }
    }

    fn parse_root_manifest_field(manifest: &mut ParsedPluginManifest, key: &str, value: &str) {
        match key {
            "id" => manifest.id = parse_string(value),
            "version" => manifest.version = parse_string(value),
            "sdk_api_version" => manifest.sdk_api_version = parse_string(value),
            "display_name" => manifest.display_name = parse_string(value),
            "description" => manifest.description = parse_string(value),
            "category" => manifest.category = parse_string(value),
            "supported_targets" => manifest.supported_targets = parse_string_array(value),
            "supported_platforms" => {
                manifest.supported_platforms = parse_string_array(value);
            }
            "capabilities" => manifest.capabilities = parse_string_array(value),
            "maturity" => manifest.maturity = parse_string(value),
            "default_packaging" => manifest.default_packaging = parse_string_array(value),
            _ => {}
        }
    }

    fn collect_manifest_schema_violations(
        package_id: &str,
        manifest: &ParsedPluginManifest,
        violations: &mut Vec<String>,
    ) {
        for (field, value) in [
            ("id", manifest.id.as_str()),
            ("version", manifest.version.as_str()),
            ("sdk_api_version", manifest.sdk_api_version.as_str()),
            ("display_name", manifest.display_name.as_str()),
            ("description", manifest.description.as_str()),
            ("category", manifest.category.as_str()),
            ("maturity", manifest.maturity.as_str()),
        ] {
            if value.trim().is_empty() {
                violations.push(format!("{package_id}: missing {field}"));
            }
        }
        for (field, values) in [
            ("supported_targets", &manifest.supported_targets),
            ("supported_platforms", &manifest.supported_platforms),
            ("capabilities", &manifest.capabilities),
            ("default_packaging", &manifest.default_packaging),
        ] {
            if values.is_empty() {
                violations.push(format!("{package_id}: missing {field}"));
            }
        }
        if manifest.modules.is_empty() {
            violations.push(format!("{package_id}: missing modules"));
        }
        for module in &manifest.modules {
            for (field, value) in [
                ("modules.name", module.name.as_str()),
                ("modules.kind", module.kind.as_str()),
                ("modules.crate_name", module.crate_name.as_str()),
            ] {
                if value.trim().is_empty() {
                    violations.push(format!("{package_id}: missing {field}"));
                }
            }
            for (field, values) in [
                ("modules.target_modes", &module.target_modes),
                ("modules.capabilities", &module.capabilities),
            ] {
                if values.is_empty() {
                    violations.push(format!("{package_id}: missing {field}"));
                }
            }
        }
    }

    fn parse_module_manifest_field(module: &mut ParsedModuleManifest, key: &str, value: &str) {
        match key {
            "name" => module.name = parse_string(value),
            "kind" => module.kind = parse_string(value),
            "crate_name" => module.crate_name = parse_string(value),
            "target_modes" => module.target_modes = parse_string_array(value),
            "capabilities" => module.capabilities = parse_string_array(value),
            "system_sets" => module.system_sets = parse_string_array(value),
            "system_anchors" => module.system_anchors = parse_string_array(value),
            _ => {}
        }
    }

    fn parse_string(value: &str) -> String {
        value
            .trim()
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .unwrap_or_else(|| panic!("expected TOML string value, got `{value}`"))
            .to_string()
    }

    fn parse_string_array(value: &str) -> Vec<String> {
        let value = value
            .trim()
            .strip_prefix('[')
            .and_then(|value| value.strip_suffix(']'))
            .unwrap_or_else(|| panic!("expected TOML string array value, got `{value}`"));
        let mut entries = Vec::new();
        let mut entry = String::new();
        let mut in_string = false;
        for character in value.chars() {
            match character {
                '"' if in_string => {
                    entries.push(entry.clone());
                    entry.clear();
                    in_string = false;
                }
                '"' => in_string = true,
                _ if in_string => entry.push(character),
                _ => {}
            }
        }
        entries
    }

    #[cfg(any(
        feature = "base-runtime-plugins",
        feature = "advanced-render-runtime-plugins",
        feature = "navigation-runtime-plugin",
        feature = "zr-vm-language-runtime-plugin"
    ))]
    fn target_mode_names(target_modes: &[RuntimeTargetMode]) -> Vec<String> {
        target_modes
            .iter()
            .map(|target_mode| {
                match target_mode {
                    RuntimeTargetMode::ClientRuntime => "client_runtime",
                    RuntimeTargetMode::ServerRuntime => "server_runtime",
                    RuntimeTargetMode::EditorHost => "editor_host",
                }
                .to_string()
            })
            .collect()
    }

    #[cfg(any(
        feature = "base-runtime-plugins",
        feature = "advanced-render-runtime-plugins",
        feature = "navigation-runtime-plugin",
        feature = "zr-vm-language-runtime-plugin"
    ))]
    fn maturity_name(maturity: PluginMaturity) -> String {
        match maturity {
            PluginMaturity::Core => "core",
            PluginMaturity::Stable => "stable",
            PluginMaturity::Beta => "beta",
            PluginMaturity::Experimental => "experimental",
            PluginMaturity::Externalized => "externalized",
            PluginMaturity::Stub => "stub",
            PluginMaturity::Deprecated => "deprecated",
        }
        .to_string()
    }

    #[cfg(any(
        feature = "base-runtime-plugins",
        feature = "advanced-render-runtime-plugins",
        feature = "navigation-runtime-plugin",
        feature = "zr-vm-language-runtime-plugin"
    ))]
    fn platform_names(platforms: &[zircon_runtime::plugin::ExportTargetPlatform]) -> Vec<String> {
        platforms
            .iter()
            .map(|platform| {
                match platform {
                    zircon_runtime::plugin::ExportTargetPlatform::Windows => "windows",
                    zircon_runtime::plugin::ExportTargetPlatform::Linux => "linux",
                    zircon_runtime::plugin::ExportTargetPlatform::Macos => "macos",
                    zircon_runtime::plugin::ExportTargetPlatform::Android => "android",
                    zircon_runtime::plugin::ExportTargetPlatform::Ios => "ios",
                    zircon_runtime::plugin::ExportTargetPlatform::WebGpu => "web_gpu",
                    zircon_runtime::plugin::ExportTargetPlatform::Wasm => "wasm",
                    zircon_runtime::plugin::ExportTargetPlatform::Headless => "headless",
                }
                .to_string()
            })
            .collect()
    }

    #[cfg(any(
        feature = "base-runtime-plugins",
        feature = "advanced-render-runtime-plugins",
        feature = "navigation-runtime-plugin",
        feature = "zr-vm-language-runtime-plugin"
    ))]
    fn packaging_names(packaging: &[ExportPackagingStrategy]) -> Vec<String> {
        packaging
            .iter()
            .map(|strategy| {
                match strategy {
                    ExportPackagingStrategy::SourceTemplate => "source_template",
                    ExportPackagingStrategy::LibraryEmbed => "library_embed",
                    ExportPackagingStrategy::NativeDynamic => "native_dynamic",
                }
                .to_string()
            })
            .collect()
    }

    fn assert_native_dynamic_fixture_keeps_single_hand_written_manifest() {
        let native_manifest = include_str!("../../native_dynamic_fixture/plugin.toml");
        assert!(
            !native_manifest.starts_with(GENERATED_MANIFEST_HEADER),
            "native_dynamic_fixture/plugin.toml must stay hand-written until native SDK generation"
        );

        let native_source = include_str!("../../native_dynamic_fixture/native/src/lib.rs");
        assert!(
            native_source.contains("concat!(include_str!(\"../../plugin.toml\"), \"\\0\")"),
            "native fixture must embed the hand-written root plugin.toml with include_str!"
        );
        assert!(
            !native_source.contains("r#\"id = \"native_dynamic_fixture\""),
            "native fixture must not carry a second inline plugin.toml copy"
        );
    }
}
