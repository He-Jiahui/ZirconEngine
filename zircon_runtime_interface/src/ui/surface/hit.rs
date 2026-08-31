use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::Error as _,
    ser::{SerializeSeq, SerializeStruct},
};
use std::{ops::Deref, slice, sync::Arc};

use crate::ui::dispatch::{UiPointerId, UiSurfaceId, UiUserId, UiWindowId};
use crate::ui::event_ui::{UiNodeId, UiTreeId};
use crate::ui::layout::{UiFrame, UiPoint};
use crate::ui::tree::UiInputPolicy;

use super::UiPersistentSequence;

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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiHitPath {
    pub target: Option<UiNodeId>,
    pub root_to_leaf: Vec<UiNodeId>,
    pub virtual_pointer: Option<UiVirtualPointerPosition>,
}

impl UiHitPath {
    pub fn from_query(query: &UiHitTestQuery) -> Self {
        Self {
            target: None,
            root_to_leaf: Vec::new(),
            virtual_pointer: query.virtual_pointer,
        }
    }

    /// Creates a hit path from the one authoritative propagation order.
    pub fn from_root_to_leaf(query: &UiHitTestQuery, root_to_leaf: Vec<UiNodeId>) -> Self {
        Self::from_query(query).with_root_to_leaf(root_to_leaf)
    }

    /// Creates a hit path from a retained bubble route without revalidating and rebuilding it.
    /// The route is already authored by the hit-test index; only the root-to-leaf view is copied.
    pub fn from_bubble_route(
        query: &UiHitTestQuery,
        target: Option<UiNodeId>,
        mut bubble_route: Vec<UiNodeId>,
    ) -> Self {
        bubble_route.reverse();
        Self {
            target,
            root_to_leaf: bubble_route,
            virtual_pointer: query.virtual_pointer,
        }
    }

    pub fn with_root_to_leaf(mut self, root_to_leaf: Vec<UiNodeId>) -> Self {
        self.target = root_to_leaf.last().copied();
        self.root_to_leaf = root_to_leaf;
        self
    }

    pub fn bubble_route(
        &self,
    ) -> impl DoubleEndedIterator<Item = UiNodeId> + ExactSizeIterator + Clone + '_ {
        self.root_to_leaf.iter().rev().copied()
    }

    pub fn into_bubble_route(mut self) -> Vec<UiNodeId> {
        self.root_to_leaf.reverse();
        self.root_to_leaf
    }

    /// Accepts caller-supplied route inputs only when they match the authoritative path.
    pub fn with_route(
        self,
        target: Option<UiNodeId>,
        root_to_leaf: Vec<UiNodeId>,
        bubble_route: Vec<UiNodeId>,
    ) -> Self {
        let expected_target = root_to_leaf.last().copied();
        let expected_bubble_route: Vec<_> = root_to_leaf.iter().rev().copied().collect();

        assert_eq!(
            target, expected_target,
            "UiHitPath target must match the final root-to-leaf node"
        );
        assert_eq!(
            bubble_route, expected_bubble_route,
            "UiHitPath bubble route must be the reverse of root-to-leaf"
        );

        self.with_root_to_leaf(root_to_leaf)
    }

    pub fn has_consistent_route(&self) -> bool {
        self.target == self.root_to_leaf.last().copied()
    }
}

impl Serialize for UiHitPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let field_count = 3 + usize::from(self.virtual_pointer.is_some());
        let mut state = serializer.serialize_struct("UiHitPath", field_count)?;
        state.serialize_field("target", &self.target)?;
        state.serialize_field("root_to_leaf", &self.root_to_leaf)?;
        state.serialize_field("bubble_route", &ReverseNodePath(&self.root_to_leaf))?;
        if self.virtual_pointer.is_some() {
            state.serialize_field("virtual_pointer", &self.virtual_pointer)?;
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for UiHitPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct WireHitPath {
            target: Option<UiNodeId>,
            #[serde(default)]
            root_to_leaf: Vec<UiNodeId>,
            #[serde(default)]
            bubble_route: Vec<UiNodeId>,
            #[serde(default)]
            virtual_pointer: Option<UiVirtualPointerPosition>,
        }

        let wire = WireHitPath::deserialize(deserializer)?;
        let mut root_to_leaf = wire.root_to_leaf;
        if root_to_leaf.is_empty() && !wire.bubble_route.is_empty() {
            root_to_leaf = wire.bubble_route;
            root_to_leaf.reverse();
        } else if !wire.bubble_route.is_empty()
            && !wire
                .bubble_route
                .iter()
                .copied()
                .eq(root_to_leaf.iter().rev().copied())
        {
            return Err(D::Error::custom(
                "UiHitPath bubble route must be the reverse of root-to-leaf",
            ));
        }
        if wire.target != root_to_leaf.last().copied() {
            return Err(D::Error::custom(
                "UiHitPath target must match the final root-to-leaf node",
            ));
        }

        Ok(Self {
            target: wire.target,
            root_to_leaf,
            virtual_pointer: wire.virtual_pointer,
        })
    }
}

struct ReverseNodePath<'path>(&'path [UiNodeId]);

impl Serialize for ReverseNodePath<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for node_id in self.0.iter().rev() {
            sequence.serialize_element(node_id)?;
        }
        sequence.end()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiHitRouteNode {
    pub node_id: UiNodeId,
    pub parent_index: u32,
    pub effective_input_policy: UiInputPolicy,
    pub pointer_path_visible: bool,
    pub descendant_pointer_path_visible: bool,
    pub route_valid: bool,
}

impl UiHitRouteNode {
    pub const NO_PARENT_INDEX: u32 = u32::MAX;

    pub const fn invalid(node_id: UiNodeId) -> Self {
        Self {
            node_id,
            parent_index: Self::NO_PARENT_INDEX,
            effective_input_policy: UiInputPolicy::Inherit,
            pointer_path_visible: false,
            descendant_pointer_path_visible: false,
            route_valid: false,
        }
    }

    pub const fn parent_index(self) -> Option<usize> {
        if self.parent_index == Self::NO_PARENT_INDEX {
            None
        } else {
            Some(self.parent_index as usize)
        }
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
    pub route_node_index: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiHitTestCell {
    pub entries: UiHitTestCellEntries,
}

/// Ordered hit-entry membership with an allocation-free empty state.
///
/// Cloned cells share non-empty membership until one generation mutates that cell.
#[derive(Clone, Debug, Default)]
pub struct UiHitTestCellEntries {
    shared: Option<Arc<Vec<usize>>>,
}

impl UiHitTestCellEntries {
    pub fn as_slice(&self) -> &[usize] {
        self.shared
            .as_deref()
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn push(&mut self, entry_index: usize) -> usize {
        let (entries, cloned_entry_count) = self.make_mut_with_clone_count();
        entries.push(entry_index);
        cloned_entry_count
    }

    pub fn insert(&mut self, index: usize, entry_index: usize) -> usize {
        let (entries, cloned_entry_count) = self.make_mut_with_clone_count();
        entries.insert(index, entry_index);
        cloned_entry_count
    }

    pub fn retain(&mut self, mut predicate: impl FnMut(&usize) -> bool) -> usize {
        if self.shared.is_none() {
            return 0;
        }
        let (entries, cloned_entry_count) = self.make_mut_with_clone_count();
        entries.retain(|entry_index| predicate(entry_index));
        cloned_entry_count
    }

    fn make_mut_with_clone_count(&mut self) -> (&mut Vec<usize>, usize) {
        let cloned_entry_count = self
            .shared
            .as_ref()
            .filter(|entries| Arc::strong_count(entries) > 1)
            .map_or(0, |entries| entries.len());
        let entries = self.shared.get_or_insert_with(|| Arc::new(Vec::new()));
        (Arc::make_mut(entries), cloned_entry_count)
    }
}

impl Deref for UiHitTestCellEntries {
    type Target = [usize];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl From<Vec<usize>> for UiHitTestCellEntries {
    fn from(entries: Vec<usize>) -> Self {
        Self {
            shared: (!entries.is_empty()).then(|| Arc::new(entries)),
        }
    }
}

impl PartialEq for UiHitTestCellEntries {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Serialize for UiHitTestCellEntries {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_slice().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for UiHitTestCellEntries {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<usize>::deserialize(deserializer).map(Into::into)
    }
}

impl<'a> IntoIterator for &'a UiHitTestCellEntries {
    type Item = &'a usize;
    type IntoIter = slice::Iter<'a, usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiHitTestGrid {
    pub bounds: UiFrame,
    pub cell_size: f32,
    pub columns: u32,
    pub rows: u32,
    pub scope: UiHitTestScope,
    pub route_nodes: Arc<Vec<UiHitRouteNode>>,
    pub entries: UiPersistentSequence<UiHitTestEntry>,
    pub cells: UiPersistentSequence<UiHitTestCell>,
}

impl Default for UiHitTestGrid {
    fn default() -> Self {
        Self {
            bounds: UiFrame::default(),
            cell_size: 64.0,
            columns: 0,
            rows: 0,
            scope: UiHitTestScope::empty(),
            route_nodes: Arc::new(Vec::new()),
            entries: UiPersistentSequence::default(),
            cells: UiPersistentSequence::default(),
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

#[cfg(test)]
mod cell_entry_tests {
    use super::*;

    #[test]
    fn empty_membership_is_allocation_free_and_keeps_flat_serde() {
        let entries = UiHitTestCellEntries::default();

        assert!(entries.shared.is_none());
        assert!(entries.is_empty());
        assert_eq!(serde_json::to_string(&entries).unwrap(), "[]");
        assert!(
            serde_json::from_str::<UiHitTestCellEntries>("[]")
                .unwrap()
                .shared
                .is_none()
        );
    }

    #[test]
    fn retained_membership_clones_only_the_mutated_cell_entries() {
        let retained: UiHitTestCellEntries = vec![1, 2, 3].into();
        let mut next = retained.clone();

        let cloned_entry_count = next.insert(1, 9);

        assert_eq!(cloned_entry_count, 3);
        assert_eq!(retained.as_slice(), &[1, 2, 3]);
        assert_eq!(next.as_slice(), &[1, 9, 2, 3]);
    }
}
