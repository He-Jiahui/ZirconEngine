use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::plugin::RuntimePluginAvailabilityReport;

use super::super::ExportGeneratedFile;
use super::ExportBuildPlan;

#[test]
fn bare_materialize_cannot_write_a_native_manifest_without_a_plugin_root() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    let output_root = std::env::temp_dir().join(format!(
        "zircon-export-native-bare-admission-{}-{nonce}",
        std::process::id()
    ));
    let manifest = output_root.join("plugins/native_plugins.toml");
    fs::create_dir_all(manifest.parent().expect("manifest parent"))
        .expect("create existing output");
    fs::write(&manifest, "previous native manifest").expect("write previous manifest");
    let mut plan = ExportBuildPlan {
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
            path: "plugins/native_plugins.toml".to_owned(),
            purpose: "native manifest".to_owned(),
            contents: "untrusted carrier selection".to_owned(),
        }],
        diagnostics: Vec::new(),
        fatal_diagnostics: Vec::new(),
    };
    plan.seal_admitted_plan();

    for result in [
        plan.materialize(&output_root).map(|_| ()),
        plan.write_generated_files(&output_root).map(|_| ()),
        plan.preview_materialize(&output_root).map(|_| ()),
    ] {
        let error = result.expect_err("native export requires a plugin-root admission gate");
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    }
    assert_eq!(
        fs::read_to_string(&manifest).expect("read previous manifest"),
        "previous native manifest"
    );
    debug_assert!(output_root.starts_with(std::env::temp_dir()));
    fs::remove_dir_all(output_root).expect("remove owned fixture directory");
}

#[test]
fn reserved_native_authority_collision_cannot_overwrite_project_files() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-export-native-authority-collision-{}-{nonce}",
        std::process::id()
    ));
    let output_root = root.join("output");
    let generated = output_root.join("src/main.rs");
    fs::create_dir_all(generated.parent().expect("generated parent"))
        .expect("create existing output");
    fs::write(&generated, "previous main source").expect("write existing source");
    let mut plan = ExportBuildPlan {
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
        generated_files: vec![
            ExportGeneratedFile {
                path: "src/main.rs".to_owned(),
                purpose: "generated runtime".to_owned(),
                contents: "replacement source".to_owned(),
            },
            ExportGeneratedFile {
                path: "SRC//ZIRCON_NATIVE_AUTHORITY.JSON".to_owned(),
                purpose: "reserved path collision".to_owned(),
                contents: "untrusted authority".to_owned(),
            },
        ],
        diagnostics: Vec::new(),
        fatal_diagnostics: Vec::new(),
    };
    plan.seal_admitted_plan();

    let error = plan
        .materialize_with_native_packages(root.join("plugins"), &output_root)
        .expect_err("generated authority collisions must fail before writing output");
    assert!(error
        .to_string()
        .contains("generated native authority path is reserved"));
    assert_eq!(
        fs::read_to_string(&generated).expect("read previous source"),
        "previous main source"
    );
    debug_assert!(root.starts_with(std::env::temp_dir()));
    fs::remove_dir_all(root).expect("remove owned fixture directory");
}

#[test]
fn directory_native_entry_collisions_preserve_existing_project_files() {
    for collision in [
        "plugins/carrier/plugin.toml",
        "PLUGINS/CARRIER/native_dynamic_package.toml",
    ] {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "zircon-export-native-directory-collision-{}-{nonce}",
            std::process::id()
        ));
        let plugin_root = root.join("plugins");
        let carrier = plugin_root.join("carrier");
        let output_root = root.join("output");
        let generated = output_root.join("src/main.rs");
        fs::create_dir_all(&carrier).expect("create selected package");
        fs::create_dir_all(generated.parent().expect("generated parent"))
            .expect("create existing output");
        fs::write(
            carrier.join("plugin.toml"),
            "id = \"carrier\"\nversion = \"0.1.0\"\ndisplay_name = \"Carrier\"\npackage_role = \"production\"\n",
        )
        .expect("write selected package manifest");
        fs::write(&generated, "previous main source").expect("write previous output");
        let mut plan = ExportBuildPlan {
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
            generated_files: vec![
                ExportGeneratedFile {
                    path: "src/main.rs".to_owned(),
                    purpose: "generated runtime".to_owned(),
                    contents: "replacement main source".to_owned(),
                },
                ExportGeneratedFile {
                    path: collision.to_owned(),
                    purpose: "native entry collision".to_owned(),
                    contents: "untrusted generated content".to_owned(),
                },
            ],
            diagnostics: Vec::new(),
            fatal_diagnostics: Vec::new(),
        };
        plan.seal_admitted_plan();

        let error = plan
            .materialize_with_native_packages(&plugin_root, &output_root)
            .expect_err("native collisions must reject before any output writes");
        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(
            fs::read_to_string(&generated).unwrap(),
            "previous main source"
        );
        assert!(!output_root.join(collision).exists());
        assert!(plan
            .preview_materialize_with_native_packages(&plugin_root, &output_root)
            .is_err());

        fs::remove_dir_all(root).expect("remove owned fixture directory");
    }
}

#[test]
fn colliding_native_package_ids_reject_directory_and_archive_before_publication() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-export-native-id-collision-{}-{nonce}",
        std::process::id()
    ));
    let plugin_root = root.join("plugins");
    let output_root = root.join("output");
    let generated = output_root.join("src/main.rs");
    let archive = root.join("previous.zip");
    fs::create_dir_all(generated.parent().expect("generated file parent"))
        .expect("create existing output");
    fs::write(&generated, "previous main source").expect("write previous output");
    fs::write(&archive, b"previous archive bytes").expect("write previous archive");
    for id in ["carrier.alpha", "carrier_alpha"] {
        let package = plugin_root.join(id);
        fs::create_dir_all(&package).expect("create selected package");
        fs::write(
            package.join("plugin.toml"),
            format!(
                "id = {id:?}\nversion = \"0.1.0\"\ndisplay_name = {id:?}\npackage_role = \"production\"\n"
            ),
        )
        .expect("write selected package manifest");
    }
    let mut plan = ExportBuildPlan {
        profile: Default::default(),
        platform_policy: Default::default(),
        enabled_runtime_plugins: Vec::new(),
        linked_runtime_crates: Vec::new(),
        linked_feature_source_count: 0,
        linked_feature_sources: Vec::new(),
        admitted_plan_proof: None,
        native_dynamic_packages: vec!["carrier.alpha".to_owned(), "carrier_alpha".to_owned()],
        native_dynamic_package_exports: Vec::new(),
        runtime_plugin_availability: RuntimePluginAvailabilityReport::default(),
        library_embed_compile_host: None,
        source_template_build: None,
        generated_files: vec![ExportGeneratedFile {
            path: "src/main.rs".to_owned(),
            purpose: "generated runtime".to_owned(),
            contents: "replacement main source".to_owned(),
        }],
        diagnostics: Vec::new(),
        fatal_diagnostics: Vec::new(),
    };
    plan.seal_admitted_plan();

    for result in [
        plan.preview_materialize_with_native_packages(&plugin_root, &output_root),
        plan.preview_zip_archive(&plugin_root, &archive),
        plan.materialize_with_native_packages(&plugin_root, &output_root),
        plan.materialize_zip_archive(&plugin_root, &archive),
    ] {
        let error = result.expect_err("two selected ids cannot share a native output path");
        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert!(error.to_string().contains("carrier_alpha"));
        assert_eq!(
            fs::read_to_string(&generated).unwrap(),
            "previous main source"
        );
        assert_eq!(fs::read(&archive).unwrap(), b"previous archive bytes");
    }

    fs::remove_dir_all(root).expect("remove owned fixture directory");
}

#[test]
fn rejected_native_carrier_cannot_overwrite_generated_project_files() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-export-native-directory-role-{}-{nonce}",
        std::process::id()
    ));
    let plugin_root = root.join("plugins");
    let carrier = plugin_root.join("carrier");
    let output_root = root.join("output");
    let generated = output_root.join("src").join("main.rs");
    fs::create_dir_all(&carrier).expect("create selected native package");
    fs::create_dir_all(generated.parent().expect("generated file parent"))
        .expect("create existing output");
    fs::write(
        carrier.join("plugin.toml"),
        "id = \"carrier\"\nversion = \"0.1.0\"\ndisplay_name = \"Carrier\"\npackage_role = \"test_fixture\"\n",
    )
    .expect("write selected test carrier manifest");
    fs::write(&generated, "previous output").expect("write existing output");
    let mut plan = ExportBuildPlan {
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
            path: "src/main.rs".to_owned(),
            purpose: "admission regression".to_owned(),
            contents: "should not replace prior output".to_owned(),
        }],
        diagnostics: Vec::new(),
        fatal_diagnostics: Vec::new(),
    };
    plan.seal_admitted_plan();

    let error = plan
        .materialize_with_native_packages(&plugin_root, &output_root)
        .expect_err("test carrier must be rejected before generated files are written");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(fs::read_to_string(&generated).unwrap(), "previous output");
    fs::remove_dir_all(root).expect("remove owned fixture directory");
}
