use crate::core::math::{Real, Transform, Vec3, Vec4};

use crate::core::framework::scene::EntityId;

use super::DisplayMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneGizmoKind {
    Camera,
    DirectionalLight,
    VirtualGeometryBvh,
    VirtualGeometryVisBuffer,
    NavigationMesh,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ViewportIconId {
    Camera,
    DirectionalLight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OverlayLineSegment {
    pub start: Vec3,
    pub end: Vec3,
    pub color: Vec4,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OverlayWireShape {
    Frustum {
        transform: Transform,
        fov_y_radians: Real,
        aspect_ratio: Real,
        z_near: Real,
        z_far: Real,
        color: Vec4,
    },
    Arrow {
        origin: Vec3,
        direction: Vec3,
        length: Real,
        color: Vec4,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct OverlayBillboardIcon {
    pub id: ViewportIconId,
    pub position: Vec3,
    pub tint: Vec4,
    pub size: Real,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OverlayPickShape {
    Sphere {
        center: Vec3,
        radius: Real,
    },
    Segment {
        start: Vec3,
        end: Vec3,
        thickness: Real,
    },
    Circle {
        center: Vec3,
        normal: Vec3,
        radius: Real,
        thickness: Real,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneGizmoOverlayExtract {
    pub owner: EntityId,
    pub kind: SceneGizmoKind,
    pub selected: bool,
    pub lines: Vec<OverlayLineSegment>,
    pub wire_shapes: Vec<OverlayWireShape>,
    pub icons: Vec<OverlayBillboardIcon>,
    pub pick_shapes: Vec<OverlayPickShape>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectionHighlightExtract {
    pub owner: EntityId,
    pub outline: bool,
    pub tint: Option<Vec4>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectionAnchorExtract {
    pub owner: EntityId,
    pub position: Vec3,
    pub size: Real,
    pub color: Vec4,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridOverlayExtract {
    pub visible: bool,
    pub snap_enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HandleElementExtract {
    AxisLine {
        axis: OverlayAxis,
        start: Vec3,
        end: Vec3,
        color: Vec4,
        pick_radius: Real,
    },
    AxisRing {
        axis: OverlayAxis,
        center: Vec3,
        normal: Vec3,
        radius: Real,
        color: Vec4,
        pick_radius: Real,
    },
    AxisScale {
        axis: OverlayAxis,
        start: Vec3,
        end: Vec3,
        color: Vec4,
        pick_radius: Real,
        handle_size: Real,
    },
    CenterAnchor {
        position: Vec3,
        size: Real,
        color: Vec4,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct HandleOverlayExtract {
    pub owner: EntityId,
    pub origin: Transform,
    pub elements: Vec<HandleElementExtract>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderOverlayExtract {
    pub selection: Vec<SelectionHighlightExtract>,
    pub selection_anchors: Vec<SelectionAnchorExtract>,
    pub grid: Option<GridOverlayExtract>,
    pub handles: Vec<HandleOverlayExtract>,
    pub scene_gizmos: Vec<SceneGizmoOverlayExtract>,
    pub display_mode: DisplayMode,
}

impl Default for RenderOverlayExtract {
    fn default() -> Self {
        Self {
            selection: Vec::new(),
            selection_anchors: Vec::new(),
            grid: None,
            handles: Vec::new(),
            scene_gizmos: Vec::new(),
            display_mode: DisplayMode::Shaded,
        }
    }
}
