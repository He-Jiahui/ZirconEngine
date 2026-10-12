use super::*;
use zircon_runtime::core::framework::project::{
    ExportProfile, ExportTargetPlatform, RuntimeProfileId,
};

#[test]
fn native_aware_export_admits_linked_source_before_using_staged_native_root() {
    let source = include_str!("../../manager.rs");
    let native_aware_plan = source
        .split_once("pub fn generate_native_aware_export_plan(")
        .expect("native-aware plan entry")
        .1
        .split_once("pub fn execute_native_aware_export_build(")
        .expect("native-aware plan boundary")
        .0;
    let execution = source
        .split_once(
            "pub(crate) fn execute_native_aware_export_build_with_cancellation_and_progress",
        )
        .expect("native-aware execution entry")
        .1
        .split_once("fn append_exported_native_diagnostics(")
        .expect("native-aware execution boundary")
        .0;

    assert!(native_aware_plan.contains("from_project_manifest_with_plugin_root"));
    assert!(native_aware_plan.contains("self.plugin_directory(project_root.as_ref())"));
    assert!(execution.contains("from_project_manifest_with_plugin_root"));
    assert!(execution.contains("&plugin_root"));
    assert!(execution.contains(
        ".materialize_with_native_packages(&native_preparation.plugin_root, output_root.as_ref())"
    ));
}

#[test]
fn astra_m5_blocked_manager_returns_typed_failed_report_without_materializing_product() {
    let plan = ExportBuildPlan {
        profile: ExportProfile::new(
            "astra",
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
            RuntimeProfileId::Client2d,
        ),
        platform_policy: ExportTargetPlatform::Windows.policy(),
        enabled_runtime_plugins: vec![],
        linked_runtime_crates: vec![],
        linked_feature_source_count: 0,
        linked_feature_sources: vec![],
        admitted_plan_proof: None,
        native_dynamic_packages: vec![],
        native_dynamic_package_exports: vec![],
        runtime_plugin_availability: Default::default(),
        library_embed_compile_host: None,
        source_template_build: None,
        generated_files: vec![],
        diagnostics: vec![],
        fatal_diagnostics: vec!["required provider is absent".into()],
    };
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-astra-blocked-export-{}-{nonce}",
        std::process::id()
    ));
    let result = blocked_native_aware_export_build_report(&root, plan, &mut |_| {});
    let diagnostics = std::fs::read_to_string(root.join("export-diagnostics.txt")).unwrap();
    let _ = std::fs::remove_dir_all(root);
    let EditorExportBuildError::ReportFailed { report } = result.unwrap_err() else {
        panic!("typed failure required")
    };
    assert!(!report.invoked_cargo);
    assert!(report.generated_files.is_empty());
    assert!(report.failure_reason().is_some());
    assert!(diagnostics.contains("required provider is absent"));
}

#[test]
fn exported_native_manifest_failure_overrides_successful_cargo_receipt() {
    for target in [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::ServerRuntime,
        RuntimeTargetMode::EditorHost,
    ] {
        for manifest in [None, Some("[[plugins]\nid =")] {
            let root = native_probe_root();
            std::fs::create_dir_all(root.join("plugins")).unwrap();
            if let Some(source) = manifest {
                std::fs::write(root.join("plugins/native_plugins.toml"), source).unwrap();
            }
            let native = exported_native_load_report_for_profile(&root, target);
            let mut report = successful_cargo_report(target);
            append_exported_native_diagnostics(
                &native,
                native.projection(),
                &mut report.diagnostics,
                &mut report.fatal_diagnostics,
            );
            std::fs::remove_dir_all(&root).unwrap();

            assert!(
                native.has_failures(),
                "missing or malformed manifest must fail"
            );
            let EditorExportBuildError::ReportFailed { report } = report.into_result().unwrap_err()
            else {
                panic!("native probe failure must retain the typed report")
            };
            assert!(report.cargo_invocation.as_ref().unwrap().success);
            assert!(!report.fatal_diagnostics.is_empty());
            assert!(report
                .fatal_diagnostics
                .iter()
                .all(|diagnostic| { report.diagnostics.contains(diagnostic) }));
            assert_eq!(
                report.failure_reason(),
                Some(report.fatal_diagnostics[0].as_str())
            );
        }
    }
}

#[test]
fn valid_empty_exported_native_manifest_preserves_warning_only_success() {
    let root = native_probe_root();
    std::fs::create_dir_all(root.join("plugins")).unwrap();
    std::fs::write(root.join("plugins/native_plugins.toml"), "plugins = []\n").unwrap();
    let native = exported_native_load_report_for_profile(&root, RuntimeTargetMode::ClientRuntime);
    let mut report = successful_cargo_report(RuntimeTargetMode::ClientRuntime);
    append_exported_native_diagnostics(
        &native,
        native.projection(),
        &mut report.diagnostics,
        &mut report.fatal_diagnostics,
    );
    std::fs::remove_dir_all(&root).unwrap();

    assert!(!native.has_failures());
    assert!(report.fatal_diagnostics.is_empty());
    assert_eq!(
        report.into_result().unwrap().diagnostics,
        vec!["existing warning".to_owned()]
    );
}

#[test]
fn exported_native_inventory_probe_does_not_require_runtime_admission_authority() {
    let root = native_probe_root();
    let package = root.join("plugins/native_tool");
    let native = package.join("native");
    std::fs::create_dir_all(&native).unwrap();
    std::fs::write(
        package.join("plugin.toml"),
        r#"
id = "native_tool"
version = "0.1.0"
display_name = "Native Tool"

[[modules]]
name = "native_tool.runtime"
kind = "runtime"
crate_name = "zircon_plugin_native_tool_runtime"
"#,
    )
    .unwrap();
    std::fs::write(
        native.join(platform_library_file_name(
            "zircon_plugin_native_tool_runtime",
        )),
        b"export-inventory-fixture",
    )
    .unwrap();
    std::fs::write(
        root.join("plugins/native_plugins.toml"),
        r#"
[[plugins]]
id = "native_tool"
path = "plugins/native_tool"
manifest = "plugins/native_tool/plugin.toml"
"#,
    )
    .unwrap();

    let native = exported_native_load_report_for_profile(&root, RuntimeTargetMode::ClientRuntime);
    let mut report = successful_cargo_report(RuntimeTargetMode::ClientRuntime);
    append_exported_native_diagnostics(
        &native,
        native.projection(),
        &mut report.diagnostics,
        &mut report.fatal_diagnostics,
    );
    std::fs::remove_dir_all(&root).unwrap();

    assert!(
        !native.has_failures(),
        "export inventory validation must not require in-process artifact authority: {:?}",
        native.diagnostics()
    );
    assert!(report.fatal_diagnostics.is_empty());
    assert!(!report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("has no artifact authority")));
}

fn successful_cargo_report(target: RuntimeTargetMode) -> EditorExportBuildReport {
    EditorExportBuildReport {
        plan: ExportBuildPlan {
            profile: ExportProfile::new(
                "native-probe",
                target,
                ExportTargetPlatform::Windows,
                RuntimeProfileId::Client2d,
            ),
            platform_policy: ExportTargetPlatform::Windows.policy(),
            enabled_runtime_plugins: vec![],
            linked_runtime_crates: vec![],
            linked_feature_source_count: 0,
            linked_feature_sources: vec![],
            admitted_plan_proof: None,
            native_dynamic_packages: vec![],
            native_dynamic_package_exports: vec![],
            runtime_plugin_availability: Default::default(),
            library_embed_compile_host: None,
            source_template_build: None,
            generated_files: vec![],
            diagnostics: vec![],
            fatal_diagnostics: vec![],
        },
        invoked_cargo: true,
        cargo_invocation: Some(
            super::super::cargo_invocation::EditorExportCargoInvocation {
                command: vec!["cargo".into(), "build".into()],
                success: true,
                status_code: Some(0),
                stdout: String::new(),
                stderr: String::new(),
            },
        ),
        native_cargo_invocations: vec![],
        generated_files: vec!["plugins/native_plugins.toml".into()],
        copied_packages: vec![],
        diagnostics: vec!["existing warning".into()],
        fatal_diagnostics: vec![],
    }
}

fn native_probe_root() -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon-native-probe-{}-{nonce}",
        std::process::id()
    ))
}

fn platform_library_file_name(crate_name: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        format!("{crate_name}.dll")
    }
    #[cfg(target_os = "macos")]
    {
        format!("lib{crate_name}.dylib")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        format!("lib{crate_name}.so")
    }
}
