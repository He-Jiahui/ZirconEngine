//! 读取生产 ZUI 资产并验证工作台状态信号的共享度量和语义配色。

use toml::Value;

use super::support::{editor_asset_root, load_zui_document};

const STATUS_BAR_ASSET: &str = "ui/editor/unreal/components/status_bar.zui";
const SEMANTIC_STATUS_SIGNAL_VARIANT: &str = "semantic_status_signal";

#[test]
fn workbench_status_signals_inherit_shared_metrics_and_semantic_palette() {
    let path = editor_asset_root().join(STATUS_BAR_ASSET);
    let document = load_zui_document(&path);

    for (node_id, expected_tone) in [
        ("ready", "primary"),
        ("errors", "muted"),
        ("warnings", "muted"),
        ("messages", "muted"),
    ] {
        let node = document
            .nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("{} should declare status node `{node_id}`", path.display()));

        assert_eq!(node.component, "Label");
        assert!(node
            .classes
            .iter()
            .any(|class| class == "unreal-status-label"));
        assert_eq!(
            node.props.get("foreground_color").and_then(Value::as_str),
            Some("$unreal_foreground")
        );
        assert_eq!(
            node.props.get("component_variant").and_then(Value::as_str),
            Some(SEMANTIC_STATUS_SIGNAL_VARIANT),
            "{node_id} should opt into the shared status-signal palette"
        );
        assert_eq!(
            node.props.get("text_tone").and_then(Value::as_str),
            Some(expected_tone),
            "{node_id} should declare only its semantic text role"
        );
        for local_override in [
            "icon_fill",
            "icon_size",
            "layout_gap",
            "layout_offset_x",
            "layout_offset_y",
            "text_color",
        ] {
            assert!(
                !node.props.contains_key(local_override),
                "{node_id} should inherit `{local_override}` from the shared status-control owner"
            );
        }
    }
}
