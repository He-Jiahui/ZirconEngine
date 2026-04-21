#[test]
fn extensions_root_stays_structural_after_module_split() {
    let source = include_str!("../../extensions/mod.rs");

    for forbidden in [
        "use std::sync::Arc;",
        "fn module_descriptor_with_driver_and_manager",
        "DriverDescriptor",
        "ManagerDescriptor",
        "ServiceObject",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected extensions/mod.rs to stay structural after split, found `{forbidden}`"
        );
    }
}

#[test]
fn extension_module_roots_stay_structural_after_module_split() {
    for (label, source, forbidden) in [
        (
            "navigation",
            include_str!("../../extensions/navigation/mod.rs"),
            [
                "pub struct NavigationConfig",
                "pub struct NavigationModule",
                "pub fn module_descriptor(",
                "impl EngineModule for NavigationModule",
            ],
        ),
        (
            "net",
            include_str!("../../extensions/net/mod.rs"),
            [
                "pub struct NetConfig",
                "pub struct NetModule",
                "pub fn module_descriptor(",
                "impl EngineModule for NetModule",
            ],
        ),
        (
            "particles",
            include_str!("../../extensions/particles/mod.rs"),
            [
                "pub struct ParticlesConfig",
                "pub struct ParticlesModule",
                "pub fn module_descriptor(",
                "impl EngineModule for ParticlesModule",
            ],
        ),
        (
            "sound",
            include_str!("../../extensions/sound/mod.rs"),
            [
                "pub struct SoundConfig",
                "pub struct SoundModule",
                "pub fn module_descriptor(",
                "impl EngineModule for SoundModule",
            ],
        ),
        (
            "texture",
            include_str!("../../extensions/texture/mod.rs"),
            [
                "pub struct TextureConfig",
                "pub struct TextureModule",
                "pub fn module_descriptor(",
                "impl EngineModule for TextureModule",
            ],
        ),
    ] {
        for marker in forbidden {
            assert!(
                !source.contains(marker),
                "expected extensions/{label}/mod.rs to stay structural after split, found `{marker}`"
            );
        }
    }
}
