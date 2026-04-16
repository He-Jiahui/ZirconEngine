use zircon_graphics::GizmoAxis;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ViewportPointerRoute {
    HandleAxis { owner: u64, axis: GizmoAxis },
    SceneGizmo { owner: u64 },
    Renderable { owner: u64 },
}
