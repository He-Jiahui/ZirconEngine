use crate::ui::{dispatch::UiInputManager, surface::UiSurface};
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    component::UiComponentEvent,
    dispatch::{
        UiDispatchDisposition, UiDispatchEffect, UiDispatchHostRequestKind, UiDispatchPhase,
        UiDispatchReply, UiInputEvent, UiInputEventMetadata, UiInputRoutePolicy, UiInputSequence,
        UiInputTimestamp, UiMouseMotionInputEvent, UiPointerEvent, UiPointerSource,
        UiPopupEffectKind, UiPopupInputEvent, UiPopupInputEventKind, UiTooltipEffectKind,
        UiTooltipTimerInputEvent, UiTooltipTimerInputEventKind, UiTransientDismissalReason,
        UiTransientDismissalTarget, UiWindowId,
    },
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiPoint, UiSize},
    surface::UiPointerEventKind,
    template::UiBindingRef,
    tree::{UiDirtyFlags, UiInputPolicy, UiTemplateNodeMetadata, UiTreeError, UiTreeNode},
    window::{
        UiWindowAction, UiWindowEvent, UiWindowEventKind, UiWindowEventMetadata,
        UiWindowInputContext, UiWindowInputPumpBatch, UiWindowInputPumpEvent, UiWindowMetrics,
        UiWindowPixelPosition, UiWindowPixelSize, UiWindowPlatformInputEvent, UiWindowRedrawReason,
    },
};

mod lifecycle;
mod metrics_dirty;
mod pointer_routes;

fn route_surface() -> UiSurface {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.window_input_pump"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_frame(UiFrame::new(0.0, 0.0, 160.0, 100.0))
            .with_state_flags(input_state()),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/first"))
                .with_frame(UiFrame::new(10.0, 10.0, 80.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/second"))
                .with_frame(UiFrame::new(10.0, 50.0, 80.0, 30.0))
                .with_input_policy(UiInputPolicy::Receive)
                .with_state_flags(input_state()),
        )
        .unwrap();
    surface.rebuild();
    surface
}

fn route_surface_with_hover_bindings() -> UiSurface {
    let mut surface = route_surface();
    for (node_id, control_id) in [
        (UiNodeId::new(2), "FirstHover"),
        (UiNodeId::new(3), "SecondHover"),
    ] {
        let target = surface.tree.nodes.get_mut(&node_id).unwrap();
        target.template_metadata = Some(UiTemplateNodeMetadata {
            component: "MaterialButton".to_string(),
            control_id: Some(control_id.to_string()),
            bindings: vec![binding(&format!("{control_id}/Hover"), UiEventKind::Hover)],
            ..Default::default()
        });
    }
    surface.rebuild();
    surface
}

fn dispatch_window_input_pump_event(
    surface: &mut UiSurface,
    event: UiWindowInputPumpEvent,
) -> Result<zircon_runtime_interface::ui::dispatch::UiInputDispatchResult, UiTreeError> {
    let mut manager = UiInputManager::default();
    surface.dispatch_window_input_pump_event(&mut manager, event)
}

fn dispatch_window_input_pump_batch(
    surface: &mut UiSurface,
    batch: UiWindowInputPumpBatch,
) -> Result<Vec<zircon_runtime_interface::ui::dispatch::UiInputDispatchResult>, UiTreeError> {
    let mut manager = UiInputManager::default();
    surface
        .dispatch_window_input_pump_batch(&mut manager, batch)
        .map(|outcome| outcome.results)
}

fn input_state() -> UiStateFlags {
    UiStateFlags {
        visible: true,
        enabled: true,
        clickable: true,
        hoverable: true,
        focusable: true,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

fn input_metadata() -> UiInputEventMetadata {
    UiInputEventMetadata::new(UiInputTimestamp::from_micros(10), UiInputSequence::new(1))
}

fn window_metadata(sequence: u64, synthetic: bool) -> UiWindowEventMetadata {
    UiWindowEventMetadata::for_window(
        UiWindowId::new("main-window"),
        UiInputTimestamp::from_micros(100 + sequence),
        UiInputSequence::new(sequence),
    )
    .synthetic(synthetic)
}

fn popup_event(kind: UiPopupInputEventKind, popup_id: &str, owner: UiNodeId) -> UiInputEvent {
    UiInputEvent::Popup(UiPopupInputEvent {
        metadata: input_metadata(),
        kind,
        popup_id: popup_id.to_string(),
        owner: Some(owner),
        anchor: Some(UiPoint::new(8.0, 12.0)),
    })
}

fn tooltip_event(
    kind: UiTooltipTimerInputEventKind,
    tooltip_id: &str,
    owner: UiNodeId,
) -> UiInputEvent {
    UiInputEvent::TooltipTimer(UiTooltipTimerInputEvent {
        metadata: input_metadata(),
        kind,
        tooltip_id: tooltip_id.to_string(),
        owner: Some(owner),
    })
}

fn raw_mouse_motion_event(delta_x: f32, delta_y: f32) -> UiInputEvent {
    UiInputEvent::MouseMotion(UiMouseMotionInputEvent {
        metadata: input_metadata(),
        delta_x,
        delta_y,
    })
}

fn binding(id: &str, event: UiEventKind) -> UiBindingRef {
    UiBindingRef {
        id: id.to_string(),
        event,
        route: Some(id.replace('/', ".")),
        action: None,
        targets: Vec::new(),
    }
}

fn open_popup(surface: &mut UiSurface, popup_id: &str, owner: UiNodeId) {
    surface.apply_dispatch_reply(
        popup_event(UiPopupInputEventKind::OpenRequested, popup_id, owner),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Popup {
            kind: UiPopupEffectKind::Open,
            popup_id: popup_id.to_string(),
            owner: Some(owner),
            anchor: Some(UiPoint::new(8.0, 12.0)),
        }),
    );
}

fn show_tooltip(surface: &mut UiSurface, tooltip_id: &str, owner: UiNodeId) {
    surface.apply_dispatch_reply(
        tooltip_event(UiTooltipTimerInputEventKind::Elapsed, tooltip_id, owner),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Tooltip {
            kind: UiTooltipEffectKind::Show,
            tooltip_id: tooltip_id.to_string(),
            owner: Some(owner),
        }),
    );
}
