mod generated_manifest;
mod runtime_projection;

use generated_manifest::STATIC_PLUGIN_MANIFESTS;

#[cfg(any(
    feature = "base-runtime-plugins",
    feature = "advanced-render-runtime-plugins",
    feature = "navigation-runtime-plugin",
    feature = "zr-vm-language-runtime-plugin"
))]
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
#[cfg(not(any(
    feature = "base-runtime-plugins",
    feature = "advanced-render-runtime-plugins",
    feature = "navigation-runtime-plugin",
    feature = "zr-vm-language-runtime-plugin"
)))]
use zircon_runtime::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};
use zircon_runtime::plugin::{PluginMaturity, PluginModuleKind, PluginPackageManifest};

use super::*;

#[cfg(any(
    feature = "base-runtime-plugins",
    feature = "advanced-render-runtime-plugins",
    feature = "navigation-runtime-plugin",
    feature = "zr-vm-language-runtime-plugin"
))]
mod provider_snapshot;

const GENERATED_MANIFEST_HEADER: &str =
    "# @generated from Rust PluginDeclaration; do not edit by hand.";

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
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, true)
                .with_target_modes([RuntimeTargetMode::ClientRuntime]),
        ],
    };

    assert!(
        first_party_runtime_plugin_registrations_for_manifest(
            RuntimeTargetMode::ClientRuntime,
            &manifest
        )
        .is_empty()
    );
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
        "\"expected_manifest_count\": 38",
        "\"manifest_count\": 38",
        "\"generated_manifest_count\": 38",
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
        "\"editor_runtime_mirror_root_count\": 4",
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
        "\"dist_build_matrix_count\": 40",
        "\"dist_capable_plugin_count\": 40",
        "\"dist_capable_plugins\": [",
        "\"ai\"",
        "\"native_dynamic_fixture\"",
        "\"zr_vm_language\"",
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
        "\"core_workspace_dependency_count\": 150",
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
    static AUDIT_OUTPUT: std::sync::OnceLock<std::process::Output> = std::sync::OnceLock::new();

    AUDIT_OUTPUT
        .get_or_init(|| {
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
        })
        .clone()
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
fn platform_names(
    platforms: &[zircon_runtime::core::framework::project::ExportTargetPlatform],
) -> Vec<String> {
    platforms
        .iter()
        .map(|platform| {
            match platform {
                zircon_runtime::core::framework::project::ExportTargetPlatform::Windows => {
                    "windows"
                }
                zircon_runtime::core::framework::project::ExportTargetPlatform::Linux => "linux",
                zircon_runtime::core::framework::project::ExportTargetPlatform::Macos => "macos",
                zircon_runtime::core::framework::project::ExportTargetPlatform::Android => {
                    "android"
                }
                zircon_runtime::core::framework::project::ExportTargetPlatform::Ios => "ios",
                zircon_runtime::core::framework::project::ExportTargetPlatform::WebGpu => "web_gpu",
                zircon_runtime::core::framework::project::ExportTargetPlatform::Wasm => "wasm",
                zircon_runtime::core::framework::project::ExportTargetPlatform::Headless => {
                    "headless"
                }
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

fn assert_native_dynamic_fixture_manifest_is_sdk_declared() {
    let native_manifest = include_str!("../../native_dynamic_fixture/plugin.toml");
    assert!(
        native_manifest.starts_with(GENERATED_MANIFEST_HEADER),
        "native_dynamic_fixture/plugin.toml must be a generated PluginDeclaration snapshot"
    );

    let native_source = include_str!("../../native_dynamic_fixture/native/src/lib.rs");
    assert!(
        native_source.contains("zircon_plugin_sdk::declare_plugin!"),
        "native fixture must declare package metadata through PluginDeclaration"
    );
    assert!(
        native_source.contains(
            "const PLUGIN_MANIFEST: &str = concat!(include_str!(\"../../plugin.toml\"), \"\\0\");"
        ),
        "native fixture ABI must embed the generated manifest snapshot"
    );
    assert!(
        !native_source.contains("native_plugin_manifest_v3!"),
        "native fixture must not restore the retired manifest macro"
    );
}
