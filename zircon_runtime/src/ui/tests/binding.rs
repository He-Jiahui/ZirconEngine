use crate::ui::binding::{
    UiBindingCall, UiBindingValue, UiEventBinding, UiEventKind, UiEventPath, UiEventRouter,
};

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
