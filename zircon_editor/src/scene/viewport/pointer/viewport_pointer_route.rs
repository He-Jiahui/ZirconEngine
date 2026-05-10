use crate::scene::viewport::GizmoAxis;
use zircon_runtime::core::framework::picking::{HitTarget, PickingAxis};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ViewportPointerRoute {
    HandleAxis { owner: u64, axis: GizmoAxis },
    SceneGizmo { owner: u64 },
    Renderable { owner: u64 },
}

impl ViewportPointerRoute {
    pub(crate) fn target(&self) -> HitTarget {
        match self {
            Self::HandleAxis { owner, axis } => HitTarget::handle_axis(*owner, picking_axis(*axis)),
            Self::SceneGizmo { owner } => HitTarget::scene_gizmo(*owner),
            Self::Renderable { owner } => HitTarget::renderable(*owner),
        }
    }

    #[cfg(test)]
    pub(crate) fn from_target(target: HitTarget) -> Self {
        match target {
            HitTarget::HandleAxis { owner, axis } => Self::HandleAxis {
                owner,
                axis: gizmo_axis(axis),
            },
            HitTarget::SceneGizmo { owner } => Self::SceneGizmo { owner },
            HitTarget::Renderable { owner } => Self::Renderable { owner },
        }
    }
}

const fn picking_axis(axis: GizmoAxis) -> PickingAxis {
    match axis {
        GizmoAxis::X => PickingAxis::X,
        GizmoAxis::Y => PickingAxis::Y,
        GizmoAxis::Z => PickingAxis::Z,
    }
}

#[cfg(test)]
const fn gizmo_axis(axis: PickingAxis) -> GizmoAxis {
    match axis {
        PickingAxis::X => GizmoAxis::X,
        PickingAxis::Y => GizmoAxis::Y,
        PickingAxis::Z => GizmoAxis::Z,
    }
}
