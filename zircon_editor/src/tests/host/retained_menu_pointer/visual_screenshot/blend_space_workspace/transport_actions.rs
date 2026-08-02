use super::*;
use crate::ui::binding::EditorUiBindingPayload;

const TRANSPORT_ACTIONS: &[(&str, &str)] = &[
    (
        "WorkbenchTransportRecord",
        "workbench.extension.animation_transport.record.toggle",
    ),
    (
        "WorkbenchTransportPlay",
        "workbench.extension.animation_transport.play.invoke",
    ),
    (
        "WorkbenchTransportPause",
        "workbench.extension.animation_transport.pause.invoke",
    ),
    (
        "WorkbenchTransportPrevious",
        "workbench.extension.animation_transport.previous.invoke",
    ),
    (
        "WorkbenchTransportNext",
        "workbench.extension.animation_transport.next.invoke",
    ),
    (
        "WorkbenchTransportLoop",
        "workbench.extension.animation_transport.loop.toggle",
    ),
];

#[test]
fn workbench_transport_controls_use_shared_dense_control_tokens() {
    let source = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "assets/ui/editor/components/workbench/composites/animation/\
                 workbench_transport_controls.zui",
    ))
    .expect("Workbench transport controls asset should be readable");

    for required in [
        "gap = \"$editor.density.gap.xsmall\"",
        "height = { min = \"$editor.control.height.dense\", preferred = \"$editor.control.height.dense\", max = \"$editor.control.height.dense\", stretch = \"Fixed\" }",
        "layout_padding_left = \"$editor.density.gap.xsmall\"",
        "layout_padding_right = \"$editor.density.gap.xsmall\"",
        "layout_padding_top = \"$editor.density.gap.xsmall\"",
        "layout_padding_bottom = \"$editor.density.gap.xsmall\"",
        "width = { min = \"$editor.control.height.dense\", preferred = \"$editor.control.height.dense\", max = \"$editor.control.height.dense\", stretch = \"Fixed\" }",
    ] {
        assert!(
            source.contains(required),
            "transport controls must consume the shared dense-control contract: {required}"
        );
    }
    assert!(
        !source.contains("layout_padding_left = 2.0"),
        "transport controls must not retain feature-local horizontal padding"
    );
    assert!(
        !source.contains("width = { min = 28.0"),
        "transport control button geometry must follow the central dense-control metric"
    );
}

#[test]
fn blend_space_transport_buttons_dispatch_unique_registered_actions() {
    let mut bridge = open_blend_space_bridge(1260, 780);
    let mut routed_actions = BTreeSet::new();

    for (control_id, expected_action) in TRANSPORT_ACTIONS {
        let binding = bridge
            .dispatch_control_state(control_id, UiEventKind::Click)
            .unwrap_or_else(|error| panic!("{control_id} should dispatch without error: {error}"))
            .unwrap_or_else(|| panic!("{control_id} should expose its Click binding"));
        assert!(matches!(
            binding.payload(),
            EditorUiBindingPayload::MenuAction { action_id } if action_id == expected_action
        ));
        assert!(
            routed_actions.insert(expected_action),
            "transport actions must be unique: {expected_action}"
        );
    }
}

#[test]
fn blend_space_transport_actions_update_independent_preview_state() {
    let mut bridge = open_blend_space_bridge(1260, 780);

    assert!(selected(&bridge, "WorkbenchTransportPlay"));
    assert!(control_bool(&bridge, "WorkbenchTransportLoop", "checked"));

    dispatch(&mut bridge, "WorkbenchTransportPause");
    assert!(!selected(&bridge, "WorkbenchTransportPlay"));
    assert!(selected(&bridge, "WorkbenchTransportPause"));
    assert_eq!(preview_status(&bridge), "Run_Fwd  |  Paused");

    dispatch(&mut bridge, "WorkbenchTransportPlay");
    assert!(selected(&bridge, "WorkbenchTransportPlay"));
    assert!(!selected(&bridge, "WorkbenchTransportPause"));
    assert_eq!(preview_status(&bridge), "Run_Fwd  |  Previewing");

    dispatch(&mut bridge, "WorkbenchTransportPrevious");
    assert_eq!(timeline_time(&bridge), 0.0);
    assert_eq!(preview_status(&bridge), "Run_Fwd  |  Start");

    dispatch(&mut bridge, "WorkbenchTransportNext");
    assert_eq!(timeline_time(&bridge), 3.0);
    assert_eq!(preview_status(&bridge), "Run_Fwd  |  End");

    dispatch(&mut bridge, "WorkbenchTransportLoop");
    assert!(!control_bool(&bridge, "WorkbenchTransportLoop", "checked"));
    assert_eq!(preview_status(&bridge), "Run_Fwd  |  Loop disabled");

    dispatch(&mut bridge, "WorkbenchTransportRecord");
    assert!(control_bool(&bridge, "WorkbenchTransportRecord", "checked"));
    assert_eq!(preview_status(&bridge), "Run_Fwd  |  Recording armed");
}

fn dispatch(bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge, control_id: &str) {
    bridge
        .dispatch_control_state(control_id, UiEventKind::Click)
        .unwrap_or_else(|error| panic!("{control_id} should dispatch without error: {error}"))
        .unwrap_or_else(|| panic!("{control_id} should expose its Click binding"));
}

fn control<'a>(
    bridge: &'a BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
) -> &'a crate::ui::template_runtime::RetainedUiHostNodeModel {
    bridge
        .host_projection()
        .node_by_control_id(control_id)
        .unwrap_or_else(|| panic!("{control_id} should exist in the retained projection"))
}

fn preview_status(bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge) -> String {
    control(bridge, "WorkbenchExtensionBlendSpacePreviewStatus")
        .text
        .as_deref()
        .unwrap_or_default()
        .to_string()
}

fn selected(bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge, control_id: &str) -> bool {
    control_bool(bridge, control_id, "selected")
}

fn control_bool(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> bool {
    bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
                .and_then(|metadata| metadata.attributes.get(property))
                .and_then(toml::Value::as_bool)
        })
        .unwrap_or(false)
}

fn timeline_time(bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge) -> f32 {
    match control(bridge, "WorkbenchExtensionBlendSpacePreviewTimeline")
        .properties
        .get("current_time")
    {
        Some(crate::ui::template_runtime::RetainedUiHostValue::Float(value)) => *value as f32,
        Some(crate::ui::template_runtime::RetainedUiHostValue::Integer(value)) => *value as f32,
        value => panic!("timeline current_time should be numeric, found {value:?}"),
    }
}
