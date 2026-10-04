//! 状态栏动态文案必须使用有限空间省略及共享文字token。
const WORKBENCH_STATUS_ITEM_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/primitives/feedback/workbench_status_item.zui"
);

#[test]
/// 防止任务或诊断长文案撑开壳层，静态契约仍依赖运行时文本测量实现。
fn status_item_ellipsizes_dynamic_text_with_the_runtime_text_tokens() {
    assert!(
        WORKBENCH_STATUS_ITEM_ASSET.contains("text_overflow = \"ellipsis\""),
        "a constrained status item must use the runtime ellipsis path instead of painting into adjacent controls"
    );
    for token in [
        "$editor.typography.body.size",
        "$editor.typography.line_height",
        "$editor.text.primary",
        "$editor.text.disabled",
    ] {
        assert!(
            WORKBENCH_STATUS_ITEM_ASSET.contains(token),
            "status item must preserve the shared runtime text token `{token}`"
        );
    }
}
