use super::*;

#[test]
fn canonical_action_identity_resolves_independently_of_binding_path_and_keeps_fallback() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    let route = bridge
        .template_surface
        .host_projection
        .nodes
        .iter_mut()
        .flat_map(|node| node.routes.iter_mut())
        .next()
        .expect("authored workbench route");
    let binding_id = route.binding_id.clone();
    let fallback = binding_path_action_id(&binding_id);
    let canonical = "regression.canonical_registered_action";
    assert_ne!(fallback, canonical);
    route.action_id = canonical.to_string();
    assert_eq!(
        bridge.binding_id_for_action_id(canonical).as_deref(),
        Some(binding_id.as_str())
    );
    assert_eq!(
        bridge.binding_id_for_action_id(&fallback).as_deref(),
        Some(binding_id.as_str())
    );
    assert_eq!(
        bridge.binding_id_for_action_id(&binding_id).as_deref(),
        Some(binding_id.as_str())
    );
    assert_eq!(bridge.binding_id_for_action_id("missing.action"), None);
}
