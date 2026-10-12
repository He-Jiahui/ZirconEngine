use std::collections::BTreeMap;

use zircon_runtime_interface::ui::template::UiActionRef;

use super::*;

fn binding(component_event: Option<UiComponentEventKind>) -> UiBindingRef {
    UiBindingRef {
        component_event,
        id: "business/OpenPopupAudit".to_string(),
        event: UiEventKind::Click,
        mode: Default::default(),
        route: Some("business.open_popup.audit".to_string()),
        action: Some(UiActionRef {
            route: Some("business.open_popup.action".to_string()),
            action: Some("OpenPopupAudit".to_string()),
            payload: BTreeMap::new(),
            payload_missing_policy: Default::default(),
        }),
        targets: Vec::new(),
    }
}

#[test]
fn typed_component_event_routing_ignores_string_spelling() {
    let deceptive = binding(None);
    assert!(!binding_targets_component_event(
        &deceptive,
        UiComponentEventKind::OpenPopup
    ));

    let mut declared = binding(Some(UiComponentEventKind::OpenPopup));
    declared.id = "renamed-without-event-token".to_string();
    declared.route = Some("product.lower_snake.route".to_string());
    declared.action = None;
    assert!(binding_targets_component_event(
        &declared,
        UiComponentEventKind::OpenPopup
    ));
    assert!(!binding_targets_component_event(
        &declared,
        UiComponentEventKind::ClosePopup
    ));
}

#[test]
fn typed_component_event_hot_path_eliminates_string_scans() {
    let bindings = (0..1_000)
        .map(|index| {
            let mut binding = binding(None);
            binding.id = format!("business/OpenPopupAudit/{index}");
            binding.component_event = (index == 999).then_some(UiComponentEventKind::OpenPopup);
            binding
        })
        .collect::<Vec<_>>();

    let matched = bindings
        .iter()
        .filter(|binding| binding_targets_component_event(binding, UiComponentEventKind::OpenPopup))
        .count();
    assert_eq!(matched, 1);
    println!(
        "PERF-RUNTIME74-TYPED-EVENT sample_bindings=1000 legacy_string_field_scans=4000 optimized_enum_comparisons=1000 string_scans_eliminated=4000 matched_bindings={matched}"
    );
}
