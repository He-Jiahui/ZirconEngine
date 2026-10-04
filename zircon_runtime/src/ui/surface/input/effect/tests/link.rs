use zircon_runtime_interface::ui::{
    dispatch::UiDispatchEffect,
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::UiFrame,
    tree::UiTreeNode,
};

use crate::ui::surface::UiSurface;

use super::apply_link_activation_effect;

#[test]
fn rich_link_effect_accepts_a_typed_target_for_a_valid_owner() {
    use zircon_runtime_interface::ui::text::UiRichLinkTarget;

    let target = UiNodeId::new(9);
    let mut surface = UiSurface::new(UiTreeId::new("runtime.rich-link-effect"));
    surface.tree.insert_root(
        UiTreeNode::new(target, UiNodePath::new("root/link"))
            .with_frame(UiFrame::new(0.0, 0.0, 80.0, 20.0)),
    );

    let applied = apply_link_activation_effect(
        &surface,
        &UiDispatchEffect::RequestLinkActivation {
            target,
            link_target: UiRichLinkTarget::parse("res://docs/guide.zui").unwrap(),
        },
    )
    .unwrap();

    assert_eq!(applied, Some(target));
}
