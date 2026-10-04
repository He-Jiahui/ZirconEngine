//! 普通及确认对话框共用尺寸token，并保留各自必要覆盖。
const EDITOR_TOKENS_ASSET: &str =
    include_str!("../../../../assets/ui/editor/theme/editor_tokens.zui");
const DIALOG_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/primitives/feedback/workbench_dialog.zui"
);
const CONFIRM_DIALOG_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/primitives/feedback/workbench_confirm_dialog.zui"
);

#[test]
/// 确保变体从共享主题取得尺寸而非复制像素边界；这是静态资产约束，不验证模态焦点。
fn modal_dialog_variants_use_shared_density_constraints() {
    for (asset, tokens, local_constraints) in [
        (
            DIALOG_ASSET,
            &[
                "$editor.density.dialog.min_width",
                "$editor.density.dialog.preferred_width",
                "$editor.density.dialog.max_width",
                "$editor.density.dialog.min_height",
                "$editor.density.dialog.preferred_height",
                "$editor.density.dialog.max_height",
            ][..],
            &[
                "min = 420.0, preferred = 480.0, max = 560.0",
                "min = 180.0, preferred = 220.0, max = 320.0",
            ][..],
        ),
        (
            CONFIRM_DIALOG_ASSET,
            &[
                "$editor.density.dialog.min_width",
                "$editor.density.confirm_dialog.preferred_width",
                "$editor.density.confirm_dialog.max_width",
                "$editor.density.confirm_dialog.min_height",
                "$editor.density.confirm_dialog.preferred_height",
                "$editor.density.confirm_dialog.max_height",
            ][..],
            &[
                "min = 420.0, preferred = 460.0, max = 540.0",
                "min = 174.0, preferred = 210.0, max = 300.0",
            ][..],
        ),
    ] {
        for token in tokens {
            assert!(
                asset.contains(token),
                "dialog variant must resolve `{token}` through the density cascade"
            );
            assert!(
                EDITOR_TOKENS_ASSET.contains(&token[1..]),
                "editor tokens must name `{token}` for V2 resolution"
            );
        }

        for local_constraint in local_constraints {
            assert!(
                !asset.contains(local_constraint),
                "dialog variant must not retain the local constraint `{local_constraint}`"
            );
        }
    }
}
