use super::*;
use zircon_runtime_interface::ui::layout::{UiFrame, UiPixelSnappingPolicy};
use zircon_runtime_interface::ui::v2::UiV2CompiledDocument;

#[test]
fn zui_pixel_snapping_policy_parses_compiles_and_round_trips() {
    let document = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/pixel_snapping_round_trip.zui"
version = 2

[root]
node = "root"

[nodes.root]
component = "Container"
pixel_snapping = "snap_to_pixel"
"#,
    )
    .expect("typed pixel snapping policy should parse from .zui");

    assert_eq!(
        document.nodes["root"].pixel_snapping,
        Some(UiPixelSnappingPolicy::SnapToPixel)
    );

    let compiled = UiV2DocumentCompiler::compile(&document)
        .expect("typed pixel snapping policy should compile into the arena");
    let root = compiled
        .arena
        .node(compiled.arena.root.expect("compiled root"))
        .expect("compiled root node");
    assert_eq!(root.pixel_snapping, UiPixelSnappingPolicy::SnapToPixel);

    let encoded = toml::to_string(&compiled).expect("compiled package should serialize");
    let decoded: UiV2CompiledDocument =
        toml::from_str(&encoded).expect("compiled package should deserialize");

    assert_eq!(decoded, compiled);
    assert_eq!(
        decoded
            .arena
            .node(decoded.arena.root.expect("decoded root"))
            .expect("decoded root node")
            .pixel_snapping,
        UiPixelSnappingPolicy::SnapToPixel
    );
}

#[test]
fn component_pixel_snapping_preserves_prototype_and_accepts_explicit_inherit_override() {
    let document = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/component_pixel_snapping.zui"
version = 2

[root]
node = "root"

[components.StaticChrome]
root = "static_chrome_root"

[nodes.static_chrome_root]
component = "Divider"
pixel_snapping = "snap_to_pixel"

[nodes.root]
component = "VerticalGroup"
children = [{ node = "preserved" }, { node = "overridden" }]

[nodes.preserved]
component = "StaticChrome"
control_id = "PreservedPolicy"

[nodes.overridden]
component = "StaticChrome"
control_id = "InheritedPolicy"
pixel_snapping = "inherit"
"#,
    )
    .expect("component pixel snapping fixture should parse");

    let compiled = UiV2DocumentCompiler::compile(&document)
        .expect("component pixel snapping fixture should expand");
    let preserved = compiled
        .arena
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some("PreservedPolicy"))
        .expect("mount without an override should expand");
    let overridden = compiled
        .arena
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some("InheritedPolicy"))
        .expect("mount with an explicit inherit override should expand");

    assert_eq!(preserved.pixel_snapping, UiPixelSnappingPolicy::SnapToPixel);
    assert_eq!(overridden.pixel_snapping, UiPixelSnappingPolicy::Inherit);
}

#[test]
fn render_extract_resolves_pixel_snapping_without_quantizing_layout_or_hit_geometry() {
    let document = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/render_pixel_snapping.zui"
version = 2

[root]
node = "root"

[nodes.root]
component = "Overlay"
pixel_snapping = "snap_to_pixel"
layout = { container = { kind = "Overlay" } }
children = [{ node = "inherited" }, { node = "disabled" }]

[nodes.inherited]
component = "Button"
control_id = "InheritedButton"
props = { text = "Inherited" }
layout = { position = { x = 8.25, y = 10.5 }, width = { min = 200.5, preferred = 200.5, max = 200.5, stretch = "Fixed" }, height = { min = 20.25, preferred = 20.25, max = 20.25, stretch = "Fixed" } }

[nodes.disabled]
component = "Button"
control_id = "DisabledButton"
pixel_snapping = "disabled"
props = { text = "Disabled" }
layout = { position = { x = 8.25, y = 40.25 }, width = { min = 200.5, preferred = 200.5, max = 200.5, stretch = "Fixed" }, height = { min = 20.25, preferred = 20.25, max = 20.25, stretch = "Fixed" } }
"#,
    )
    .expect("render pixel snapping fixture should parse");
    let compiled = UiV2DocumentCompiler::compile(&document)
        .expect("render pixel snapping fixture should compile");
    let mut surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.render_pixel_snapping"),
        &document,
        &compiled,
    )
    .expect("render pixel snapping surface should build");
    surface
        .compute_layout(UiSize::new(320.0, 96.0))
        .expect("render pixel snapping surface should arrange");

    let inherited_id = node_id_by_control_id(&surface, "InheritedButton");
    let disabled_id = node_id_by_control_id(&surface, "DisabledButton");
    let inherited_frame = surface
        .arranged_tree
        .get(inherited_id)
        .expect("inherited button should be arranged")
        .frame;
    let disabled_frame = surface
        .arranged_tree
        .get(disabled_id)
        .expect("disabled button should be arranged")
        .frame;

    assert_frame_close(inherited_frame, UiFrame::new(8.25, 10.5, 200.5, 20.25));
    assert_frame_close(disabled_frame, UiFrame::new(8.25, 40.25, 200.5, 20.25));

    let frame = surface.surface_frame();
    let inherited_hit = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == inherited_id)
        .expect("inherited button should retain logical hit geometry");
    let disabled_hit = frame
        .hit_grid
        .entries
        .iter()
        .find(|entry| entry.node_id == disabled_id)
        .expect("disabled button should retain logical hit geometry");
    assert_frame_close(inherited_hit.frame, inherited_frame);
    assert_frame_close(disabled_hit.frame, disabled_frame);

    let inherited_command = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == inherited_id)
        .expect("inherited button should emit a command");
    let disabled_command = frame
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == disabled_id)
        .expect("disabled button should emit a command");
    assert_eq!(
        inherited_command.style.pixel_snapping,
        UiPixelSnappingPolicy::SnapToPixel
    );
    assert_eq!(
        disabled_command.style.pixel_snapping,
        UiPixelSnappingPolicy::Disabled
    );
}

fn assert_frame_close(actual: UiFrame, expected: UiFrame) {
    for (actual, expected) in [
        (actual.x, expected.x),
        (actual.y, expected.y),
        (actual.width, expected.width),
        (actual.height, expected.height),
    ] {
        assert!(
            (actual - expected).abs() <= 0.001,
            "expected {expected}, got {actual}"
        );
    }
}
