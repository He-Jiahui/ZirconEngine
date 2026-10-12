#[derive(Clone, Copy)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum HostPaintCommandKind {
    Group,
    Quad,
    Text,
    Image,
}
