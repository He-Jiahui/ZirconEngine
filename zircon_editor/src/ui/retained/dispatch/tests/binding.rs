use super::*;
use zircon_runtime_interface::ui::binding::UiBindingCall;

#[test]
fn material_component_lab_custom_binding_is_paint_only_feedback() {
    let binding = EditorUiBinding::new(
        "MaterialComponentLab",
        "MaterialLabButtons",
        crate::ui::binding::EditorUiEventKind::Click,
        EditorUiBindingPayload::Custom(UiBindingCall::new("MaterialComponentLab")),
    );

    let effects =
        material_lab_feedback_effects(&binding).expect("Material Lab payload should match");
    let dirty_domains = effects.dirty_domains();

    assert!(dirty_domains.contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!dirty_domains.requires_presentation());
    assert!(!dirty_domains.requires_layout());
}
