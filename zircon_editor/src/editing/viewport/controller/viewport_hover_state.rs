use zircon_graphics::GizmoAxis;

#[derive(Clone, Debug, Default)]
pub(crate) struct ViewportHoverState {
    pub(crate) hovered_axis: Option<GizmoAxis>,
    pub(crate) hovered_entity: Option<u64>,
}
