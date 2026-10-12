use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::plugin::native::discovery::discover_native_plugins;
use zircon_runtime::plugin::{ExportBuildPlan, PluginPackageRole};

use super::{prepare_native_dynamic_packages_with_cancellation, NATIVE_DYNAMIC_CACHE_ROOT};
use crate::core::jobs::{test_job_system, CancellationToken};
use crate::ui::host::native_dynamic_export_preparation::NativeDynamicPreparationError;

#[test]
fn test_fixture_native_package_is_rejected_before_staging_or_building() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-native-preparation-admission-{}-{stamp}",
        std::process::id()
    ));
    let package_root = root.join("plugins/carrier");
    let output_root = root.join("output");
    let cache_root = output_root.join(NATIVE_DYNAMIC_CACHE_ROOT);
    let previous_staging = cache_root.join("packages/previous/marker.txt");
    fs::create_dir_all(package_root.join("native")).expect("create fixture package");
    fs::create_dir_all(previous_staging.parent().expect("marker parent"))
        .expect("create previous staging");
    fs::write(&previous_staging, b"keep previous staging").expect("write previous staging");
    fs::write(
        package_root.join("plugin.toml"),
        "id = \"carrier\"\nversion = \"0.1.0\"\ndisplay_name = \"Carrier\"\npackage_role = \"test_fixture\"\n\n[[modules]]\nname = \"carrier.runtime\"\nkind = \"runtime\"\ncrate_name = \"carrier_native\"\n",
    )
    .expect("write test fixture manifest");
    fs::write(
        package_root.join("native/Cargo.toml"),
        "[package]\nname = \"carrier_native\"\nversion = \"0.1.0\"\n",
    )
    .expect("write untrusted native build manifest");

    let discovered = discover_native_plugins(root.join("plugins"));
    assert_eq!(
        discovered.discovered().len(),
        1,
        "fixture must be discoverable"
    );
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
        runtime_plugin_availability: Default::default(),
        library_embed_compile_host: None,
        source_template_build: None,
        generated_files: Vec::new(),
        diagnostics: Vec::new(),
        fatal_diagnostics: Vec::new(),
    };
    let error = prepare_native_dynamic_packages_with_cancellation(
        &output_root,
        &plan,
        &discovered,
        &test_job_system(),
        &CancellationToken::default(),
    )
    .expect_err("test fixture must fail before staging or native cargo build");
    assert!(matches!(
        error,
        NativeDynamicPreparationError::IneligibleProductPackage {
            package_id,
            role: PluginPackageRole::TestFixture
        } if package_id == "carrier"
    ));
    assert_eq!(
        fs::read(previous_staging).expect("old staging"),
        b"keep previous staging"
    );
    assert!(!cache_root.join("packages/carrier").exists());
    assert!(!cache_root.join("manifests").exists());
    assert!(!cache_root.join("build").exists());
    debug_assert!(root.starts_with(std::env::temp_dir()));
    fs::remove_dir_all(root).expect("remove owned fixture directory");
}
