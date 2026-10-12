//! 原子控件和样本继承共享视觉及密度；源码契约与TOML结构检查承担不同断言。
use super::support::assert_tokenized_assets;
use toml::Value;

/// 为本文件测试嵌入真实资产字节，路径相对此测试源定位；不加载运行时资源目录。
macro_rules! workbench_asset {
    ($path:literal) => {
        include_str!(concat!(
            "../../../../../assets/ui/editor/components/workbench/",
            $path
        ))
    };
}
macro_rules! unreal_asset {
    ($path:literal) => {
        include_str!(concat!("../../../../../assets/ui/editor/unreal/", $path))
    };
}

#[test]
/// 列明每类原子的主题输入并拒绝裸颜色，防止样式复制脱离统一token来源。
fn atomic_controls_share_editor_visual_and_density_tokens() {
    assert_tokenized_assets(&[
        (
            "workbench_button.zui",
            workbench_asset!("primitives/inputs/workbench_button.zui"),
            &[
                "$editor.surface.1",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.density.gap.large",
                "$editor.density.gap.medium",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_icon_button.zui",
            workbench_asset!("primitives/inputs/workbench_icon_button.zui"),
            &[
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.control.height.default",
            ],
        ),
        (
            "workbench_rail_button.zui",
            workbench_asset!("primitives/chrome/workbench_rail_button.zui"),
            &[
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.density.gap.medium",
                "$editor.control.height.large",
            ],
        ),
        (
            "workbench_axis_value_field.zui",
            workbench_asset!("primitives/chrome/workbench_axis_value_field.zui"),
            &[
                "$editor.surface.recessed",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.disabled",
                "$editor.separator.soft",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.density.gap.small",
                "$editor.density.gap.medium",
                "$editor.density.axis_value_field.min_width",
                "$editor.density.axis_value_field.preferred_width",
                "$editor.density.axis_value_field.max_width",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_tab.zui",
            workbench_asset!("primitives/inputs/workbench_tab.zui"),
            &[
                "$editor.surface.2",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.small",
                "$editor.density.gap.small",
                "$editor.density.gap.medium",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_divider.zui",
            workbench_asset!("primitives/data/workbench_divider.zui"),
            &[
                "$editor.separator.soft",
                "$editor.border.disabled",
                "$editor.control.border_width",
                "$editor.density.gap.medium",
            ],
        ),
        (
            "workbench_search_input.zui",
            workbench_asset!("primitives/inputs/workbench_search_input.zui"),
            &[
                "$editor.surface.recessed",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.disabled",
                "$editor.separator.soft",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.density.gap.small",
                "$editor.density.gap.large",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_dropdown.zui",
            workbench_asset!("primitives/inputs/workbench_dropdown.zui"),
            &[
                "$editor.surface.recessed",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.accent.soft",
                "$editor.surface.disabled",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.density.gap.large",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.caption.size",
                "$editor.typography.overlay.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_field.zui",
            workbench_asset!("primitives/inputs/workbench_field.zui"),
            &[
                "$editor.surface.recessed",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.disabled",
                "$editor.separator.soft",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_number_field.zui",
            workbench_asset!("primitives/inputs/workbench_number_field.zui"),
            &[
                "$editor.surface.recessed",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.disabled",
                "$editor.separator.soft",
                "$editor.border",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.density.gap.medium",
                "$editor.density.gap.small",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_toggle.zui",
            workbench_asset!("primitives/inputs/workbench_toggle.zui"),
            &[
                "$editor.surface.2",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.separator.strong",
                "$editor.border.disabled",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.density.gap.medium",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
        (
            "workbench_segmented_control.zui",
            workbench_asset!("primitives/inputs/workbench_segmented_control.zui"),
            &[
                "$editor.surface.2",
                "$editor.surface.hover",
                "$editor.surface.3",
                "$editor.surface.selected",
                "$editor.surface.disabled",
                "$editor.border",
                "$editor.accent",
                "$editor.text.primary",
                "$editor.text.secondary",
                "$editor.text.disabled",
                "$editor.control.border_width",
                "$editor.control.radius.control",
                "$editor.control.height.dense",
                "$editor.control.height.compact",
                "$editor.control.height.default",
                "$editor.typography.body.size",
                "$editor.typography.line_height",
            ],
        ),
    ]);
}

#[test]
/// 标签水平空间由父容器分配，控件本体不保留旧的局部固定宽度。
fn workbench_tab_uses_tokenized_states_and_stretch_width() {
    let asset = workbench_asset!("primitives/inputs/workbench_tab.zui");

    assert!(
        asset.contains("width = { stretch = \"Stretch\" }"),
        "the reusable tab must let its parent allocate horizontal width"
    );
    assert!(
        [
            "layout_padding_right = 10.0",
            "layout_padding_top = 3.0",
            "layout_min_width = 72.0",
            "layout_icon_size = 14.0",
            "width = { min = 88.0, preferred = 96.0, max = 140.0, stretch = \"Fixed\" }",
        ]
        .iter()
        .all(|legacy_metric| !asset.contains(legacy_metric)),
        "the reusable tab must not retain local spacing, icon, or width metrics"
    );
}

#[test]
/// 活动栏按钮沿两轴共用compact控制尺寸，避免视觉尺寸和命中面积分开漂移。
fn workbench_rail_button_uses_shared_compact_control_geometry() {
    let asset = workbench_asset!("primitives/chrome/workbench_rail_button.zui");

    assert!(
        asset.matches("$editor.control.height.compact").count() >= 4,
        "the activity-rail button must use the shared compact control size for both hit axes"
    );
    assert!(
        [
            "layout_spacing = 0.0",
            "layout_min_width = 44.0",
            "layout_min_height = 44.0",
            "layout_icon_size = 22.0",
            "width = { min = 46.0, preferred = 48.0, max = 50.0, stretch = \"Fixed\" }",
            "height = { min = 46.0, preferred = 48.0, max = 50.0, stretch = \"Fixed\" }",
        ]
        .iter()
        .all(|legacy_metric| !asset.contains(legacy_metric)),
        "the activity-rail button must not retain private fixed geometry"
    );
}

#[test]
fn unreal_toolbar_controls_keep_native_identity_and_action_bindings() {
    let asset = unreal_asset!("components/toolbar.zui");
    for required in [
        "control_id = \"WorkbenchToolbarSave\"",
        "control_id = \"WorkbenchRunPlay\"",
        "control_id = \"WorkbenchRunStop\"",
        "file.project.save",
        "runtime.play_mode.enter",
        "runtime.play_mode.exit",
    ] {
        assert!(
            asset.contains(required),
            "Unreal toolbar must retain `{required}`"
        );
    }
}

#[test]
fn unreal_toolbar_controls_are_focusable_and_do_not_use_zero_geometry() {
    let asset = unreal_asset!("components/toolbar.zui");
    assert!(asset.matches("input_focusable = true").count() >= 6);
    assert!(!asset.contains("min = 0.0, preferred = 0.0, max = 0.0"));
    assert!(!asset.contains("component_drawer"));
}
