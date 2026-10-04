//! 图标与活动栏控件的静止外观和状态反馈保持共享material声明。
const WORKBENCH_ICON_BUTTON_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/primitives/inputs/workbench_icon_button.zui"
);
const WORKBENCH_RAIL_BUTTON_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/primitives/chrome/workbench_rail_button.zui"
);

#[test]
/// 透明静止表面与hover、选中和强调token须共同保留；未模拟指针或焦点状态。
fn text_icon_controls_keep_resting_material_quiet_and_state_feedback_explicit() {
    for (asset_name, asset) in [
        ("workbench_icon_button.zui", WORKBENCH_ICON_BUTTON_ASSET),
        ("workbench_rail_button.zui", WORKBENCH_RAIL_BUTTON_ASSET),
    ] {
        assert!(
            asset.contains("button_variant = \"text\"")
                && asset.contains("background_color = \"transparent\"")
                && asset.contains("border_color = \"transparent\""),
            "{asset_name} must not draw a card-like resting material for text icon controls"
        );
        for token in [
            "$editor.surface.hover",
            "$editor.surface.3",
            "$editor.surface.selected",
            "$editor.accent",
        ] {
            assert!(
                asset.contains(token),
                "{asset_name} must preserve the interactive feedback token `{token}`"
            );
        }
    }
}
