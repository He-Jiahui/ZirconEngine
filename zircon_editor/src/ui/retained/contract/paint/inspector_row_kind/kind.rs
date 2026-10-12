//! 分类结果是绘制子类而非领域属性类型，供 Inspector 命令入口选择资源、分组或阴影组合。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum InspectorRowKind {
    Resource(InspectorResourceKind),
    Disclosure,
    ShadowSelect,
    ShadowCheck,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum InspectorResourceKind {
    Mesh,
    Material,
}
