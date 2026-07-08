use std::path::Path;

pub(super) const EXPECTED_RUNTIME_01_MANIFESTS: &[&str] = &[
    "../Cargo.toml",
    "Cargo.toml",
    "../zircon_runtime_interface/Cargo.toml",
    "../zircon_editor/Cargo.toml",
    "../zircon_plugins/physics/runtime/Cargo.toml",
];

pub(super) fn assert_runtime_01_manifests_exist(runtime_root: &Path) {
    for manifest in EXPECTED_RUNTIME_01_MANIFESTS {
        let path = runtime_root.join(manifest);
        assert!(
            path.exists(),
            "Runtime 01 audited manifest `{}` is missing; update tech_stack_boundary before changing dependency ownership",
            path.display()
        );
    }
}
