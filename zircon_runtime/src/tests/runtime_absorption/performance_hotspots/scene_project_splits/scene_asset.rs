#[test]
fn runtime_07_scene_asset_folder_split_keeps_public_surface_and_single_owner() {
    fn occurrence_count(source: &str, needle: &str) -> usize {
        source.matches(needle).count()
    }

    let scene_mod = include_str!("../../../../asset/assets/scene/mod.rs");
    let scene_lighting = include_str!("../../../../asset/assets/scene/lighting.rs");
    let scene_physics = include_str!("../../../../asset/assets/scene/physics.rs");
    let asset_assets_mod = include_str!("../../../../asset/assets/mod.rs");
    let asset_mod = include_str!("../../../../asset/mod.rs");
    let runtime_07_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let hotspot_doc =
        include_str!("../../../../../../docs/zircon_runtime/performance/hotspot_inventory.md");
    let scene_doc = include_str!("../../../../../../docs/zircon_runtime/asset/assets/scene.md");

    for module_decl in [
        "mod animation;",
        "mod asset;",
        "mod camera;",
        "mod defaults;",
        "mod entity;",
        "mod extensions;",
        "mod lighting;",
        "mod management;",
        "mod mesh;",
        "mod physics;",
        "mod post_process;",
        "mod transform;",
    ] {
        assert!(
            scene_mod.contains(module_decl),
            "scene/mod.rs should keep folder-backed declaration `{module_decl}`"
        );
    }

    assert_eq!(
        occurrence_count(scene_mod, "pub enum SceneMobilityAsset"),
        1,
        "scene/mod.rs should be the only SceneMobilityAsset enum owner"
    );
    assert!(
        !scene_physics.contains("SceneMobilityAsset"),
        "scene/physics.rs should not reintroduce a duplicate SceneMobilityAsset owner"
    );

    for export_anchor in [
        "pub use lighting::{",
        "SceneSpotLightAsset",
        "pub use scene::{",
        "SceneMobilityAsset",
    ] {
        assert!(
            scene_mod.contains(export_anchor)
                || asset_assets_mod.contains(export_anchor)
                || asset_mod.contains(export_anchor),
            "scene asset export chain should retain `{export_anchor}`"
        );
    }

    for spot_light_anchor in [
        "pub struct SceneSpotLightAsset",
        "pub direction: [Real; 3]",
        "pub outer_angle_radians: Real",
    ] {
        assert!(
            scene_lighting.contains(spot_light_anchor),
            "SceneSpotLightAsset should retain public field anchor `{spot_light_anchor}`"
        );
    }

    for doc_anchor in [
        "scene/{mod,animation,asset,camera,defaults,entity,extensions,lighting,management,mesh,physics,post_process,transform}.rs",
        "SceneMobilityAsset",
        "SceneSpotLightAsset",
        "split-drift repair",
        "split_drift_static_passed_cargo_deferred_active_lanes",
        "scene asset split-drift repair",
    ] {
        assert!(
            runtime_07_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || hotspot_doc.contains(doc_anchor)
                || scene_doc.contains(doc_anchor),
            "Runtime 07 scene split docs should retain `{doc_anchor}`"
        );
    }
}
