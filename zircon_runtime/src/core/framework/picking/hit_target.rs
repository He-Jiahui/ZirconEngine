use crate::core::framework::scene::EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PickingAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// 运行时与编辑器共同使用的可选取身份；轴柄、场景 gizmo 和普通可渲染体按交互优先级排序。
pub enum HitTarget {
    HandleAxis { owner: EntityId, axis: PickingAxis },
    SceneGizmo { owner: EntityId },
    Renderable { owner: EntityId },
}

impl HitTarget {
    pub const fn handle_axis(owner: EntityId, axis: PickingAxis) -> Self {
        Self::HandleAxis { owner, axis }
    }

    pub const fn scene_gizmo(owner: EntityId) -> Self {
        Self::SceneGizmo { owner }
    }

    pub const fn renderable(owner: EntityId) -> Self {
        Self::Renderable { owner }
    }

    pub const fn owner(self) -> EntityId {
        match self {
            Self::HandleAxis { owner, .. }
            | Self::SceneGizmo { owner }
            | Self::Renderable { owner } => owner,
        }
    }

    pub const fn priority(self) -> PickingTargetPriority {
        match self {
            Self::HandleAxis { .. } => PickingTargetPriority::HandleAxis,
            Self::SceneGizmo { .. } => PickingTargetPriority::SceneGizmo,
            Self::Renderable { .. } => PickingTargetPriority::Renderable,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PickingTargetPriority {
    HandleAxis,
    SceneGizmo,
    Renderable,
}

impl PickingTargetPriority {
    pub const fn sort_key(self) -> u8 {
        match self {
            Self::HandleAxis => 0,
            Self::SceneGizmo => 1,
            Self::Renderable => 2,
        }
    }
}
