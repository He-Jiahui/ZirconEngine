use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ExportPackagingStrategy;
use zircon_runtime::plugin::PluginModuleKind;

use crate::{
    package_manifest, PARTICLES_DIST_CRATE_NAME, PARTICLES_DIST_RUNTIME_ENTRY, RUNTIME_CAPABILITIES,
};

#[test]
fn particles_package_manifest_declares_dist_contract() {
    let manifest = package_manifest();

    assert!(manifest
        .default_packaging
        .contains(&ExportPackagingStrategy::NativeDynamic));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("particles distribution manifest");
    assert_eq!(distribution.forms, vec!["dist".to_string()]);
    assert_eq!(
        distribution.default_packaging,
        vec![ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, PARTICLES_DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(distribution.runtime_entry, PARTICLES_DIST_RUNTIME_ENTRY);

    let native_module = manifest
        .modules
        .iter()
        .find(|module| module.name == "particles.dist")
        .expect("particles native dist module");
    assert_eq!(native_module.kind, PluginModuleKind::Native);
    assert_eq!(native_module.crate_name, PARTICLES_DIST_CRATE_NAME);
    assert_eq!(
        native_module.target_modes,
        vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost
        ]
    );
    for capability in RUNTIME_CAPABILITIES {
        assert!(native_module.capabilities.contains(&capability.to_string()));
    }

    assert!(manifest
        .modules
        .iter()
        .any(|module| module.name == "particles.runtime"));
    assert_eq!(
        manifest
            .optional_features
            .iter()
            .map(|feature| feature.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "particles.physics",
            "particles.animation_control",
            "particles.gpu_simulation",
        ]
    );
}
