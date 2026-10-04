use super::*;
use crate::ui::retained_host::host_contract::HostChromeTooltipTarget;
use zircon_runtime_interface::ui::dispatch::{
    UiInputEventMetadata, UiInputSequence, UiPointerEvent,
};
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};
use zircon_runtime_interface::ui::tree::UiTemplateNodeMetadata;

#[test]
fn disabled_control_keeps_its_explicit_tooltip_reason() {
    let mut metadata = UiTemplateNodeMetadata::default();
    metadata
        .attributes
        .insert("disabled".to_string(), toml::Value::Boolean(true));
    metadata.attributes.insert(
        "tooltip".to_string(),
        toml::Value::String("Save is unavailable until the project is valid".to_string()),
    );

    assert_eq!(
        icon_tooltip_text(&metadata),
        Some("Save is unavailable until the project is valid")
    );
}

#[test]
fn host_chrome_anchor_crosses_the_physical_boundary_once() {
    let frame = logical_host_chrome_tooltip_frame(
        &FrameRect {
            x: 148.0,
            y: 92.0,
            width: 200.0,
            height: 48.0,
        },
        UiFrame::new(100.0, 60.0, 800.0, 600.0),
        2.0,
    )
    .unwrap();

    assert_eq!(frame, UiFrame::new(24.0, 16.0, 100.0, 24.0));
}

#[test]
fn host_chrome_tab_uses_the_runtime_delay_and_real_control_anchor() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(800.0, 600.0)).unwrap();
    let input = UiPointerInputEvent {
        metadata: UiInputEventMetadata::new(
            UiInputTimestamp::from_micros(0),
            UiInputSequence::new(1),
        ),
        event: UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(140.0, 72.0)),
        precise_scroll: None,
    };

    assert!(bridge
        .update_workbench_icon_tooltip_candidate(
            input,
            Some(WorkbenchTooltipPointerTarget::HostChrome(
                HostChromeTooltipTarget {
                    identity: "DocumentSceneTab".into(),
                    label: "Scene".into(),
                    frame: FrameRect {
                        x: 100.0,
                        y: 60.0,
                        width: 96.0,
                        height: 28.0,
                    },
                },
            )),
        )
        .unwrap());
    bridge.refresh_prepared_state_change().unwrap();
    assert_eq!(
        bridge
            .control_frame(WORKBENCH_HOST_CHROME_TOOLTIP_ANCHOR_CONTROL_ID)
            .unwrap(),
        UiFrame::new(100.0, 60.0, 96.0, 28.0)
    );
    assert!(!bridge
        .tick_workbench_icon_tooltip(UiInputTimestamp::from_micros(149_999))
        .unwrap());
    assert!(bridge
        .tick_workbench_icon_tooltip(UiInputTimestamp::from_micros(150_000))
        .unwrap());

    let popup_anchor = bridge
        .control_node_id(WORKBENCH_ICON_TOOLTIP_CONTROL_ID)
        .and_then(|node_id| bridge.surface().tree.nodes.get(&node_id))
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.widget.popup_anchor.control_id());
    assert_eq!(
        popup_anchor,
        Some(WORKBENCH_HOST_CHROME_TOOLTIP_ANCHOR_CONTROL_ID)
    );
}
