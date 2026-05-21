use crate::ui::{
    accessibility::{
        UiA11yCheckedState, UiA11yRole, UiA11yState, UiAccessibilityAction,
        UiAccessibilityDiagnostic, UiAccessibilityDiagnosticCode,
        UiAccessibilityDiagnosticSeverity, UiAccessibilityNode, UiAccessibilityTreeSnapshot,
    },
    binding::{
        UiBindingDirtyDomain, UiBindingSource, UiBindingSourceKind, UiBindingTarget,
        UiBindingTargetKind, UiBindingUpdate, UiBindingUpdateReport, UiBindingUpdateStatus,
    },
    component::UiValue,
    dispatch::{
        UiDispatchReply, UiInputDispatchResult, UiInputEvent, UiInputEventMetadata,
        UiNavigationDispatchResult, UiPointerDispatchResult, UiTextInputEvent,
    },
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    focus::{
        UiFocusChangeEvent, UiFocusChangeReason, UiFocusVisible, UiFocusVisibleReason,
        UiFocusedInput, UiFocusedInputKind, UiInputFocus,
    },
    layout::{UiAxis, UiFrame, UiPoint},
    navigation::{
        UiDirectionalNavigation, UiDirectionalNavigationTarget, UiNavigationGroup,
        UiNavigationGroupId, UiTabIndex,
    },
    picking::{UiPickMode, UiPickPolicy, UiPointerCapture, UiPointerCaptureKind},
    style::{
        ButtonColor, ButtonDimension, ButtonEventKind, ButtonIconPlacement, ButtonInteractionState,
        ButtonSize, ButtonVariant, ResolvedButtonStyle, StyleDimension, UiResolvedElementStyle,
        UiRgbaColor, UiStyleColor,
    },
    surface::{
        UiEditableTextState, UiNavigationEventKind, UiNavigationRoute, UiPointerActivationPhase,
        UiPointerEventKind, UiPointerRoute, UiRenderCommand, UiRenderCommandKind, UiRenderExtract,
        UiRenderExtractKind, UiRenderList, UiRenderStats, UiResolvedStyle, UiTextCaret,
        UiTextEditAction, UiTextSelection,
    },
    template::{UiAssetDocument, UiTemplateDocument, UiTemplateNode},
    text::{UiTextCursorStyle, UiTextEdit, UiTextEditSource},
    tree::UiDirtyFlags,
    widget::{
        UiWidgetBehavior, UiWidgetContract, UiWidgetEvent, UiWidgetEventKind, UiWidgetEventSource,
    },
};

fn round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    serde_json::from_str(&serde_json::to_string(value).unwrap()).unwrap()
}

#[test]
fn ui_typed_style_and_button_contracts_round_trip_with_material_defaults() {
    let custom = UiRgbaColor::from_u8(12, 34, 56, 200);
    let element = UiResolvedElementStyle {
        background_color: Some(UiStyleColor::Role("material.primary".to_string())),
        foreground_color: Some(UiStyleColor::Rgba(custom)),
        border_color: Some(UiStyleColor::Transparent),
        border_width: 1.0,
        corner_radius: 10.0,
        width: StyleDimension::Fill,
        height: StyleDimension::Fixed(40.0),
        opacity: 0.85,
    };
    let button = ResolvedButtonStyle {
        variant: ButtonVariant::Default,
        color: ButtonColor::Custom(custom),
        size: ButtonSize::Custom {
            width: ButtonDimension::Fill,
            height: ButtonDimension::Fixed(44.0),
        },
        width: ButtonDimension::Fill,
        height: ButtonDimension::Fixed(44.0),
        icon_placement: ButtonIconPlacement::Start,
        interaction_state: ButtonInteractionState::Pressed,
        loading: true,
        disabled: true,
        element,
    };

    assert_eq!(custom.to_u8(), [12, 34, 56, 200]);
    assert_eq!(ButtonVariant::Default.normalized(), ButtonVariant::Text);
    assert_eq!(round_trip(&button), button);
    assert_eq!(
        ButtonVariant::OPTIONS,
        ["default", "text", "contained", "outlined"]
    );
    assert_eq!(ButtonColor::OPTIONS[0], "default");
    assert_eq!(ButtonSize::OPTIONS, ["small", "medium", "large"]);
    assert_eq!(ButtonIconPlacement::OPTIONS[3], "icon_only");
    assert_eq!(round_trip(&ButtonEventKind::Enter), ButtonEventKind::Enter);

    let default_button = ResolvedButtonStyle::default();
    assert_eq!(default_button.variant, ButtonVariant::Text);
    assert_eq!(default_button.color, ButtonColor::Primary);
    assert_eq!(default_button.size, ButtonSize::Medium);
    assert_eq!(default_button.width, StyleDimension::Auto);
}

#[test]
fn ui_focus_navigation_and_picking_contracts_round_trip_with_defaults() {
    let focused = UiNodeId::new(7);
    let previous = UiNodeId::new(3);
    let focus = UiInputFocus {
        focused: Some(focused),
        previous: Some(previous),
        pending_autofocus: Some(focused),
        focus_visible: UiFocusVisible::visible(UiFocusVisibleReason::KeyboardNavigation),
    };
    let change = UiFocusChangeEvent {
        previous: Some(previous),
        current: Some(focused),
        reason: UiFocusChangeReason::Navigation,
        visible: focus.focus_visible,
    };
    let focused_input = UiFocusedInput {
        focused,
        kind: UiFocusedInputKind::Keyboard,
        route: vec![UiNodeId::new(1), focused],
        handled_by: Some(focused),
        accepted: true,
    };
    let group = UiNavigationGroup {
        group_id: UiNavigationGroupId::new("dialog.primary"),
        parent: None,
        root: Some(UiNodeId::new(1)),
        modal: true,
        wrap: true,
        order: 2,
    };
    let directional = UiDirectionalNavigation {
        up: UiDirectionalNavigationTarget::Blocked,
        down: UiDirectionalNavigationTarget::Node(UiNodeId::new(9)),
        left: UiDirectionalNavigationTarget::Auto,
        right: UiDirectionalNavigationTarget::Group(group.group_id.clone()),
    };
    let pick_policy = UiPickPolicy::receive()
        .with_focus(UiPickMode::Receive)
        .with_accessibility(UiPickMode::Receive)
        .with_text_hit(true);
    let capture = UiPointerCapture {
        owner: Some(focused),
        pointer_id: Some(crate::ui::dispatch::UiPointerId::new(4)),
        kind: UiPointerCaptureKind::Drag,
        active: true,
    };

    assert_eq!(round_trip(&focus), focus);
    assert_eq!(round_trip(&change), change);
    assert_eq!(round_trip(&focused_input), focused_input);
    assert_eq!(round_trip(&UiTabIndex::new(4)), UiTabIndex::new(4));
    assert_eq!(round_trip(&group), group);
    assert_eq!(round_trip(&directional), directional);
    assert_eq!(round_trip(&pick_policy), pick_policy);
    assert_eq!(round_trip(&capture), capture);

    let default_focus: UiInputFocus = serde_json::from_str("{}").unwrap();
    let default_tab: UiTabIndex = serde_json::from_str("{}").unwrap();
    let default_policy: UiPickPolicy = serde_json::from_str("{}").unwrap();
    assert_eq!(default_focus.focused, None);
    assert!(!default_focus.focus_visible.visible);
    assert!(!default_tab.tabbable);
    assert_eq!(default_policy.pointer, UiPickMode::Inherit);
}

#[test]
fn ui_accessibility_snapshot_represents_roles_states_actions_and_diagnostics() {
    let button = UiNodeId::new(11);
    let label = UiNodeId::new(10);
    let node = UiAccessibilityNode {
        node_id: button,
        node_path: Some(UiNodePath::new("root/actions/save")),
        role: UiA11yRole::Button,
        name: Some("Save".to_string()),
        description: Some("Write the current asset".to_string()),
        bounds: Some(UiFrame::new(8.0, 12.0, 80.0, 24.0)),
        state: UiA11yState {
            focused: true,
            checked: Some(UiA11yCheckedState::False),
            ..UiA11yState::default()
        },
        actions: vec![
            UiAccessibilityAction::Activate,
            UiAccessibilityAction::Focus,
        ],
        children: Vec::new(),
        labelled_by: Some(label),
        label_for: None,
        tooltip: Some("Save asset".to_string()),
    };
    let missing_name = UiAccessibilityDiagnostic {
        severity: UiAccessibilityDiagnosticSeverity::Warning,
        code: UiAccessibilityDiagnosticCode::MissingName,
        node_id: Some(UiNodeId::new(12)),
        message: "interactive node has no accessible name".to_string(),
    };
    let snapshot = UiAccessibilityTreeSnapshot {
        tree_id: UiTreeId::new("ui.a11y"),
        roots: vec![button],
        nodes: vec![node.clone()],
        focused: Some(button),
        diagnostics: vec![missing_name.clone()],
    };

    let round_tripped = round_trip(&snapshot);
    assert_eq!(round_tripped.node(button), Some(&node));
    assert_eq!(round_tripped.focused, Some(button));
    assert_eq!(round_tripped.diagnostics[0], missing_name);
    assert!(serde_json::to_string(&round_tripped)
        .unwrap()
        .contains("missing_name"));

    let empty_node: UiAccessibilityNode = serde_json::from_str("{\"node_id\":13}").unwrap();
    assert_eq!(empty_node.role, UiA11yRole::Generic);
    assert!(empty_node.actions.is_empty());
}

#[test]
fn ui_widget_text_and_cursor_contracts_serialize_typed_events() {
    let node_id = UiNodeId::new(21);
    let before = UiEditableTextState {
        text: "Hello".to_string(),
        caret: UiTextCaret::default(),
        selection: Some(UiTextSelection::collapsed(0)),
        composition: None,
        read_only: false,
    };
    let after = UiEditableTextState {
        text: "Hello!".to_string(),
        caret: UiTextCaret {
            offset: 6,
            ..UiTextCaret::default()
        },
        selection: Some(UiTextSelection::collapsed(6)),
        composition: None,
        read_only: false,
    };
    let edit = UiTextEdit {
        node_id,
        source: UiTextEditSource::Keyboard,
        action: UiTextEditAction::Insert {
            text: "!".to_string(),
        },
        before: before.clone(),
        after: after.clone(),
    };
    let events = vec![
        UiWidgetEvent::Activate {
            target: node_id,
            source: UiWidgetEventSource::Keyboard,
            action_id: Some("save".to_string()),
        },
        UiWidgetEvent::ValueChange {
            target: node_id,
            property: "value".to_string(),
            previous: Some(UiValue::String("old".to_string())),
            value: UiValue::String("new".to_string()),
            source: UiWidgetEventSource::Pointer,
        },
        UiWidgetEvent::TextEditChange { edit: edit.clone() },
        UiWidgetEvent::OpenChanged {
            target: node_id,
            open: true,
            source: UiWidgetEventSource::Pointer,
        },
        UiWidgetEvent::SelectionChanged {
            target: node_id,
            selection: vec![UiValue::String("item.a".to_string())],
            source: UiWidgetEventSource::Navigation,
        },
    ];
    let cursor = UiTextCursorStyle {
        width: 2.0,
        color: Some("#ffffff".to_string()),
        blink_period_millis: Some(600),
        visible: true,
    };
    let widget = UiWidgetContract {
        behavior: UiWidgetBehavior::Range,
        value_property: Some("amount".to_string()),
        min_property: Some("minimum".to_string()),
        max_property: Some("maximum".to_string()),
        step_property: Some("interval".to_string()),
        ..UiWidgetContract::default()
    };
    let scrollbar = UiWidgetContract {
        behavior: UiWidgetBehavior::Scrollbar,
        scroll_target: Some("#42".to_string()),
        scroll_axis: Some(UiAxis::Vertical),
        min_thumb_extent: Some(18.0),
        ..UiWidgetContract::default()
    };

    let round_tripped_events = round_trip(&events);
    assert_eq!(round_tripped_events, events);
    assert_eq!(events[0].kind(), UiWidgetEventKind::Activate);
    assert_eq!(events[2].kind(), UiWidgetEventKind::TextEditChange);
    assert_eq!(round_trip(&edit), edit);
    assert_eq!(round_trip(&cursor), cursor);
    assert_eq!(round_trip(&widget), widget);
    assert_eq!(round_trip(&scrollbar), scrollbar);
    assert_eq!(
        widget.resolved_behavior("CustomMeter"),
        UiWidgetBehavior::Range
    );
    assert_eq!(
        UiWidgetContract::default().resolved_behavior("TextField"),
        UiWidgetBehavior::TextInput
    );
    assert_eq!(
        UiWidgetContract::default().resolved_behavior("RadioButton"),
        UiWidgetBehavior::Radio
    );
    assert_eq!(
        UiWidgetContract::default().resolved_behavior("RadioGroup"),
        UiWidgetBehavior::RadioGroup
    );
    assert_eq!(
        UiWidgetContract::default().resolved_behavior("ScrollBar"),
        UiWidgetBehavior::Scrollbar
    );
    assert_eq!(
        UiWidgetContract::default().resolved_behavior("ScrollbarThumb"),
        UiWidgetBehavior::ScrollbarThumb
    );
    assert_eq!(UiTextCursorStyle::default().width, 1.0);
    assert!(serde_json::to_string(&round_tripped_events)
        .unwrap()
        .contains("selection_changed"));
}

#[test]
fn ui_binding_update_contract_represents_attribute_state_and_ecs_domains() {
    let node_id = UiNodeId::new(31);
    let update = UiBindingUpdate::applied(
        UiBindingSource::retained_attribute(node_id, "selected"),
        UiBindingTarget::component_state_value(node_id, "selected"),
        UiValue::Bool(true),
    )
    .with_previous(Some(UiValue::Bool(false)))
    .with_dirty_flags(UiDirtyFlags {
        render: true,
        input: true,
        ..UiDirtyFlags::default()
    });
    let ecs_update = UiBindingUpdate::unchanged(
        UiBindingSource::runtime_ecs("scene.entity:42/Transform.translation"),
        UiBindingTarget::runtime_ecs("ui.node:31/runtime_position"),
        UiValue::Vec3([1.0, 2.0, 3.0]),
    )
    .with_dirty([UiBindingDirtyDomain::Schedule]);
    let rejected = UiBindingUpdate::rejected(
        UiBindingSource::accessibility_action(node_id, "value"),
        UiBindingTarget::widget_alias(node_id, "value"),
        UiValue::String("bad".to_string()),
        "value alias is read-only",
    );
    let report =
        UiBindingUpdateReport::from_updates(vec![update.clone(), ecs_update.clone(), rejected]);

    assert_eq!(round_trip(&update), update);
    assert_eq!(update.source.kind, UiBindingSourceKind::RetainedAttribute);
    assert_eq!(update.target.kind, UiBindingTargetKind::ComponentStateValue);
    assert_eq!(
        update.dirty,
        vec![UiBindingDirtyDomain::Render, UiBindingDirtyDomain::Input]
    );
    assert_eq!(ecs_update.source.kind, UiBindingSourceKind::RuntimeEcs);
    assert_eq!(ecs_update.target.kind, UiBindingTargetKind::RuntimeEcs);
    let runtime_target = UiBindingTarget::runtime_state(node_id, "scroll_offset");
    assert_eq!(round_trip(&runtime_target), runtime_target);
    assert_eq!(runtime_target.kind, UiBindingTargetKind::RuntimeState);
    assert_eq!(report.applied_count, 1);
    assert_eq!(report.unchanged_count, 1);
    assert_eq!(report.rejected_count, 1);
    assert!(report.dirty.contains(&UiBindingDirtyDomain::Render));
    assert!(report.dirty.contains(&UiBindingDirtyDomain::Schedule));
    assert!(serde_json::to_string(&report)
        .unwrap()
        .contains("component_state_value"));
    assert_eq!(
        UiBindingUpdateStatus::default(),
        UiBindingUpdateStatus::Applied
    );

    let mut dispatch_result = UiInputDispatchResult::new(
        UiInputEvent::Text(UiTextInputEvent {
            metadata: UiInputEventMetadata::default(),
            text: "commit".to_string(),
        }),
        UiDispatchReply::handled(),
    );
    dispatch_result.record_binding_report(report.clone());
    assert_eq!(round_trip(&dispatch_result), dispatch_result);
    assert_eq!(dispatch_result.binding_reports, vec![report.clone()]);

    let mut legacy_dispatch = serde_json::to_value(&dispatch_result).unwrap();
    legacy_dispatch
        .as_object_mut()
        .unwrap()
        .remove("binding_reports");
    let legacy_dispatch: UiInputDispatchResult = serde_json::from_value(legacy_dispatch).unwrap();
    assert!(legacy_dispatch.binding_reports.is_empty());

    let mut pointer_result = UiPointerDispatchResult::new(UiPointerRoute {
        kind: UiPointerEventKind::Down,
        button: None,
        activation_phase: UiPointerActivationPhase::PrimaryPress,
        point: UiPoint::default(),
        scroll_delta: 0.0,
        target: Some(node_id),
        hit_path: Default::default(),
        bubbled: vec![node_id],
        stacked: vec![node_id],
        entered: Vec::new(),
        left: Vec::new(),
        captured: None,
        pressed: Some(node_id),
        click_target: None,
        release_inside_pressed: false,
        focused: Some(node_id),
        fallback_to_root: false,
        root_targets: Vec::new(),
    });
    pointer_result.record_binding_report(report.clone());
    assert_eq!(round_trip(&pointer_result), pointer_result);
    assert_eq!(pointer_result.binding_reports, vec![report.clone()]);

    let mut legacy_pointer = serde_json::to_value(&pointer_result).unwrap();
    legacy_pointer
        .as_object_mut()
        .unwrap()
        .remove("binding_reports");
    let legacy_pointer: UiPointerDispatchResult = serde_json::from_value(legacy_pointer).unwrap();
    assert!(legacy_pointer.binding_reports.is_empty());

    let mut navigation_result = UiNavigationDispatchResult::new(UiNavigationRoute {
        kind: UiNavigationEventKind::Activate,
        target: Some(node_id),
        bubbled: vec![node_id],
        fallback_to_root: false,
        root_targets: Vec::new(),
    });
    navigation_result.record_binding_report(report.clone());
    assert_eq!(round_trip(&navigation_result), navigation_result);
    assert_eq!(navigation_result.binding_reports, vec![report]);

    let mut legacy_navigation = serde_json::to_value(&navigation_result).unwrap();
    legacy_navigation
        .as_object_mut()
        .unwrap()
        .remove("binding_reports");
    let legacy_navigation: UiNavigationDispatchResult =
        serde_json::from_value(legacy_navigation).unwrap();
    assert!(legacy_navigation.binding_reports.is_empty());
}

#[test]
fn ui_render_extract_kind_and_stats_contract_does_not_break_legacy_extracts() {
    let extract = UiRenderExtract {
        tree_id: UiTreeId::new("ui.render.m1"),
        list: UiRenderList {
            commands: vec![
                UiRenderCommand {
                    node_id: UiNodeId::new(1),
                    kind: UiRenderCommandKind::Quad,
                    frame: UiFrame::new(0.0, 0.0, 10.0, 10.0),
                    clip_frame: None,
                    z_index: 0,
                    style: UiResolvedStyle::default(),
                    text_layout: None,
                    text: None,
                    image: None,
                    opacity: 1.0,
                },
                UiRenderCommand {
                    node_id: UiNodeId::new(2),
                    kind: UiRenderCommandKind::Text,
                    frame: UiFrame::new(0.0, 12.0, 40.0, 16.0),
                    clip_frame: None,
                    z_index: 1,
                    style: UiResolvedStyle::default(),
                    text_layout: None,
                    text: Some("Label".to_string()),
                    image: None,
                    opacity: 1.0,
                },
            ],
        },
    };
    let stats = UiRenderStats::from_extract(&extract);

    assert_eq!(
        UiRenderExtractKind::default(),
        UiRenderExtractKind::LegacyCommandList
    );
    assert_eq!(stats.command_count, 2);
    assert_eq!(stats.text_command_count, 1);
    assert_eq!(stats.image_command_count, 0);
    assert_eq!(round_trip(&stats), stats);

    let legacy_extract: UiRenderExtract =
        serde_json::from_str(r#"{"tree_id":"legacy","list":{"commands":[]}}"#).unwrap();
    let missing_stats: UiRenderStats = serde_json::from_str("{}").unwrap();
    assert_eq!(legacy_extract.list.commands.len(), 0);
    assert_eq!(missing_stats.command_count, 0);
}

#[test]
fn ui_template_contract_sections_are_defaulted_for_legacy_toml() {
    let legacy: UiTemplateDocument = toml::from_str(
        r#"
version = 1

[root]
component = "Button"
control_id = "ok"
"#,
    )
    .unwrap();

    assert_eq!(legacy.root.component.as_deref(), Some("Button"));
    assert!(!legacy.root.focus.focusable);
    assert!(legacy.root.navigation.tab_index.is_none());
    assert_eq!(legacy.root.picking.pointer, UiPickMode::Inherit);
    assert_eq!(legacy.root.a11y.role, UiA11yRole::Generic);
    assert!(!legacy.root.widget.disabled);

    let modern: UiTemplateDocument = toml::from_str(
        r#"
version = 1

[root]
component = "TextInput"
control_id = "search"

[root.focus]
focusable = true
autofocus = true

[root.navigation.tab_index]
order = 3
tabbable = true

[root.picking]
pointer = "receive"
focus = "receive"
accessibility = "receive"
text_hit = true

[root.a11y]
role = "text_input"
name = "Search"
tooltip = "Search assets"

[root.widget]
behavior = "text_input"
disabled = false
checked = false
tooltip = "Search assets"
"#,
    )
    .unwrap();

    assert!(modern.root.focus.focusable);
    assert!(modern.root.focus.autofocus);
    assert_eq!(modern.root.navigation.tab_index, Some(UiTabIndex::new(3)));
    assert_eq!(modern.root.picking.pointer, UiPickMode::Receive);
    assert_eq!(modern.root.a11y.role, UiA11yRole::TextInput);
    assert_eq!(modern.root.a11y.name.as_deref(), Some("Search"));
    assert_eq!(modern.root.widget.checked, Some(false));
    assert_eq!(modern.root.widget.behavior, UiWidgetBehavior::TextInput);

    let mut authored = UiTemplateNode {
        component: Some("Checkbox".to_string()),
        ..UiTemplateNode::default()
    };
    authored.focus.focusable = true;
    authored.widget.checked = Some(true);
    authored.picking = UiPickPolicy::receive();

    assert!(serde_json::to_string(&authored)
        .unwrap()
        .contains("focusable"));

    let legacy_asset: UiAssetDocument = toml::from_str(
        r#"
[asset]
kind = "layout"
id = "legacy.asset"

[root]
node_id = "root"
kind = "native"
type = "Button"
"#,
    )
    .unwrap();
    let legacy_asset_root = legacy_asset.root.as_ref().unwrap();
    assert!(legacy_asset_root.focus.is_none());
    assert!(legacy_asset_root.navigation.is_none());
    assert!(legacy_asset_root.picking.is_none());
    assert!(legacy_asset_root.a11y.is_none());
    assert!(legacy_asset_root.widget.is_none());

    let modern_asset: UiAssetDocument = toml::from_str(
        r#"
[asset]
kind = "layout"
id = "modern.asset"

[root]
node_id = "root"
kind = "native"
type = "TextInput"

[root.focus]
focusable = true

[root.navigation.tab_index]
order = 7
tabbable = true

[root.picking]
pointer = "receive"
text_hit = true

[root.a11y]
role = "text_input"
name = "Search"

[root.widget]
disabled = false
tooltip = "Search assets"
"#,
    )
    .unwrap();
    let modern_asset_root = modern_asset.root.as_ref().unwrap();
    assert!(modern_asset_root.focus.as_ref().unwrap().focusable);
    assert_eq!(
        modern_asset_root.navigation.as_ref().unwrap().tab_index,
        Some(UiTabIndex::new(7))
    );
    assert_eq!(
        modern_asset_root.picking.as_ref().unwrap().pointer,
        UiPickMode::Receive
    );
    assert_eq!(
        modern_asset_root.a11y.as_ref().unwrap().role,
        UiA11yRole::TextInput
    );
    assert_eq!(
        modern_asset_root
            .widget
            .as_ref()
            .unwrap()
            .tooltip
            .as_deref(),
        Some("Search assets")
    );
}
