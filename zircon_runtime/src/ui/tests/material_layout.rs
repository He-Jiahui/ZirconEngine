//! 这里搭建真实表面测量夹具；子测试从作者布局指标到期望尺寸和绘制命令验证 Material 约束。
//! 共享夹具在本地表面执行完整布局，返回自有期望尺寸或渲染命令；表面由各次调用单独创建和释放。
//! 子模块覆盖组件行为；父模块还保留源码文本守卫，不能据此推断实际属性扫描次数。

use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{AxisConstraint, BoxConstraints, DesiredSize, LayoutBoundary, StretchMode, UiSize},
    surface::{UiRenderCommandKind, UiVisualAssetRef},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
};

#[path = "material_layout/asset_icon_roles.rs"]
mod asset_icon_roles;
#[path = "material_layout/button_icon_metrics.rs"]
mod button_icon_metrics;
#[path = "material_layout/constraints_children.rs"]
mod constraints_children;
#[path = "material_layout/field_values.rs"]
mod field_values;
#[path = "material_layout/row_label_metrics.rs"]
mod row_label_metrics;

// TODO: [CR-UI-TEST-0204] 确认是否需观测实际扫描次数；当前源码字符串守卫无法证明未换名重引入预扫描。
#[test]
fn material_layout_resolves_metrics_without_preflight_attribute_rescan() {
    let source = include_str!("../layout/pass/material.rs");

    assert!(!source.contains("has_layout_attribute"));
    // BUG: [CR-ZCA-R051-0001] 材料测量已改用带 Dropdown 回退的链式解析，此处旧文本锚点不存在，
    // 因而该包含断言会失败；这里只核对了静态源码，未执行 Rust 测试。
    assert!(source.contains("let Some(metrics) = MaterialLayoutMetrics::resolve(metadata) else"));
}

/// 在临时表面的内容驱动根下插入单个组件，执行完整布局后读取其期望尺寸。
///
/// # 参数
/// `component` 与 TOML `attributes` 构造节点自有元数据；`constraints` 直接赋给被测节点。
/// 根布局区域固定为 240×120；属性文本须能解析为 TOML 表。
///
/// # 返回与生命周期
/// 返回复制出的 `DesiredSize`；本地表面在返回后释放，不保留树节点或元数据借用。
/// 属性解析、构树或布局失败会在夹具中触发 panic。
fn measure_material_leaf(
    component: &str,
    attributes: &str,
    constraints: BoxConstraints,
) -> DesiredSize {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.material_layout"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_layout_boundary(LayoutBoundary::ContentDriven),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/control"))
                .with_layout_boundary(LayoutBoundary::ContentDriven)
                .with_constraints(constraints)
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    ..Default::default()
                }),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(240.0, 120.0)).unwrap();
    surface
        .tree
        .node(UiNodeId::new(2))
        .unwrap()
        .layout_cache
        .desired_size
}

/// 在新的临时表面上布局单个组件，并取出该节点的首个渲染命令。
///
/// # 参数
/// `component` 与 TOML `attributes` 构造节点自有元数据，节点使用本模块的固有尺寸约束。
///
/// # 返回与生命周期
/// 返回从列表移出的自有 `UiRenderCommand`，可在临时表面销毁后继续读取。
/// 属性解析、构树、布局失败或缺少目标命令会触发 panic。
fn render_material_leaf_command(
    component: &str,
    attributes: &str,
) -> zircon_runtime_interface::ui::surface::UiRenderCommand {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.material_layout.render"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_layout_boundary(LayoutBoundary::ContentDriven),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/control"))
                .with_layout_boundary(LayoutBoundary::ContentDriven)
                .with_constraints(intrinsic_constraints())
                .with_template_metadata(UiTemplateNodeMetadata {
                    component: component.to_string(),
                    attributes: toml::from_str(attributes).unwrap(),
                    ..Default::default()
                }),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(240.0, 120.0)).unwrap();
    surface
        .render_extract
        .list
        .commands
        .into_iter()
        .find(|command| command.node_id == UiNodeId::new(2))
        .unwrap_or_else(|| panic!("render extract should include `{component}` command"))
}

/// 将最小、最大与首选尺寸绑定为同一输入，建立高优先级 Fixed 轴。
/// 不另行验证输入数值；测试通过它隔离作者固定约束与内容尺寸的优先关系。
fn fixed_constraint(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

/// 为宽高分别使用下方的无上界轴配置，供内容驱动夹具复用。
fn intrinsic_constraints() -> BoxConstraints {
    BoxConstraints {
        width: intrinsic_axis_constraint(),
        height: intrinsic_axis_constraint(),
    }
}

/// 此处仍使用 Fixed 模式；负 max 在轴解析时成为无上界，供内容驱动夹具使用。
fn intrinsic_axis_constraint() -> AxisConstraint {
    AxisConstraint {
        min: 0.0,
        max: -1.0,
        preferred: 0.0,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}
