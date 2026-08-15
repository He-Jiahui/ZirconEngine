const EDITOR_TOKENS_ASSET: &str =
    include_str!("../../../../assets/ui/editor/theme/editor_tokens.zui");
const NOTIFICATION_CENTER_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/primitives/feedback/workbench_notification_center.zui"
);

#[test]
fn notification_center_uses_shared_density_constraints_for_its_popup_surface() {
    for token in [
        "$editor.density.notification_panel.min_width",
        "$editor.density.notification_panel.preferred_width",
        "$editor.density.notification_panel.max_width",
        "$editor.density.notification_panel.min_height",
        "$editor.density.notification_panel.preferred_height",
        "$editor.density.notification_panel.max_height",
    ] {
        assert!(
            NOTIFICATION_CENTER_ASSET.contains(token),
            "notification popup must resolve `{token}` through the density cascade"
        );
        assert!(
            EDITOR_TOKENS_ASSET.contains(&token[1..]),
            "editor tokens must name `{token}` for V2 resolution"
        );
    }

    for local_constraint in [
        "min = 280.0, preferred = 320.0, max = 380.0",
        "min = 160.0, preferred = 220.0, max = 320.0",
    ] {
        assert!(
            !NOTIFICATION_CENTER_ASSET.contains(local_constraint),
            "notification popup must not retain the local constraint `{local_constraint}`"
        );
    }
}
