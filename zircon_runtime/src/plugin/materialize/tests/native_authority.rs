use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ExportTargetPlatform;
use crate::plugin::native::{
    NativePluginArtifactDigest, NativePluginArtifactExpectation, NativePluginArtifactTarget,
};
use crate::plugin::PluginModuleKind;

use super::{validate_captured_expectations, NativePackageInventory};

#[test]
fn authority_requires_manifest_named_library_even_if_sibling_has_same_digest() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon-export-native-authority-{}-{nonce}",
        std::process::id()
    ));
    let package = root.join("carrier");
    let native = package.join("native");
    fs::create_dir_all(&native).expect("create native fixture directory");
    let manifest_path = package.join("plugin.toml");
    fs::write(
        &manifest_path,
        "id = \"carrier\"\nversion = \"0.1.0\"\ndisplay_name = \"Carrier\"\npackage_role = \"production\"\n[[modules]]\nname = \"carrier.runtime\"\nkind = \"runtime\"\ncrate_name = \"zircon_plugin_carrier_runtime\"\n",
    )
    .expect("write native package manifest");
    let decoy = native.join(format!(
        "{}decoy{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    ));
    fs::write(&decoy, b"identical library bytes").expect("write sibling library");
    let expectation = NativePluginArtifactExpectation::trusted_local_first_party(
        "carrier",
        "carrier",
        NativePluginArtifactDigest::capture(&manifest_path).expect("digest manifest"),
        NativePluginArtifactDigest::capture(&decoy).expect("digest sibling library"),
        "test-build",
        NativePluginArtifactTarget::new(
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        ),
        [PluginModuleKind::Runtime],
        Vec::<String>::new(),
    );

    let inventory = NativePackageInventory::build(&root, &["carrier".to_owned()])
        .expect("snapshot package with sibling library");
    let error = validate_captured_expectations("carrier", &inventory, &[expectation.clone()])
        .expect_err("a same-digest sibling must not authorize an absent manifest library");
    assert!(error.to_string().contains("library"));
    drop(inventory);

    fs::write(
        native.join(format!(
            "{}zircon_plugin_carrier_runtime{}",
            std::env::consts::DLL_PREFIX,
            std::env::consts::DLL_SUFFIX
        )),
        b"identical library bytes",
    )
    .expect("write manifest library");
    let inventory = NativePackageInventory::build(&root, &["carrier".to_owned()])
        .expect("snapshot package with required library");
    validate_captured_expectations("carrier", &inventory, &[expectation])
        .expect("manifest library with matching digest should be accepted");

    drop(inventory);
    fs::remove_dir_all(root).expect("remove owned fixture directory");
}
