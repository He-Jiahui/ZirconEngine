use serde::{Deserialize, Serialize};

use crate::ui::dispatch::{UiPointerId, UiSurfaceId, UiUserId, UiWindowId};
use crate::ui::event_ui::{UiNodeId, UiTreeId};
use crate::ui::layout::{UiFrame, UiPoint};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiVirtualPointerPosition {
    pub current: UiPoint,
    pub previous: UiPoint,
}

impl UiVirtualPointerPosition {
    pub const fn new(current: UiPoint, previous: UiPoint) -> Self {
        Self { current, previous }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiHitCoordinateSpace {
    #[default]
    Surface,
    Window,
    Screen,
    World,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiHitTestScope {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UiUserId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_id: Option<UiWindowId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surface_id: Option<UiSurfaceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pointer_id: Option<UiPointerId>,
}

impl UiHitTestScope {
    pub const fn empty() -> Self {
        Self {
            user_id: None,
            window_id: None,
            surface_id: None,
            pointer_id: None,
        }
    }

    pub fn with_user_id(mut self, user_id: UiUserId) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_window_id(mut self, window_id: UiWindowId) -> Self {
        self.window_id = Some(window_id);
        self
    }

    pub fn with_surface_id(mut self, surface_id: UiSurfaceId) -> Self {
        self.surface_id = Some(surface_id);
        self
    }

    pub fn with_pointer_id(mut self, pointer_id: UiPointerId) -> Self {
        self.pointer_id = Some(pointer_id);
        self
    }

    pub fn conflicts_with(&self, query: &Self) -> bool {
        scoped_value_conflicts(&self.user_id, &query.user_id)
            || scoped_value_conflicts(&self.window_id, &query.window_id)
            || scoped_value_conflicts(&self.surface_id, &query.surface_id)
            || scoped_value_conflicts(&self.pointer_id, &query.pointer_id)
    }

    pub fn accepts_query(&self, query: &Self) -> bool {
        !self.conflicts_with(query)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiWorldHitRay {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}

impl UiWorldHitRay {
    pub const fn new(origin: [f32; 3], direction: [f32; 3]) -> Self {
        Self { origin, direction }
    }

    pub fn is_finite(self) -> bool {
        self.origin.iter().all(|value| value.is_finite())
            && self.direction.iter().all(|value| value.is_finite())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiHitTestQuery {
    pub point: UiPoint,
    pub cursor_radius: f32,
    pub virtual_pointer: Option<UiVirtualPointerPosition>,
    pub scope: UiHitTestScope,
    pub coordinate_space: UiHitCoordinateSpace,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_ray: Option<UiWorldHitRay>,
}

impl Default for UiHitTestQuery {
    fn default() -> Self {
        Self {
            point: UiPoint::default(),
            cursor_radius: 0.0,
            virtual_pointer: None,
            scope: UiHitTestScope::empty(),
            coordinate_space: UiHitCoordinateSpace::Surface,
            world_ray: None,
        }
    }
}

impl UiHitTestQuery {
    pub const fn new(point: UiPoint) -> Self {
        Self {
            point,
            cursor_radius: 0.0,
            virtual_pointer: None,
            scope: UiHitTestScope::empty(),
            coordinate_space: UiHitCoordinateSpace::Surface,
            world_ray: None,
        }
    }

    pub const fn with_cursor_radius(mut self, cursor_radius: f32) -> Self {
        self.cursor_radius = cursor_radius;
        self
    }

    pub const fn with_virtual_pointer(mut self, virtual_pointer: UiVirtualPointerPosition) -> Self {
        self.virtual_pointer = Some(virtual_pointer);
        self
    }

    pub fn with_scope(mut self, scope: UiHitTestScope) -> Self {
        self.scope = scope;
        self
    }

    pub const fn with_coordinate_space(mut self, coordinate_space: UiHitCoordinateSpace) -> Self {
        self.coordinate_space = coordinate_space;
        self
    }

    pub const fn with_world_ray(mut self, world_ray: UiWorldHitRay) -> Self {
        self.world_ray = Some(world_ray);
        self.coordinate_space = UiHitCoordinateSpace::World;
        self
    }

    pub const fn with_projected_world_hit(
        mut self,
        world_ray: UiWorldHitRay,
        virtual_pointer: UiVirtualPointerPosition,
    ) -> Self {
        self.world_ray = Some(world_ray);
        self.virtual_pointer = Some(virtual_pointer);
        self.coordinate_space = UiHitCoordinateSpace::World;
        self
    }

    pub fn hit_point(&self) -> UiPoint {
        self.virtual_pointer
            .map(|virtual_pointer| virtual_pointer.current)
            .unwrap_or(self.point)
    }

    pub fn sanitized_cursor_radius(&self) -> f32 {
        if self.cursor_radius.is_finite() {
            self.cursor_radius.max(0.0)
        } else {
            0.0
        }
    }

    pub fn uses_surface_coordinates(&self) -> bool {
        match self.coordinate_space {
            UiHitCoordinateSpace::Surface => true,
            UiHitCoordinateSpace::World => self.has_projected_world_hit(),
            UiHitCoordinateSpace::Window | UiHitCoordinateSpace::Screen => false,
        }
    }

    pub fn has_projected_world_hit(&self) -> bool {
        self.coordinate_space == UiHitCoordinateSpace::World
            && self.virtual_pointer.is_some()
            && self
                .world_ray
                .map(|world_ray| world_ray.is_finite())
                .unwrap_or(false)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiHitPath {
    pub target: Option<UiNodeId>,
    pub root_to_leaf: Vec<UiNodeId>,
    pub bubble_route: Vec<UiNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub virtual_pointer: Option<UiVirtualPointerPosition>,
}

impl UiHitPath {
    pub fn from_query(query: &UiHitTestQuery) -> Self {
        Self {
            target: None,
            root_to_leaf: Vec::new(),
            bubble_route: Vec::new(),
            virtual_pointer: query.virtual_pointer,
        }
    }

    pub fn with_route(
        mut self,
        target: Option<UiNodeId>,
        root_to_leaf: Vec<UiNodeId>,
        bubble_route: Vec<UiNodeId>,
    ) -> Self {
        self.target = target;
        self.root_to_leaf = root_to_leaf;
        self.bubble_route = bubble_route;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiHitTestEntry {
    pub node_id: UiNodeId,
    pub frame: UiFrame,
    pub clip_frame: UiFrame,
    pub z_index: i32,
    pub paint_order: u64,
    pub control_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiHitTestCell {
    pub entries: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiHitTestGrid {
    pub bounds: UiFrame,
    pub cell_size: f32,
    pub columns: u32,
    pub rows: u32,
    pub scope: UiHitTestScope,
    pub entries: Vec<UiHitTestEntry>,
    pub cells: Vec<UiHitTestCell>,
}

impl Default for UiHitTestGrid {
    fn default() -> Self {
        Self {
            bounds: UiFrame::default(),
            cell_size: 64.0,
            columns: 0,
            rows: 0,
            scope: UiHitTestScope::empty(),
            entries: Vec::new(),
            cells: Vec::new(),
        }
    }
}

impl UiHitTestGrid {
    pub fn with_scope(mut self, scope: UiHitTestScope) -> Self {
        self.scope = scope;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiHitTestDebugDump {
    pub tree_id: UiTreeId,
    pub point: crate::ui::layout::UiPoint,
    pub hit_stack: Vec<UiNodeId>,
    pub hit_path: UiHitPath,
    pub inspected: usize,
    pub rejected: Vec<UiHitTestReject>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiHitTestReject {
    pub node_id: UiNodeId,
    pub control_id: Option<String>,
    pub reason: UiHitTestRejectReason,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiHitTestRejectReason {
    OutsideFrame,
    OutsideClip,
    VisibilityFiltered,
    Disabled,
    InputPolicyIgnore,
    NotPointerTarget,
    MissingAncestry,
    StaleGridEntry,
    CustomHitPathUnavailable,
    ScopeMismatch,
    UnsupportedCoordinateSpace,
    WorldHitUnavailable,
}

fn scoped_value_conflicts<T: PartialEq>(grid_value: &Option<T>, query_value: &Option<T>) -> bool {
    matches!((grid_value, query_value), (Some(grid_value), Some(query_value)) if grid_value != query_value)
}
