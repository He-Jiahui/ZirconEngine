use zircon_runtime_interface::{
    resource::{ResourceLocator, ResourceScheme},
    ui::{dispatch::UiDispatchEffect, event_ui::UiNodeId},
};

use super::super::super::surface::UiSurface;
use super::super::{
    require_valid_input_owner, UiSurfaceInputEffectError, UiSurfaceInputEffectResult,
};

pub(super) fn apply_link_activation_effect(
    surface: &UiSurface,
    effect: &UiDispatchEffect,
) -> UiSurfaceInputEffectResult<Option<UiNodeId>> {
    let UiDispatchEffect::RequestLinkActivation { target, href } = effect else {
        return Err(UiSurfaceInputEffectError::UnexpectedEffect {
            expected: "rich link activation",
        });
    };
    require_valid_input_owner(surface, *target)?;
    let locator = ResourceLocator::parse(href)
        .map_err(|_| UiSurfaceInputEffectError::InvalidRichLinkTarget { href: href.clone() })?;
    if !matches!(
        locator.scheme(),
        ResourceScheme::Res
            | ResourceScheme::Library
            | ResourceScheme::Package
            | ResourceScheme::Builtin
    ) {
        return Err(UiSurfaceInputEffectError::InvalidRichLinkTarget { href: href.clone() });
    }
    Ok(Some(*target))
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{
        dispatch::UiDispatchEffect,
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        layout::UiFrame,
        tree::UiTreeNode,
    };

    use crate::ui::surface::UiSurface;

    use super::{apply_link_activation_effect, UiSurfaceInputEffectError};

    #[test]
    fn rich_link_effect_rejects_network_scheme_even_for_valid_owner() {
        let target = UiNodeId::new(9);
        let mut surface = UiSurface::new(UiTreeId::new("runtime.rich-link-effect"));
        surface.tree.insert_root(
            UiTreeNode::new(target, UiNodePath::new("root/link"))
                .with_frame(UiFrame::new(0.0, 0.0, 80.0, 20.0)),
        );

        let error = apply_link_activation_effect(
            &surface,
            &UiDispatchEffect::RequestLinkActivation {
                target,
                href: "https://example.com/escape".to_string(),
            },
        )
        .expect_err("network links must not cross the rich-link host boundary");

        assert_eq!(
            error,
            UiSurfaceInputEffectError::InvalidRichLinkTarget {
                href: "https://example.com/escape".to_string(),
            }
        );
    }
}
