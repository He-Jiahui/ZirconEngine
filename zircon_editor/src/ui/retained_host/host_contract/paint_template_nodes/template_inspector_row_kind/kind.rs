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
