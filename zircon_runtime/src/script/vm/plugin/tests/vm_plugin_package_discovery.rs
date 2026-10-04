use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::script::{VmError, VmPluginGarbageCollectionMode, VmPluginHotReloadPolicy};

use super::{
    bytecode_file_name, discover_vm_plugin_package, discover_vm_plugin_package_with_limits,
    discover_vm_plugin_packages_with_limits, VmPluginDiscoveryLimits, VmPluginPayloadCache,
};

#[test]
fn bytecode_file_name_borrows_custom_and_default_values() {
    assert_eq!(
        bytecode_file_name(Some("module/runtime.zrbc")),
        "module/runtime.zrbc"
    );
    assert_eq!(bytecode_file_name(None), "plugin.bin");
}

#[test]
fn discovery_defers_bytecode_until_selected_package_materialization() {
    let fixture = PackageFixture::new();
    fixture.write_bytecode_package(concat!(
        "name = \"lazy_payload\"\n",
        "version = \"0.1.0\"\n",
        "entry = \"main\"\n",
        "backend = \"mock\"\n",
        "bytecode = \"plugin.bin\"\n",
    ));

    let discovered = discover_vm_plugin_package(&fixture.manifest_path).unwrap();

    assert!(discovered.package.bytecode.is_empty());
    let cache = VmPluginPayloadCache::default();
    let materialized = cache.materialize(&discovered).unwrap();
    assert_eq!(materialized.bytecode, [1, 2, 3]);
}

#[test]
fn discovery_rejects_tree_depth_beyond_the_configured_limit() {
    let fixture = PackageFixture::new();
    let nested_root = fixture.root.join("nested");
    fs::create_dir_all(&nested_root).unwrap();
    fs::write(
        nested_root.join("plugin.toml"),
        concat!(
            "name = \"too_deep\"\n",
            "version = \"0.1.0\"\n",
            "entry = \"main\"\n",
            "backend = \"mock\"\n",
        ),
    )
    .unwrap();
    let limits = VmPluginDiscoveryLimits {
        max_depth: 0,
        ..VmPluginDiscoveryLimits::default()
    };

    let error = discover_vm_plugin_packages_with_limits(&fixture.root, limits).unwrap_err();

    assert!(error.to_string().contains("depth"));
}

#[test]
fn discovery_rejects_manifest_and_bytecode_before_oversized_allocation() {
    let fixture = PackageFixture::new();
    fixture.write_bytecode_package(concat!(
        "name = \"bounded_payload\"\n",
        "version = \"0.1.0\"\n",
        "entry = \"main\"\n",
        "backend = \"mock\"\n",
        "bytecode = \"plugin.bin\"\n",
    ));
    let manifest_limits = VmPluginDiscoveryLimits {
        max_manifest_bytes: 16,
        ..VmPluginDiscoveryLimits::default()
    };
    let manifest_error =
        discover_vm_plugin_package_with_limits(&fixture.manifest_path, manifest_limits)
            .unwrap_err();
    assert!(manifest_error.to_string().contains("manifest byte budget"));

    let discovered = discover_vm_plugin_package(&fixture.manifest_path).unwrap();
    let payload_cache = VmPluginPayloadCache::new(VmPluginDiscoveryLimits {
        max_bytecode_bytes: 2,
        ..VmPluginDiscoveryLimits::default()
    });
    let payload_error = payload_cache.materialize(&discovered).unwrap_err();
    assert!(payload_error.to_string().contains("bytecode byte budget"));
}

#[test]
fn unchanged_bytecode_fingerprint_reuses_the_single_flight_payload() {
    let fixture = PackageFixture::new();
    fixture.write_bytecode_package(concat!(
        "name = \"single_flight\"\n",
        "version = \"0.1.0\"\n",
        "entry = \"main\"\n",
        "backend = \"mock\"\n",
        "bytecode = \"plugin.bin\"\n",
    ));
    let cache = VmPluginPayloadCache::default();

    let first = cache.load_path(&fixture.bytecode_path).unwrap();
    let second = cache.load_path(&fixture.bytecode_path).unwrap();

    assert!(std::sync::Arc::ptr_eq(&first, &second));
}

#[test]
fn discovery_defaults_vm_management_policy_when_manifest_omits_it() {
    let fixture = PackageFixture::new();
    fixture.write_bytecode_package(concat!(
        "name = \"default_policy\"\n",
        "version = \"0.1.0\"\n",
        "entry = \"main\"\n",
        "backend = \"mock\"\n",
        "bytecode = \"plugin.bin\"\n",
        "\n",
        "[capabilities]\n",
        "capabilities = [\"render\"]\n",
    ));

    let discovered = discover_vm_plugin_package(&fixture.manifest_path).unwrap();

    assert_eq!(
        discovered.package.manifest.management.hot_reload,
        VmPluginHotReloadPolicy::PreserveState
    );
    assert_eq!(
        discovered
            .package
            .manifest
            .management
            .garbage_collection
            .mode,
        VmPluginGarbageCollectionMode::BackendManaged
    );
}

#[test]
fn discovery_parses_vm_management_policy_from_manifest() {
    let fixture = PackageFixture::new();
    fixture.write_bytecode_package(concat!(
        "name = \"managed_policy\"\n",
        "version = \"0.1.0\"\n",
        "entry = \"main\"\n",
        "backend = \"mock\"\n",
        "bytecode = \"plugin.bin\"\n",
        "\n",
        "[capabilities]\n",
        "capabilities = [\"render\"]\n",
        "\n",
        "[management]\n",
        "hot_reload = \"stateless\"\n",
        "\n",
        "[management.garbage_collection]\n",
        "mode = \"cooperative\"\n",
        "interval_frames = 120\n",
        "\n",
        "[management.memory]\n",
        "soft_limit_bytes = 1024\n",
        "hard_limit_bytes = 2048\n",
    ));

    let discovered = discover_vm_plugin_package(&fixture.manifest_path).unwrap();
    let management = discovered.package.manifest.management;

    assert_eq!(management.hot_reload, VmPluginHotReloadPolicy::Stateless);
    assert_eq!(
        management.garbage_collection.mode,
        VmPluginGarbageCollectionMode::Cooperative
    );
    assert_eq!(management.garbage_collection.interval_frames, Some(120));
    assert_eq!(management.memory.soft_limit_bytes, Some(1024));
    assert_eq!(management.memory.hard_limit_bytes, Some(2048));
}

#[test]
fn discovery_rejects_invalid_vm_management_policy() {
    let fixture = PackageFixture::new();
    fixture.write_bytecode_package(concat!(
        "name = \"bad_policy\"\n",
        "version = \"0.1.0\"\n",
        "entry = \"main\"\n",
        "backend = \"mock\"\n",
        "bytecode = \"plugin.bin\"\n",
        "\n",
        "[capabilities]\n",
        "capabilities = [\"render\"]\n",
        "\n",
        "[management.memory]\n",
        "soft_limit_bytes = 2048\n",
        "hard_limit_bytes = 1024\n",
    ));

    let error = discover_vm_plugin_package(&fixture.manifest_path).unwrap_err();

    assert!(matches!(error, VmError::Parse(_)));
    assert!(error
        .to_string()
        .contains("invalid plugin management policy"));
    assert!(error.to_string().contains("soft_limit_bytes 2048 exceeds"));
}

#[test]
fn discovery_rejects_zr_vm_project_fallback_backend() {
    let fixture = PackageFixture::new();
    fs::write(
        &fixture.manifest_path,
        concat!(
            "name = \"fallback_project\"\n",
            "version = \"0.1.0\"\n",
            "entry = \"main\"\n",
            "backend = \"zr_vm_fallback:project\"\n",
            "\n",
            "[zr_vm]\n",
            "project = \"script/plugin.zrp\"\n",
        ),
    )
    .unwrap();

    let error = discover_vm_plugin_package(&fixture.manifest_path).unwrap_err();

    assert!(matches!(error, VmError::Parse(_)));
    assert!(error
        .to_string()
        .contains("[zr_vm] project section requires backend = \"zr_vm:project\""));
}

struct PackageFixture {
    root: PathBuf,
    manifest_path: PathBuf,
    bytecode_path: PathBuf,
}

impl PackageFixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("zircon-vm-management-{nonce}"));
        fs::create_dir_all(&root).unwrap();
        Self {
            manifest_path: root.join("plugin.toml"),
            bytecode_path: root.join("plugin.bin"),
            root,
        }
    }

    fn write_bytecode_package(&self, manifest: &str) {
        fs::write(&self.manifest_path, manifest).unwrap();
        fs::write(&self.bytecode_path, [1, 2, 3]).unwrap();
    }
}

impl Drop for PackageFixture {
    fn drop(&mut self) {
        remove_dir_all_if_exists(&self.root);
    }
}

fn remove_dir_all_if_exists(path: &Path) {
    if path.exists() {
        let _ = fs::remove_dir_all(path);
    }
}
