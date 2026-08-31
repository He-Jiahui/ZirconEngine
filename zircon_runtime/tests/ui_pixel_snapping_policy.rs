use std::collections::BTreeMap;

use zircon_runtime::ui::v2::{
    UiV2DocumentCompiler, UiV2PrototypeStore, UiV2SurfaceBuilder, UiZuiAssetLoader,
};
use zircon_runtime_interface::ui::{
    event_ui::UiTreeId,
    layout::{UiFrame, UiPixelSnappingPolicy, UiSize},
};

const CARD_COMPONENT: &str = r#"
[asset]
kind = "component"
id = "res://ui/pixel_card.zui"
version = 2
display_name = "Pixel Card"

[components.PixelCard]
root = "root"

[nodes.root]
component = "Panel"
pixel_snapping = "disabled"
"#;

const POLICY_VIEW: &str = r#"
[asset]
kind = "view"
id = "res://ui/pixel_policy_view.zui"
version = 2
display_name = "Pixel Policy View"

[root]
node = "root"

[imports]
widgets = ["res://ui/pixel_card.zui#PixelCard"]

[nodes.root]
component = "Panel"
control_id = "PolicyRoot"
pixel_snapping = "snap_to_pixel"

[[nodes.root.children]]
node = "disabled_card"

[[nodes.root.children]]
node = "inherited_card"

[nodes.disabled_card]
component = "PixelCard"
control_id = "DisabledCard"

[nodes.inherited_card]
component = "PixelCard"
control_id = "InheritedCard"
pixel_snapping = "inherit"
"#;

#[test]
fn zui_pixel_snapping_policy_survives_component_expansion_package_and_render_extract() {
    let component = UiZuiAssetLoader::load_zui_str(CARD_COMPONENT).expect("component zui");
    let view = UiZuiAssetLoader::load_zui_str(POLICY_VIEW).expect("view zui");
    let mut store = UiV2PrototypeStore::new();
    store.insert(component);

    let compiled = UiV2DocumentCompiler::compile_with_prototype_store(&view, &store)
        .expect("compile policy view");
    let serialized = serde_json::to_vec(&compiled).expect("serialize compiled policy package");
    let compiled =
        serde_json::from_slice(&serialized).expect("deserialize compiled policy package");
    let compiled_policies = compiled
        .arena
        .nodes
        .iter()
        .filter_map(|node| {
            node.control_id
                .as_deref()
                .map(|control_id| (control_id, node.pixel_snapping))
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        compiled_policies.get("PolicyRoot"),
        Some(&UiPixelSnappingPolicy::SnapToPixel)
    );
    assert_eq!(
        compiled_policies.get("DisabledCard"),
        Some(&UiPixelSnappingPolicy::Disabled)
    );
    assert_eq!(
        compiled_policies.get("InheritedCard"),
        Some(&UiPixelSnappingPolicy::Inherit)
    );

    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("pixel-policy"),
        &view,
        &compiled,
    )
    .expect("build policy surface");
    let mut node_ids = BTreeMap::new();
    for node in surface.tree.nodes.values_mut() {
        node.layout_cache.frame = UiFrame::new(10.6, 8.0, 80.0, 24.0);
        if let Some(control_id) = node
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
        {
            node_ids.insert(control_id.to_string(), node.node_id);
        }
    }
    surface.rebuild_authored_frames(UiSize::new(320.0, 180.0));

    let command_policy = |control_id: &str| {
        let node_id = node_ids[control_id];
        let policies = surface
            .render_extract
            .list
            .commands
            .iter()
            .filter(|command| command.node_id == node_id)
            .map(|command| command.style.pixel_snapping)
            .collect::<Vec<_>>();
        assert!(!policies.is_empty(), "{control_id} should emit commands");
        assert!(policies.windows(2).all(|pair| pair[0] == pair[1]));
        policies[0]
    };

    assert_eq!(
        command_policy("PolicyRoot"),
        UiPixelSnappingPolicy::SnapToPixel
    );
    assert_eq!(
        command_policy("DisabledCard"),
        UiPixelSnappingPolicy::Disabled
    );
    assert_eq!(
        command_policy("InheritedCard"),
        UiPixelSnappingPolicy::SnapToPixel
    );
}

#[test]
fn zui_pixel_snapping_policy_rejects_unknown_tokens() {
    let invalid = POLICY_VIEW.replace("snap_to_pixel", "nearest_maybe");
    let error = UiZuiAssetLoader::load_zui_str(&invalid).expect_err("unknown policy must fail");

    assert!(error.to_string().contains("nearest_maybe"));
}
