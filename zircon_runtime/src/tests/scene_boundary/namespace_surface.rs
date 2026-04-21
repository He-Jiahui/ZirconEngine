#[test]
fn runtime_scene_surface_keeps_scene_domain_under_namespaces() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let scene_mod_source =
        std::fs::read_to_string(runtime_root.join("src/scene/mod.rs")).unwrap_or_default();

    for required in [
        "pub mod components;",
        "pub mod semantics;",
        "pub mod serializer;",
        "pub mod world;",
    ] {
        assert!(
            scene_mod_source.contains(required),
            "zircon_runtime::scene should expose namespace surface `{required}`"
        );
    }

    for forbidden in [
        "pub use zircon_scene::{",
        "CameraComponent",
        "ComponentData",
        "EntityIdentity",
        "SceneAssetSerializer",
        "SceneProjectError",
        "WorldMatrix",
    ] {
        assert!(
            !scene_mod_source.contains(forbidden),
            "zircon_runtime::scene should stop flattening scene-domain surface `{forbidden}`"
        );
    }
}
