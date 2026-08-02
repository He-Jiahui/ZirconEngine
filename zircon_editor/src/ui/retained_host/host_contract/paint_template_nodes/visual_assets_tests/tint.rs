use super::super::{
    ICON_TINT_ACTIVE, ICON_TINT_DISABLED, ICON_TINT_ERROR, ICON_TINT_WARNING, template_image_tint,
};

#[test]
fn template_icon_tint_uses_material_state_priority() {
    assert_eq!(
        template_image_tint(true, true, true, "error", "error", Some(ICON_TINT_ACTIVE)),
        Some(ICON_TINT_DISABLED)
    );
    assert_eq!(
        template_image_tint(true, true, false, "", "error", Some(ICON_TINT_ACTIVE)),
        Some(ICON_TINT_ERROR)
    );
    assert_eq!(
        template_image_tint(
            true,
            true,
            false,
            "warning",
            "normal",
            Some(ICON_TINT_ACTIVE),
        ),
        Some(ICON_TINT_WARNING)
    );
    assert_eq!(
        template_image_tint(true, true, false, "", "normal", Some(ICON_TINT_ERROR)),
        Some(ICON_TINT_ERROR)
    );
    assert_eq!(
        template_image_tint(true, true, false, "", "normal", None),
        Some(ICON_TINT_ACTIVE)
    );
    assert_eq!(
        template_image_tint(false, true, false, "error", "error", Some(ICON_TINT_ERROR)),
        None
    );
}
