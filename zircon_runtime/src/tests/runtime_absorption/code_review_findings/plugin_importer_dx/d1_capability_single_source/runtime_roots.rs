const D1_RUNTIME_CAPABILITY_ROOTS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "ai",
        include_str!("../../../../../../../zircon_plugins/ai/runtime/src/capability.rs"),
        include_str!("../../../../../../../zircon_plugins/ai/runtime/src/lib.rs"),
        include_str!("../../../../../../../zircon_plugins/ai/runtime/src/plugin.rs"),
        include_str!("../../../../../../../zircon_plugins/ai/plugin.toml"),
    ),
    (
        "animation",
        include_str!("../../../../../../../zircon_plugins/animation/runtime/src/capability.rs"),
        include_str!("../../../../../../../zircon_plugins/animation/runtime/src/lib.rs"),
        include_str!("../../../../../../../zircon_plugins/animation/runtime/src/plugin.rs"),
        include_str!("../../../../../../../zircon_plugins/animation/plugin.toml"),
    ),
    (
        "hybrid_gi",
        include_str!("../../../../../../../zircon_plugins/hybrid_gi/runtime/src/capability.rs"),
        include_str!("../../../../../../../zircon_plugins/hybrid_gi/runtime/src/lib.rs"),
        include_str!("../../../../../../../zircon_plugins/hybrid_gi/runtime/src/plugin.rs"),
        include_str!("../../../../../../../zircon_plugins/hybrid_gi/plugin.toml"),
    ),
    (
        "navigation",
        include_str!("../../../../../../../zircon_plugins/navigation/runtime/src/capability.rs"),
        include_str!("../../../../../../../zircon_plugins/navigation/runtime/src/lib.rs"),
        include_str!("../../../../../../../zircon_plugins/navigation/runtime/src/plugin.rs"),
        include_str!("../../../../../../../zircon_plugins/navigation/plugin.toml"),
    ),
    (
        "net",
        include_str!("../../../../../../../zircon_plugins/net/runtime/src/capability.rs"),
        include_str!("../../../../../../../zircon_plugins/net/runtime/src/lib.rs"),
        include_str!("../../../../../../../zircon_plugins/net/runtime/src/plugin.rs"),
        include_str!("../../../../../../../zircon_plugins/net/plugin.toml"),
    ),
    (
        "particles",
        include_str!("../../../../../../../zircon_plugins/particles/runtime/src/capability.rs"),
        include_str!("../../../../../../../zircon_plugins/particles/runtime/src/lib.rs"),
        include_str!("../../../../../../../zircon_plugins/particles/runtime/src/plugin.rs"),
        include_str!("../../../../../../../zircon_plugins/particles/plugin.toml"),
    ),
    (
        "physics",
        include_str!("../../../../../../../zircon_plugins/physics/runtime/src/capability.rs"),
        include_str!("../../../../../../../zircon_plugins/physics/runtime/src/lib.rs"),
        include_str!("../../../../../../../zircon_plugins/physics/runtime/src/plugin.rs"),
        include_str!("../../../../../../../zircon_plugins/physics/plugin.toml"),
    ),
    (
        "prefab_tools",
        include_str!("../../../../../../../zircon_plugins/prefab_tools/runtime/src/capability.rs"),
        include_str!("../../../../../../../zircon_plugins/prefab_tools/runtime/src/lib.rs"),
        include_str!("../../../../../../../zircon_plugins/prefab_tools/runtime/src/plugin.rs"),
        include_str!("../../../../../../../zircon_plugins/prefab_tools/plugin.toml"),
    ),
    (
        "rendering",
        include_str!("../../../../../../../zircon_plugins/rendering/runtime/src/capability.rs"),
        include_str!("../../../../../../../zircon_plugins/rendering/runtime/src/lib.rs"),
        include_str!("../../../../../../../zircon_plugins/rendering/runtime/src/plugin.rs"),
        include_str!("../../../../../../../zircon_plugins/rendering/plugin.toml"),
    ),
    (
        "solari",
        include_str!("../../../../../../../zircon_plugins/solari/runtime/src/capability.rs"),
        include_str!("../../../../../../../zircon_plugins/solari/runtime/src/lib.rs"),
        include_str!("../../../../../../../zircon_plugins/solari/runtime/src/plugin.rs"),
        include_str!("../../../../../../../zircon_plugins/solari/plugin.toml"),
    ),
    (
        "terrain",
        include_str!("../../../../../../../zircon_plugins/terrain/runtime/src/capability.rs"),
        include_str!("../../../../../../../zircon_plugins/terrain/runtime/src/lib.rs"),
        include_str!("../../../../../../../zircon_plugins/terrain/runtime/src/plugin.rs"),
        include_str!("../../../../../../../zircon_plugins/terrain/plugin.toml"),
    ),
    (
        "texture",
        include_str!("../../../../../../../zircon_plugins/texture/runtime/src/capability.rs"),
        include_str!("../../../../../../../zircon_plugins/texture/runtime/src/lib.rs"),
        include_str!("../../../../../../../zircon_plugins/texture/runtime/src/plugin.rs"),
        include_str!("../../../../../../../zircon_plugins/texture/plugin.toml"),
    ),
    (
        "tilemap_2d",
        include_str!("../../../../../../../zircon_plugins/tilemap_2d/runtime/src/capability.rs"),
        include_str!("../../../../../../../zircon_plugins/tilemap_2d/runtime/src/lib.rs"),
        include_str!("../../../../../../../zircon_plugins/tilemap_2d/runtime/src/plugin.rs"),
        include_str!("../../../../../../../zircon_plugins/tilemap_2d/plugin.toml"),
    ),
    (
        "virtual_geometry",
        include_str!(
            "../../../../../../../zircon_plugins/virtual_geometry/runtime/src/capability.rs"
        ),
        include_str!("../../../../../../../zircon_plugins/virtual_geometry/runtime/src/lib.rs"),
        include_str!("../../../../../../../zircon_plugins/virtual_geometry/runtime/src/plugin.rs"),
        include_str!("../../../../../../../zircon_plugins/virtual_geometry/plugin.toml"),
    ),
    (
        "zr_vm_language",
        include_str!(
            "../../../../../../../zircon_plugins/zr_vm_language/runtime/src/capability.rs"
        ),
        include_str!("../../../../../../../zircon_plugins/zr_vm_language/runtime/src/lib.rs"),
        include_str!("../../../../../../../zircon_plugins/zr_vm_language/runtime/src/plugin.rs"),
        include_str!("../../../../../../../zircon_plugins/zr_vm_language/plugin.toml"),
    ),
];

pub(super) fn assert_runtime_capability_roots_use_single_source() {
    assert_eq!(
        D1_RUNTIME_CAPABILITY_ROOTS.len(),
        15,
        "D1 capability single-source guard should cover every first-party trait-backed runtime root"
    );
    for (root, capability_source, lib_source, plugin_source, manifest_source) in
        D1_RUNTIME_CAPABILITY_ROOTS
    {
        assert!(
            capability_source.contains("pub const RUNTIME_CAPABILITIES: &[&str]")
                && capability_source.contains("&["),
            "{root} runtime capability.rs should own the runtime capability slice"
        );
        assert!(
            capability_source
                .lines()
                .any(|line| line.trim_start().starts_with("pub const ")
                    && line.contains("CAPABILITY")
                    && line.contains(": &str")),
            "{root} runtime capability.rs should own named capability constants"
        );
        assert!(
            lib_source.contains("mod capability;"),
            "{root} runtime lib.rs should mount capability.rs as the single source"
        );
        assert!(
            lib_source.contains("pub use capability::")
                && lib_source.contains("RUNTIME_CAPABILITIES"),
            "{root} runtime lib.rs should re-export capability.rs constants instead of restating them"
        );
        assert!(
            lib_source.contains("runtime_capabilities"),
            "{root} runtime lib.rs should expose the runtime capability accessor"
        );
        for (line_number, line) in lib_source.lines().enumerate() {
            let trimmed = line.trim_start();
            assert!(
                !(trimmed.starts_with("pub const ")
                    && trimmed.contains("CAPABILITY")
                    && trimmed.contains(": &str")),
                "{root} runtime lib.rs should not redeclare capability constants at line {}",
                line_number + 1
            );
        }
        assert!(
            plugin_source.contains("pub fn runtime_capabilities() -> &'static [&'static str]")
                && plugin_source.contains("RUNTIME_CAPABILITIES"),
            "{root} runtime plugin.rs should project capability.rs through the SDK-facing accessor"
        );
        assert!(
            manifest_source.contains("capabilities = [") && manifest_source.contains("[[modules]]"),
            "{root} plugin.toml should keep root and module capability lists auditable"
        );
    }
}
