//! 壳层复合资产继承共享表面、密度与语义槽；响应式隐藏属于资产声明。
use super::chrome_theme::strict_theme_rule;
use super::support::assert_tokenized_assets;
use toml::Value;

/// 嵌入本测试实际检查的壳资产，源码断言不代表已完成窗口交互验证。
macro_rules! workbench_asset {
    ($path:literal) => {
        include_str!(concat!("../../../../assets/ui/editor/workbench/", $path))
    };
}

macro_rules! unreal_asset {
    ($path:literal) => {
        include_str!(concat!(
            "../../../../assets/ui/editor/unreal/components/",
            $path
        ))
    };
}

#[test]
fn unreal_shell_chrome_uses_real_toolbar_and_status_assets() {
    let toolbar = unreal_asset!("toolbar.zui");
    let status = unreal_asset!("status_bar.zui");
    assert!(toolbar.contains("control_id = \"WorkbenchWindowTopToolbar\""));
    assert!(toolbar.contains("separator_edge = \"bottom\""));
    assert!(toolbar.contains("height = { min = 48.0, preferred = 48.0, max = 48.0"));
    assert!(status.contains("control_id = \"UnrealEditorStatusBar\""));
    assert!(status.contains("separator_edge = \"top\""));
    assert!(status.contains("control_id = \"UnrealContentDrawerToggle\""));
}

#[test]
/// 保留可拉伸语义槽及窄屏最小tier，防止宿主组合退回固定绝对尺寸。
fn activity_drawer_window_uses_relative_shell_constraints() {
    let asset = workbench_asset!("shell/activity_drawer_window.zui");
    assert_tokenized_assets(&[(
        "activity_drawer_window.zui",
        asset,
        &[
            "$editor.density.compact_side_min_width",
            "$editor.density.left_drawer_width",
            "$editor.density.right_drawer_width",
            "$editor.density.ultra_compact_bottom_min_height",
            "$editor.density.bottom_output_height",
        ],
    )]);

    for stretch_slot in [
        "ActivityDrawerWindowContentSlot",
        "ActivityDrawerWindowLeftTopSlot",
        "ActivityDrawerWindowLeftBottomSlot",
        "ActivityDrawerWindowRightTopSlot",
        "ActivityDrawerWindowRightBottomSlot",
        "ActivityDrawerWindowBottomLeftSlot",
        "ActivityDrawerWindowBottomRightSlot",
    ] {
        assert!(
            asset.contains(stretch_slot),
            "activity drawer window must retain the {stretch_slot} relative slot"
        );
    }
    assert!(
        asset.matches("stretch = \"Stretch\"").count() >= 11,
        "activity drawer window must keep its root, bands, and slots stretch-based"
    );
    let document = asset
        .parse::<Value>()
        .expect("activity drawer window must remain valid ZUI TOML");
    let nodes = document
        .get("nodes")
        .and_then(Value::as_table)
        .expect("activity drawer window must expose nodes");
    for (node_id, minimum_tier) in [
        ("left_column", "narrow"),
        ("right_column", "regular"),
        ("bottom_right_slot", "regular"),
    ] {
        let node = nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("activity drawer window must retain `{node_id}`"));
        let responsive_tier = node
            .get("props")
            .and_then(Value::as_table)
            .and_then(|props| props.get("responsive_min_tier"))
            .and_then(Value::as_str);
        assert_eq!(
            responsive_tier,
            Some(minimum_tier),
            "activity drawer node `{node_id}` must collapse below `{minimum_tier}`"
        );
    }
    assert!(
        [
            "min = 240.0",
            "preferred = 280.0",
            "max = 420.0",
            "preferred = 300.0",
            "max = 460.0",
            "min = 170.0",
            "preferred = 220.0",
            "max = 320.0",
        ]
        .iter()
        .all(|legacy_extent| !asset.contains(legacy_extent)),
        "activity drawer window must not retain local drawer or bottom-band geometry"
    );
}

#[test]
fn unreal_viewport_uses_native_viewport_identity_and_shared_toolbar_height() {
    let asset = unreal_asset!("viewport.zui");
    assert!(asset.contains("control_id = \"UnrealLevelViewport\""));
    assert!(asset.contains("control_id = \"WorkbenchViewportSurface\""));
    assert!(asset.contains("$editor.chrome.viewport_toolbar.height"));
    assert!(!asset.contains("activity_rail"));
}

#[test]
/// 标题与动作作为可分配空间的槽共存，允许长标题与宿主操作共同布局。
fn panel_header_uses_shared_container_surface_and_density() {
    let asset = workbench_asset!("composites/chrome/panel_header.zui");
    assert_tokenized_assets(&[(
        "workbench_panel_header.zui",
        asset,
        &[
            "$editor.surface.1",
            "$editor.separator.soft",
            "$editor.control.border_width",
            "$editor.density.gap.small",
            "$editor.control.height.dense",
            "$editor.control.height.compact",
        ],
    )]);
    assert!(
        asset.contains("name = \"title\"")
            && asset.contains("name = \"actions\"")
            && asset.matches("stretch = \"Stretch\"").count() >= 4,
        "panel header must preserve its stretch title/actions slot contract"
    );
    assert!(
        !asset.contains("gap = 2.0")
            && !asset.contains("min = 28.0")
            && !asset.contains("max = 30.0"),
        "panel header must not retain local density metrics"
    );
}

#[test]
fn unreal_content_drawer_has_semantic_focus_and_dismiss_contract() {
    let asset = unreal_asset!("content_drawer.zui");
    assert!(asset.contains("control_id = \"UnrealContentDrawer\""));
    assert!(asset.contains("dismiss_on_focus_loss = true"));
    assert!(asset.contains("open_focus_control_id = \"SearchEdited\""));
    assert!(asset.contains("workbench.asset.set_search_query"));
    assert!(asset.contains("workbench.content_drawer.dismiss"));
}
