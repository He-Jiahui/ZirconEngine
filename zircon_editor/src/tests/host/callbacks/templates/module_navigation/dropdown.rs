use super::super::super::support::{
    dispatch_componentized_workbench_option_selected, env_lock,
    BuiltinWorkbenchWindowTemplateSurfaceBridge, EditorUiBindingPayload, EventRuntimeHarness,
    UiEventKind, UiSize,
};
use super::super::support::{control_bool, control_string};
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::retained_host::HostInvalidationMask;

#[test]
fn workbench_module_dropdowns_open_select_and_close_with_shared_dropdown_path() {
    let _guard = env_lock().lock().unwrap_or_else(|error| error.into_inner());

    let harness = EventRuntimeHarness::new("zircon_workbench_module_dropdown_select");
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .dispatch_control_state("WorkbenchModuleMaterial", UiEventKind::Click)
        .unwrap()
        .expect("material module tab should expose a preview binding");

    assert!(!control_bool(
        &bridge,
        "WorkbenchMaterialDomainDropdown",
        "popup_open"
    ));
    assert_eq!(
        control_string(&bridge, "WorkbenchMaterialDomainDropdown", "value").as_deref(),
        Some("surface")
    );

    let open_binding = bridge
        .dispatch_control_state("WorkbenchMaterialDomainDropdown", UiEventKind::Change)
        .unwrap()
        .expect("module dropdown should expose its field edit binding");
    assert!(matches!(
        open_binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id }
            if action_id == "workbench.module.material.domain.edit"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchMaterialDomainDropdown",
        "popup_open"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchMaterialDomainDropdown",
        "focused"
    ));
    assert!(control_bool(
        &bridge,
        "WorkbenchMaterialDomainDropdown",
        "selected"
    ));

    let effects = dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchMaterialDomainDropdown",
        "post_process",
    )
    .expect("module dropdown option selection should dispatch");

    assert_eq!(
        control_string(&bridge, "WorkbenchMaterialDomainDropdown", "value").as_deref(),
        Some("post_process")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchMaterialDomainDropdown", "value_text").as_deref(),
        Some("Post Process")
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchMaterialDomainDropdown")
            .expect("material domain dropdown projection after selection")
            .value_text
            .as_deref(),
        Some("Post Process")
    );
    assert!(!control_bool(
        &bridge,
        "WorkbenchMaterialDomainDropdown",
        "popup_open"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchMaterialDomainDropdown",
        "focused"
    ));
    assert!(!control_bool(
        &bridge,
        "WorkbenchMaterialDomainDropdown",
        "selected"
    ));
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert_eq!(harness.runtime.journal().records().len(), 1);

    let no_effects = dispatch_componentized_workbench_option_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchMaterialDomainDropdown",
        "unsupported_domain",
    )
    .expect("unknown module dropdown option should be swallowed");
    assert_eq!(
        control_string(&bridge, "WorkbenchMaterialDomainDropdown", "value").as_deref(),
        Some("post_process")
    );
    assert_eq!(harness.runtime.journal().records().len(), 1);
    assert_eq!(no_effects, UiHostEventEffects::default());
}
