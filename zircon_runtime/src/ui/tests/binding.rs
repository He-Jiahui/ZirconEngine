use crate::ui::binding::{
    binding_update_report, component_state_value_update, rejected_widget_alias_update,
    retained_attribute_update, runtime_state_update_with_source_kind, UiEventRouter,
};
use zircon_runtime_interface::ui::binding::{
    UiBindingCall, UiBindingDirtyDomain, UiBindingSourceKind, UiBindingTargetKind,
    UiBindingUpdateStatus, UiBindingValue, UiEventBinding, UiEventKind, UiEventPath,
};
use zircon_runtime_interface::ui::{component::UiValue, event_ui::UiNodeId, tree::UiDirtyFlags};

#[derive(Clone, Debug, PartialEq, Eq)]
enum MockUiCommand {
    ActivateCell { section_path: String, index: u32 },
}

#[test]
fn native_binding_roundtrip_preserves_generic_ui_contract() {
    let binding = UiEventBinding::new(
        UiEventPath::new("ExampleView", "PrimaryButton", UiEventKind::Click),
        UiBindingCall::new("ActivateCell")
            .with_argument(UiBindingValue::string("panel/main"))
            .with_argument(UiBindingValue::unsigned(24)),
    );

    assert_eq!(
        binding.native_binding(),
        r#"ExampleView/PrimaryButton:onClick(ActivateCell("panel/main",24))"#
    );
    assert_eq!(
        UiEventBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn drop_binding_roundtrip_preserves_reference_payload_contract() {
    let binding = UiEventBinding::new(
        UiEventPath::new("ReferenceField", "AssetSlot", UiEventKind::Drop),
        UiBindingCall::new("DropReference")
            .with_argument(UiBindingValue::string("asset"))
            .with_argument(UiBindingValue::string("res://textures/grid.albedo.png")),
    );

    assert_eq!(
        binding.native_binding(),
        r#"ReferenceField/AssetSlot:onDrop(DropReference("asset","res://textures/grid.albedo.png"))"#
    );
    assert_eq!(
        UiEventBinding::parse_native_binding(&binding.native_binding()).unwrap(),
        binding
    );
}

#[test]
fn headless_router_dispatches_native_binding_without_ui_runtime() {
    let binding = UiEventBinding::new(
        UiEventPath::new("ExampleView", "PrimaryButton", UiEventKind::Click),
        UiBindingCall::new("ActivateCell")
            .with_argument(UiBindingValue::string("panel/main"))
            .with_argument(UiBindingValue::unsigned(24)),
    );
    let mut router = UiEventRouter::<MockUiCommand>::default();
    router.register_exact(binding.path.clone(), |binding| {
        let action = binding.action.as_ref().expect("action payload");
        MockUiCommand::ActivateCell {
            section_path: action
                .argument(0)
                .and_then(UiBindingValue::as_str)
                .unwrap()
                .to_string(),
            index: action.argument(1).and_then(UiBindingValue::as_u32).unwrap(),
        }
    });

    assert_eq!(
        router.dispatch(&binding),
        vec![MockUiCommand::ActivateCell {
            section_path: "panel/main".to_string(),
            index: 24,
        }]
    );
}

#[test]
fn binding_update_helpers_classify_runtime_state_and_attribute_writes() {
    let node_id = UiNodeId::new(44);
    let state_update = component_state_value_update(
        node_id,
        "selected",
        Some(UiValue::Bool(false)),
        UiValue::Bool(true),
        UiDirtyFlags {
            render: true,
            input: true,
            ..UiDirtyFlags::default()
        },
        UiBindingUpdateStatus::Applied,
    );
    let attribute_update = retained_attribute_update(
        node_id,
        "label",
        None,
        UiValue::String("Run".to_string()),
        UiDirtyFlags {
            layout: true,
            render: true,
            text: true,
            ..UiDirtyFlags::default()
        },
        UiBindingUpdateStatus::Unchanged,
    );
    let rejected = rejected_widget_alias_update(
        node_id,
        "value",
        UiValue::String("bad".to_string()),
        "value alias is read-only",
    );
    let scroll_update = runtime_state_update_with_source_kind(
        UiNodeId::new(45),
        "scroll_target",
        UiBindingSourceKind::WidgetBehavior,
        node_id,
        "scroll_offset",
        Some(UiValue::Float(0.0)),
        UiValue::Float(100.0),
        UiDirtyFlags {
            layout: true,
            hit_test: true,
            render: true,
            input: true,
            ..UiDirtyFlags::default()
        },
        UiBindingUpdateStatus::Applied,
        None,
    );
    let report = binding_update_report(vec![
        state_update.clone(),
        attribute_update.clone(),
        rejected.clone(),
        scroll_update.clone(),
    ]);

    assert_eq!(state_update.source.kind, UiBindingSourceKind::RuntimeState);
    assert_eq!(
        state_update.target.kind,
        UiBindingTargetKind::ComponentStateValue
    );
    assert_eq!(
        state_update.dirty,
        vec![UiBindingDirtyDomain::Render, UiBindingDirtyDomain::Input]
    );
    assert_eq!(
        attribute_update.source.kind,
        UiBindingSourceKind::RetainedAttribute
    );
    assert_eq!(
        attribute_update.target.kind,
        UiBindingTargetKind::RetainedAttribute
    );
    assert_eq!(rejected.status, UiBindingUpdateStatus::Rejected);
    assert_eq!(
        scroll_update.source.kind,
        UiBindingSourceKind::WidgetBehavior
    );
    assert_eq!(scroll_update.target.kind, UiBindingTargetKind::RuntimeState);
    assert_eq!(report.applied_count, 2);
    assert_eq!(report.unchanged_count, 1);
    assert_eq!(report.rejected_count, 1);
    assert!(report.dirty.contains(&UiBindingDirtyDomain::Layout));
    assert!(report.dirty.contains(&UiBindingDirtyDomain::Render));
}
