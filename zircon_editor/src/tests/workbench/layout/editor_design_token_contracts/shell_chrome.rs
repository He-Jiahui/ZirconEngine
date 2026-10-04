//! 壳层复合资产继承共享表面、密度与语义槽；响应式隐藏属于资产声明。
use super::chrome_theme::strict_theme_rule;
use super::support::assert_tokenized_assets;
use toml::Value;

/// 嵌入本测试实际检查的壳资产，源码断言不代表已完成窗口交互验证。
macro_rules! workbench_asset {
    ($path:literal) => {
        include_str!(concat!(
            "../../../../../assets/ui/editor/components/workbench/",
            $path
        ))
    };
}

#[test]
/// 工具栏与状态栏共同依赖壳层主题，不允许各自私有颜色和文字度量。
fn shell_chrome_roots_share_tokenized_surface_typography_and_density_inputs() {
    assert_tokenized_assets(&[
        (
            "workbench_top_toolbar.zui",
            workbench_asset!("shell/workbench_top_toolbar.zui"),
            &[
                "$editor.surface.2",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.separator.soft",
                "$editor.border",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_status_bar.zui",
            workbench_asset!("shell/workbench_status_bar.zui"),
            &[
                "$editor.surface.0",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.separator.soft",
                "$editor.border",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
    ]);
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
/// 活动栏自身宽度、间距和按钮高度应从统一密度来源获得。
fn activity_rail_uses_shared_component_spacing() {
    let asset = workbench_asset!("shell/workbench_activity_rail.zui");
    assert_tokenized_assets(&[(
        "workbench_activity_rail.zui",
        asset,
        &[
            "$editor.density.gap.medium",
            "$editor.chrome.activity_rail.width",
            "$editor.control.height.compact",
        ],
    )]);
    assert!(
        !asset.contains("gap = 10.0")
            && !asset.contains("min = 72.0")
            && !asset.contains("preferred = 72.0")
            && !asset.contains("max = 72.0")
            && !asset.contains("min = 48.0")
            && !asset.contains("preferred = 48.0")
            && !asset.contains("max = 48.0"),
        "activity rail must not keep local gap, shell-width, or item-size geometry"
    );
}

#[test]
/// 标题与动作作为可分配空间的槽共存，允许长标题与宿主操作共同布局。
fn panel_header_uses_shared_container_surface_and_density() {
    let asset = workbench_asset!("composites/chrome/workbench_panel_header.zui");
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
/// 样本容器继承共用类规则；局部圆角或边宽不能掩盖主题容器行为。
fn component_drawer_sample_cards_share_container_chrome() {
    let asset = workbench_asset!("shell/workbench_component_drawer.zui");
    let document = asset
        .parse::<Value>()
        .expect("component drawer must remain valid ZUI TOML");
    let nodes = document
        .get("nodes")
        .and_then(Value::as_table)
        .expect("component drawer must expose nodes");
    let card_nodes = [
        "component_buttons",
        "component_icon_buttons",
        "component_inputs",
        "component_sliders",
        "component_selection",
        "component_labs",
        "component_feedback",
        "component_list",
        "component_table",
    ];

    for node_id in card_nodes {
        let node = nodes
            .get(node_id)
            .unwrap_or_else(|| panic!("component drawer must retain `{node_id}`"));
        let classes = node
            .get("classes")
            .and_then(Value::as_array)
            .unwrap_or_else(|| panic!("component drawer card `{node_id}` must expose classes"));
        assert!(
            classes
                .iter()
                .filter_map(Value::as_str)
                .any(|class| class == "workbench-component-sample-card"),
            "component drawer card `{node_id}` must use the shared sample-card class"
        );
        let props = node
            .get("props")
            .and_then(Value::as_table)
            .unwrap_or_else(|| panic!("component drawer card `{node_id}` must expose props"));
        for property in ["corner_radius", "border_width"] {
            assert!(
                !props.contains_key(property),
                "component drawer card `{node_id}` must inherit `{property}` from its class"
            );
        }
    }

    let rule = strict_theme_rule(".workbench-component-sample-card");
    for declaration in [
        "background_color = \"$workbench_panel\"",
        "border_color = \"$workbench_border\"",
        "border_width = \"$editor.control.border_width\"",
        "radius = \"$editor.control.radius.control\"",
    ] {
        assert!(
            rule.contains(declaration),
            "shared component drawer card chrome must declare {declaration}"
        );
    }
}
