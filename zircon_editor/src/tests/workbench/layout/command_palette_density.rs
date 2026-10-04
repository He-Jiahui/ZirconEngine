//! 命令面板弹层尺寸来自共享density级联，主题必须登记相同token。
const EDITOR_TOKENS_ASSET: &str =
    include_str!("../../../../assets/ui/editor/theme/editor_tokens.zui");
const COMMAND_PALETTE_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/primitives/feedback/workbench_command_palette.zui"
);

#[test]
/// 同时核对消费资产和token定义，防止局部像素约束绕过全局密度设置。
fn command_palette_uses_shared_density_constraints_for_its_popup_surface() {
    for token in [
        "$editor.density.command_palette.min_width",
        "$editor.density.command_palette.preferred_width",
        "$editor.density.command_palette.max_width",
        "$editor.density.command_palette.min_height",
        "$editor.density.command_palette.preferred_height",
        "$editor.density.command_palette.max_height",
    ] {
        assert!(
            COMMAND_PALETTE_ASSET.contains(token),
            "command palette must resolve `{token}` through the density cascade"
        );
        assert!(
            EDITOR_TOKENS_ASSET.contains(&token[1..]),
            "editor tokens must name `{token}` for V2 resolution"
        );
    }

    for local_constraint in [
        "min = 520.0, preferred = 560.0, max = 640.0",
        "min = 180.0, preferred = 220.0, max = 280.0",
        "popup_anchor_width = 560.0",
    ] {
        assert!(
            !COMMAND_PALETTE_ASSET.contains(local_constraint),
            "command palette must not retain the local constraint `{local_constraint}`"
        );
    }
}
