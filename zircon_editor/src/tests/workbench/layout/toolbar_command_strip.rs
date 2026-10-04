//! 工具栏辅助命令保持安静，Compile保留主要动作显著性；检查限定节点源码。
// 此处嵌入真实工具栏资产，节点范围辅助只服务本文件的声明契约。
const WORKBENCH_TOP_TOOLBAR_ASSET: &str = include_str!(
    "../../../../assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui"
);

/// 为工具栏断言定位一个节点表及其后续范围，避免其它按钮的样式声明满足当前按钮检查。
fn node_source(node_id: &str) -> &str {
    let marker = format!("[nodes.{node_id}]");
    let source = WORKBENCH_TOP_TOOLBAR_ASSET
        .split_once(&marker)
        .unwrap_or_else(|| panic!("toolbar asset must declare node `{node_id}`"))
        .1;
    source.split("\n[nodes.").next().unwrap_or(source)
}

#[test]
/// 按具体命令节点核对material意图，视觉优先级不改变命令执行许可。
fn module_toolbar_keeps_auxiliary_commands_quiet_and_compile_prominent() {
    for node_id in [
        "module_save",
        "module_browse",
        "module_diff",
        "module_simulate",
    ] {
        let source = node_source(node_id);
        assert!(
            source.contains("button_variant = \"tertiary\""),
            "{node_id} must use the quiet toolbar button variant"
        );
        assert!(
            source.contains("background_color = \"transparent\""),
            "{node_id} must keep its resting surface transparent"
        );
        assert!(
            source.contains("border_color = \"transparent\""),
            "{node_id} must reserve its border for interaction feedback"
        );
    }

    let compile = node_source("module_compile");
    assert!(compile.contains("button_variant = \"filled\""));
    assert!(compile.contains("button_color = \"accent\""));
    assert!(
        !compile.contains("background_color = \"transparent\""),
        "Compile must retain the prominent command treatment"
    );
}
