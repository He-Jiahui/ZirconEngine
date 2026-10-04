use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::plugin::RuntimePluginAvailabilityReport;

use super::{materialize_zip_archive, ExportBuildPlan};
use crate::plugin::ExportGeneratedFile;

#[test]
fn reserved_authority_file_cannot_replace_an_existing_archive() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-export-zip-native-authority-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("create fixture root");
    let archive = root.join("existing.zip");
    fs::write(&archive, b"previous archive contents").expect("create existing archive");
    let plan = ExportBuildPlan {
        profile: Default::default(),
        platform_policy: Default::default(),
        enabled_runtime_plugins: Vec::new(),
        linked_runtime_crates: Vec::new(),
        linked_feature_source_count: 0,
        linked_feature_sources: Vec::new(),
        admitted_plan_proof: None,
        native_dynamic_packages: vec!["carrier".to_owned()],
        native_dynamic_package_exports: Vec::new(),
        runtime_plugin_availability: RuntimePluginAvailabilityReport::default(),
        library_embed_compile_host: None,
        source_template_build: None,
        generated_files: vec![ExportGeneratedFile {
            path: "SRC//ZIRCON_NATIVE_AUTHORITY.JSON".to_owned(),
            purpose: "reserved path collision".to_owned(),
            contents: "untrusted authority".to_owned(),
        }],
        diagnostics: Vec::new(),
        fatal_diagnostics: Vec::new(),
    };

    let error = materialize_zip_archive(&plan, &root.join("plugins"), &archive)
        .expect_err("reserved authority path must be rejected before opening the archive");
    assert!(error
        .to_string()
        .contains("generated native authority path is reserved"));
    assert_eq!(
        fs::read(&archive).expect("read existing archive"),
        b"previous archive contents"
    );
    debug_assert!(root.starts_with(std::env::temp_dir()));
    fs::remove_dir_all(root).expect("remove owned fixture directory");
}

#[test]
fn generated_native_manifest_collision_preserves_previous_archive() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-export-zip-native-path-{}-{nonce}",
        std::process::id()
    ));
    let package = root.join("plugins/carrier");
    fs::create_dir_all(&package).expect("create product package");
    fs::write(
        package.join("plugin.toml"),
        "id = \"carrier\"\nversion = \"0.1.0\"\ndisplay_name = \"Carrier\"\npackage_role = \"production\"\n",
    )
    .expect("write product manifest");
    let archive = root.join("existing.zip");
    fs::write(&archive, b"previous archive contents").expect("create previous archive");
    let plan = ExportBuildPlan {
        profile: Default::default(),
        platform_policy: Default::default(),
        enabled_runtime_plugins: Vec::new(),
        linked_runtime_crates: Vec::new(),
        linked_feature_source_count: 0,
        linked_feature_sources: Vec::new(),
        admitted_plan_proof: None,
        native_dynamic_packages: vec!["carrier".to_owned()],
        native_dynamic_package_exports: Vec::new(),
        runtime_plugin_availability: RuntimePluginAvailabilityReport::default(),
        library_embed_compile_host: None,
        source_template_build: None,
        generated_files: vec![ExportGeneratedFile {
            path: "plugins/carrier/PLUGIN.TOML".to_owned(),
            purpose: "collision with selected native package".to_owned(),
            contents: "untrusted manifest".to_owned(),
        }],
        diagnostics: Vec::new(),
        fatal_diagnostics: Vec::new(),
    };

    let error = materialize_zip_archive(&plan, &root.join("plugins"), &archive)
        .expect_err("native manifest collisions must be rejected before publication");
    assert!(
        error.to_string().contains("collides with native package"),
        "{error}"
    );
    assert_eq!(fs::read(&archive).unwrap(), b"previous archive contents");
    fs::remove_dir_all(root).expect("remove owned fixture directory");
}

#[test]
fn rejected_native_carrier_does_not_replace_an_existing_archive() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-export-zip-native-role-{}-{nonce}",
        std::process::id()
    ));
    let package = root.join("plugins").join("carrier");
    fs::create_dir_all(&package).expect("create selected package");
    fs::write(
        package.join("plugin.toml"),
        "id = \"carrier\"\nversion = \"0.1.0\"\ndisplay_name = \"Carrier\"\npackage_role = \"test_fixture\"\n",
    )
    .expect("write selected test fixture manifest");
    let archive = root.join("existing.zip");
    fs::write(&archive, "previous archive contents").expect("create existing archive");
    let plan = ExportBuildPlan {
        profile: Default::default(),
        platform_policy: Default::default(),
        enabled_runtime_plugins: Vec::new(),
        linked_runtime_crates: Vec::new(),
        linked_feature_source_count: 0,
        linked_feature_sources: Vec::new(),
        admitted_plan_proof: None,
        native_dynamic_packages: vec!["carrier".to_owned()],
        native_dynamic_package_exports: Vec::new(),
        runtime_plugin_availability: RuntimePluginAvailabilityReport::default(),
        library_embed_compile_host: None,
        source_template_build: None,
        generated_files: Vec::new(),
        diagnostics: Vec::new(),
        fatal_diagnostics: Vec::new(),
    };

    let error = materialize_zip_archive(&plan, &root.join("plugins"), &archive)
        .expect_err("selected test fixture must not be exported");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(
        fs::read(&archive).expect("read existing archive"),
        b"previous archive contents"
    );
    fs::remove_dir_all(root).expect("remove owned fixture directory");
}

#[test]
fn native_package_archive_entries_are_streamed_from_disk() {
    let source = include_str!("../archive.rs");
    let whole_file_read = ["fs::", "read(&entry.source_path)"].concat();
    assert!(
        !source.contains(&whole_file_read),
        "native package files should stream into ZipWriter without a full-file Vec"
    );
}

#[test]
fn archive_materialization_does_not_preview_then_rescan_each_package() {
    let source = include_str!("../archive.rs");
    let linear_lookup = [".find(|package| package.", "package_id == package_id)"].concat();
    let cloned_lookup = [".copied()", "\n            .cloned()"].concat();
    let write_body = source
        .split("fn write_native_package_entries")
        .nth(1)
        .and_then(|body| body.split("fn preview_native_package_entries").next())
        .expect("write-native-package body should remain available");

    assert!(!write_body.contains("preview_native_dynamic_package_copy"));
    assert!(write_body.contains("inventory.file_inventory(package_id)"));
    assert!(!source.contains(&linear_lookup));
    assert!(!write_body.contains(&cloned_lookup));
}

#[test]
fn archive_projection_preallocates_known_plan_bounds() {
    let source = include_str!("../archive.rs");

    assert!(source.contains("HashSet::with_capacity(archive_entry_capacity(plan))"));
    assert_eq!(
        source
            .matches("Vec::with_capacity(plan.generated_files.len())")
            .count(),
        2
    );
    assert_eq!(
        source
            .matches("Vec::with_capacity(plan.native_dynamic_packages.len())")
            .count(),
        2
    );
    assert_eq!(
        source
            .matches("HashSet::with_capacity(plan.native_dynamic_packages.len())")
            .count(),
        2
    );
}
