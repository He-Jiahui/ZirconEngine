use zircon_runtime_interface::ZrRuntimeViewportHandle;

use crate::core::gateway::EditorRuntimeHighlightSet;

#[test]
fn canonicalizes_entity_ids_before_crossing_gateway() {
    let set = EditorRuntimeHighlightSet::new(
        ZrRuntimeViewportHandle::new(2),
        4,
        [7, 1, 7, 3],
        true,
        [0.3, 0.5, 0.8, 1.0],
    );

    assert_eq!(set.entities(), &[1, 3, 7]);
}

#[test]
fn highlight_set_production_owner_has_no_inline_test_module() {
    let source = include_str!("../../core/gateway/highlight_set.rs");

    assert!(
        !source.contains("#[cfg(test)]"),
        "highlight-set production owner must not retain an inline test module"
    );
}
