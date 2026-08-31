use std::collections::{HashMap, HashSet};

use crate::core::framework::window::{NativeWindowId, WindowId, WindowRegistryId};
use zircon_runtime_interface::ZrRuntimeViewportHandle;

use super::relationship::WindowParentRelation;
use super::slot::WindowRegistrySlot;
use super::{PrimaryWindowRoleChange, WindowCloseBegin, WindowParentKind, WindowRegistryError};

/// Main-thread platform-host ownership of the engine/native window mapping.
///
/// A slot only becomes reusable after the host has marked it closing and
/// completed external teardown such as surface-lease retirement. Reuse always
/// advances generation, and a wrapped slot is permanently retired.
pub(crate) struct WindowRegistry {
    registry_id: WindowRegistryId,
    slots: Vec<WindowRegistrySlot>,
    reusable_slots: Vec<u32>,
    native_to_slot: HashMap<NativeWindowId, u32>,
    viewport_to_window: HashMap<ZrRuntimeViewportHandle, WindowId>,
    primary_window: Option<WindowId>,
    primary_generation: u64,
}

impl WindowRegistry {
    pub(crate) fn new(registry_id: WindowRegistryId) -> Self {
        Self {
            registry_id,
            slots: Vec::new(),
            reusable_slots: Vec::new(),
            native_to_slot: HashMap::new(),
            viewport_to_window: HashMap::new(),
            primary_window: None,
            primary_generation: 0,
        }
    }

    pub(crate) const fn registry_id(&self) -> WindowRegistryId {
        self.registry_id
    }

    pub(crate) fn register(
        &mut self,
        native_window: NativeWindowId,
    ) -> Result<WindowId, WindowRegistryError> {
        if self.native_to_slot.contains_key(&native_window) {
            return Err(WindowRegistryError::DuplicateNativeWindow { native_window });
        }

        self.native_to_slot
            .try_reserve(1)
            .map_err(|_| WindowRegistryError::SlotCapacityExhausted)?;
        let slot_index = self.allocate_slot()?;
        let slot = &mut self.slots[slot_index as usize];
        debug_assert!(slot.native_window.is_none());
        debug_assert!(!slot.closing);
        debug_assert!(slot.viewports.is_empty());
        slot.native_window = Some(native_window);
        let window = WindowId::new(self.registry_id, slot_index, slot.generation);
        let previous = self.native_to_slot.insert(native_window, slot_index);
        debug_assert!(previous.is_none());
        Ok(window)
    }

    pub(crate) fn resolve_native(
        &self,
        native_window: NativeWindowId,
    ) -> Result<WindowId, WindowRegistryError> {
        let Some(slot_index) = self.native_to_slot.get(&native_window).copied() else {
            return Err(WindowRegistryError::UnknownNativeWindow { native_window });
        };
        let Some(slot) = self.slots.get(slot_index as usize) else {
            return Err(WindowRegistryError::InconsistentNativeWindowMapping {
                native_window,
                slot: slot_index,
            });
        };
        if slot.native_window != Some(native_window) {
            return Err(WindowRegistryError::InconsistentNativeWindowMapping {
                native_window,
                slot: slot_index,
            });
        }
        let window = WindowId::new(self.registry_id, slot_index, slot.generation);
        if slot.closing {
            return Err(WindowRegistryError::ClosingWindow { window });
        }
        Ok(window)
    }

    pub(crate) fn native_for(
        &self,
        window: WindowId,
    ) -> Result<NativeWindowId, WindowRegistryError> {
        let slot = self.resolve_live_slot(window)?;
        if slot.closing {
            return Err(WindowRegistryError::ClosingWindow { window });
        }
        slot.native_window
            .ok_or(WindowRegistryError::StaleWindow { window })
    }

    /// Stops normal event and command routing while external owners release
    /// the generation-qualified work associated with this window.
    pub(crate) fn begin_close(
        &mut self,
        window: WindowId,
    ) -> Result<WindowCloseBegin, WindowRegistryError> {
        let slot = self.resolve_routable_slot(window)?;
        if !slot.children.is_empty() {
            return Err(WindowRegistryError::WindowHasLiveChildren {
                window,
                child_count: slot.children.len(),
            });
        }
        let mut close_order = Vec::new();
        close_order
            .try_reserve(1)
            .map_err(|_| WindowRegistryError::RelationshipCapacityExhausted)?;
        close_order.push(window);
        let mut close_entries = self.begin_close_order(close_order)?;
        debug_assert_eq!(close_entries.len(), 1);
        close_entries
            .pop()
            .ok_or(WindowRegistryError::InconsistentCloseTransaction { window })
    }

    /// Starts a single close transaction for a relationship subtree.
    ///
    /// Entries are post-order: callers retire the deepest child resources
    /// before their parent and then complete `finish_destroy` in this order.
    pub(crate) fn begin_close_tree(
        &mut self,
        root: WindowId,
    ) -> Result<Vec<WindowCloseBegin>, WindowRegistryError> {
        let close_order = self.preflight_close_tree(root)?;
        self.begin_close_order_after_preflight(close_order)
    }

    /// Produces the unique child-first close order and validates every
    /// registry-side invariant without changing window routing.
    pub(crate) fn preflight_close_tree(
        &self,
        root: WindowId,
    ) -> Result<Vec<WindowId>, WindowRegistryError> {
        let close_order = self.collect_close_order(root)?;
        self.preflight_close_order(&close_order)?;
        Ok(close_order)
    }

    /// Commits a close order returned by `preflight_close_tree`. The method
    /// repeats its local validation so direct callers never gain an unchecked
    /// mutation path.
    pub(crate) fn begin_close_order_after_preflight(
        &mut self,
        close_order: Vec<WindowId>,
    ) -> Result<Vec<WindowCloseBegin>, WindowRegistryError> {
        self.begin_close_order(close_order)
    }

    /// Removes both mapping directions after the caller has completed surface
    /// and native-window teardown for the closing generation.
    pub(crate) fn finish_destroy(
        &mut self,
        window: WindowId,
    ) -> Result<NativeWindowId, WindowRegistryError> {
        let slot = self.resolve_live_slot(window)?;
        if !slot.closing {
            return Err(WindowRegistryError::WindowNotClosing { window });
        }
        if !slot.children.is_empty() {
            return Err(WindowRegistryError::WindowHasLiveChildren {
                window,
                child_count: slot.children.len(),
            });
        }
        if !slot.viewports.is_empty() {
            return Err(WindowRegistryError::WindowHasLiveViewportBindings {
                window,
                viewport_count: slot.viewports.len(),
            });
        }
        let native_window = slot
            .native_window
            .ok_or(WindowRegistryError::StaleWindow { window })?;
        if self.native_to_slot.get(&native_window).copied() != Some(window.slot()) {
            return Err(WindowRegistryError::InconsistentNativeWindowMapping {
                native_window,
                slot: window.slot(),
            });
        }
        let parent = slot.parent;
        if let Some(parent) = parent {
            self.ensure_parent_contains_child(parent.window, window)?;
        }
        let next_generation = window
            .generation()
            .checked_add(1)
            .and_then(std::num::NonZeroU32::new);
        if next_generation.is_some() {
            self.reusable_slots
                .try_reserve(1)
                .map_err(|_| WindowRegistryError::SlotCapacityExhausted)?;
        }

        if let Some(parent) = parent {
            self.detach_child_from_parent(parent.window, window)?;
        }
        {
            let slot = self.resolve_live_slot_mut(window)?;
            slot.native_window = None;
            slot.closing = false;
            slot.parent = None;
        };
        let removed = self.native_to_slot.remove(&native_window);
        debug_assert_eq!(removed, Some(window.slot()));
        debug_assert_ne!(self.primary_window, Some(window));
        debug_assert!(!self
            .viewport_to_window
            .values()
            .any(|candidate| *candidate == window));
        if let Some(next_generation) = next_generation {
            self.slots[window.slot() as usize].generation = next_generation;
            self.reusable_slots.push(window.slot());
        }
        Ok(native_window)
    }

    pub(crate) fn len(&self) -> usize {
        self.native_to_slot.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.native_to_slot.is_empty()
    }

    pub(crate) const fn primary_window(&self) -> Option<WindowId> {
        self.primary_window
    }

    pub(crate) const fn primary_generation(&self) -> u64 {
        self.primary_generation
    }

    pub(crate) fn parent_of(
        &self,
        window: WindowId,
    ) -> Result<Option<(WindowId, WindowParentKind)>, WindowRegistryError> {
        Ok(self
            .resolve_live_slot(window)?
            .parent
            .map(|parent| (parent.window, parent.kind)))
    }

    pub(crate) fn children_of(
        &self,
        window: WindowId,
    ) -> Result<Vec<WindowId>, WindowRegistryError> {
        let slot = self.resolve_live_slot(window)?;
        let mut children = Vec::new();
        children
            .try_reserve(slot.children.len())
            .map_err(|_| WindowRegistryError::RelationshipCapacityExhausted)?;
        children.extend_from_slice(&slot.children);
        Ok(children)
    }

    /// Binds one ABI viewport to a live window generation.
    ///
    /// A viewport has exactly one native window owner, while a render window
    /// may bind multiple viewports and tool windows may bind none.
    pub(crate) fn bind_viewport(
        &mut self,
        window: WindowId,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<(), WindowRegistryError> {
        if !viewport.is_valid() {
            return Err(WindowRegistryError::InvalidViewport { viewport });
        }
        self.resolve_routable_slot(window)?;
        if let Some(bound_window) = self.viewport_to_window.get(&viewport).copied() {
            return Err(WindowRegistryError::ViewportAlreadyBound {
                viewport,
                window: bound_window,
            });
        }
        self.slots[window.slot() as usize]
            .viewports
            .try_reserve(1)
            .map_err(|_| WindowRegistryError::RelationshipCapacityExhausted)?;
        self.viewport_to_window
            .try_reserve(1)
            .map_err(|_| WindowRegistryError::RelationshipCapacityExhausted)?;
        self.slots[window.slot() as usize].viewports.push(viewport);
        let previous = self.viewport_to_window.insert(viewport, window);
        debug_assert!(previous.is_none());
        Ok(())
    }

    pub(crate) fn unbind_viewport(
        &mut self,
        window: WindowId,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<(), WindowRegistryError> {
        self.resolve_routable_slot(window)?;
        let Some(position) = self.slots[window.slot() as usize]
            .viewports
            .iter()
            .position(|candidate| *candidate == viewport)
        else {
            return Err(WindowRegistryError::UnknownViewportBinding { viewport });
        };
        let Some(bound_window) = self.viewport_to_window.get(&viewport).copied() else {
            return Err(WindowRegistryError::InconsistentViewportBinding { window, viewport });
        };
        if bound_window != window {
            return Err(WindowRegistryError::InconsistentViewportBinding { window, viewport });
        }
        let removed_window = self.viewport_to_window.remove(&viewport);
        debug_assert_eq!(removed_window, Some(window));
        self.slots[window.slot() as usize]
            .viewports
            .remove(position);
        Ok(())
    }

    pub(crate) fn window_for_viewport(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<WindowId, WindowRegistryError> {
        let Some(window) = self.viewport_to_window.get(&viewport).copied() else {
            return Err(WindowRegistryError::UnknownViewportBinding { viewport });
        };
        self.resolve_routable_slot(window)?;
        Ok(window)
    }

    pub(crate) fn viewports_for(
        &self,
        window: WindowId,
    ) -> Result<Vec<ZrRuntimeViewportHandle>, WindowRegistryError> {
        let slot = self.resolve_routable_slot(window)?;
        let mut viewports = Vec::new();
        viewports
            .try_reserve(slot.viewports.len())
            .map_err(|_| WindowRegistryError::RelationshipCapacityExhausted)?;
        viewports.extend_from_slice(&slot.viewports);
        Ok(viewports)
    }

    pub(crate) fn set_parent(
        &mut self,
        child: WindowId,
        parent: WindowId,
        kind: WindowParentKind,
    ) -> Result<(), WindowRegistryError> {
        if child == parent {
            return Err(WindowRegistryError::WindowRelationshipCycle { child, parent });
        }
        self.resolve_routable_slot(child)?;
        self.resolve_routable_slot(parent)?;
        self.ensure_parent_chain_is_acyclic(child, parent)?;

        let next_relation = WindowParentRelation::new(parent, kind);
        let current_relation = self.resolve_live_slot(child)?.parent;
        if current_relation == Some(next_relation) {
            return Ok(());
        }
        if current_relation.is_some_and(|current| current.window == parent) {
            self.ensure_parent_contains_child(parent, child)?;
            self.slots[child.slot() as usize].parent = Some(next_relation);
            return Ok(());
        }
        self.slots[parent.slot() as usize]
            .children
            .try_reserve(1)
            .map_err(|_| WindowRegistryError::RelationshipCapacityExhausted)?;
        if let Some(current_relation) = current_relation {
            self.ensure_parent_contains_child(current_relation.window, child)?;
            self.detach_child_from_parent(current_relation.window, child)?;
        }
        self.slots[child.slot() as usize].parent = Some(next_relation);
        self.slots[parent.slot() as usize].children.push(child);
        Ok(())
    }

    pub(crate) fn clear_parent(&mut self, child: WindowId) -> Result<(), WindowRegistryError> {
        self.resolve_routable_slot(child)?;
        let Some(parent) = self.resolve_live_slot(child)?.parent else {
            return Ok(());
        };
        self.ensure_parent_contains_child(parent.window, child)?;
        self.detach_child_from_parent(parent.window, child)?;
        self.slots[child.slot() as usize].parent = None;
        Ok(())
    }

    /// Selects a live, routable window for the primary role.
    ///
    /// There is no implicit first-window promotion: deciding whether a new
    /// window is primary belongs to its creation policy, not to an identity
    /// registry that lacks window kind and application ownership context.
    ///
    /// The registry does not infer application exit behavior from this role;
    /// policy owners consume `PrimaryWindowRoleChange` independently.
    pub(crate) fn set_primary(
        &mut self,
        window: WindowId,
    ) -> Result<Option<PrimaryWindowRoleChange>, WindowRegistryError> {
        let slot = self.resolve_live_slot(window)?;
        if slot.closing {
            return Err(WindowRegistryError::ClosingWindow { window });
        }
        self.replace_primary(Some(window))
    }

    fn replace_primary(
        &mut self,
        current: Option<WindowId>,
    ) -> Result<Option<PrimaryWindowRoleChange>, WindowRegistryError> {
        let previous = self.primary_window;
        if previous == current {
            return Ok(None);
        }
        let next_generation = self
            .primary_generation
            .checked_add(1)
            .ok_or(WindowRegistryError::PrimaryRoleGenerationExhausted)?;
        self.primary_window = current;
        self.primary_generation = next_generation;
        Ok(Some(PrimaryWindowRoleChange::new(
            previous,
            current,
            next_generation,
        )))
    }

    fn collect_close_order(&self, root: WindowId) -> Result<Vec<WindowId>, WindowRegistryError> {
        self.resolve_routable_slot(root)?;
        let mut close_order = Vec::new();
        let mut pending = Vec::new();
        let mut visited = HashSet::new();
        pending
            .try_reserve(1)
            .map_err(|_| WindowRegistryError::RelationshipCapacityExhausted)?;
        pending.push((root, false));

        while let Some((window, expanded)) = pending.pop() {
            if expanded {
                close_order
                    .try_reserve(1)
                    .map_err(|_| WindowRegistryError::RelationshipCapacityExhausted)?;
                close_order.push(window);
                continue;
            }
            visited
                .try_reserve(1)
                .map_err(|_| WindowRegistryError::RelationshipCapacityExhausted)?;
            if !visited.insert(window) {
                return Err(WindowRegistryError::WindowRelationshipCycle {
                    child: root,
                    parent: window,
                });
            }
            let slot = self.resolve_routable_slot(window)?;
            let pending_capacity = slot
                .children
                .len()
                .checked_add(1)
                .ok_or(WindowRegistryError::RelationshipCapacityExhausted)?;
            pending
                .try_reserve(pending_capacity)
                .map_err(|_| WindowRegistryError::RelationshipCapacityExhausted)?;
            pending.push((window, true));
            for child in slot.children.iter().rev() {
                pending.push((*child, false));
            }
        }
        Ok(close_order)
    }

    fn begin_close_order(
        &mut self,
        close_order: Vec<WindowId>,
    ) -> Result<Vec<WindowCloseBegin>, WindowRegistryError> {
        let mut close_entries = Vec::new();
        close_entries
            .try_reserve(close_order.len())
            .map_err(|_| WindowRegistryError::RelationshipCapacityExhausted)?;
        self.preflight_close_order(&close_order)?;
        let primary = self.primary_window;
        let primary_role_change = if primary.is_some_and(|window| close_order.contains(&window)) {
            self.replace_primary(None)?
        } else {
            None
        };
        for window in close_order {
            let (native_window, viewports) = {
                let slot = self.resolve_routable_slot_mut(window)?;
                let native_window = slot
                    .native_window
                    .ok_or(WindowRegistryError::StaleWindow { window })?;
                slot.closing = true;
                let viewports = std::mem::take(&mut slot.viewports);
                (native_window, viewports)
            };
            for viewport in &viewports {
                let removed_window = self.viewport_to_window.remove(viewport);
                debug_assert_eq!(removed_window, Some(window));
            }
            let role_change = if primary == Some(window) {
                primary_role_change
            } else {
                None
            };
            close_entries.push(WindowCloseBegin::new(
                window,
                native_window,
                role_change,
                viewports,
            ));
        }
        Ok(close_entries)
    }

    fn preflight_close_order(&self, close_order: &[WindowId]) -> Result<(), WindowRegistryError> {
        self.ensure_viewport_bindings_match(close_order)?;
        self.ensure_native_mappings_match(close_order)?;
        if self
            .primary_window
            .is_some_and(|window| close_order.contains(&window))
        {
            self.primary_generation
                .checked_add(1)
                .ok_or(WindowRegistryError::PrimaryRoleGenerationExhausted)?;
        }
        Ok(())
    }

    fn ensure_viewport_bindings_match(
        &self,
        close_order: &[WindowId],
    ) -> Result<(), WindowRegistryError> {
        let mut closing_windows = HashSet::new();
        closing_windows
            .try_reserve(close_order.len())
            .map_err(|_| WindowRegistryError::RelationshipCapacityExhausted)?;
        let mut expected_bindings = HashSet::new();
        for window in close_order {
            closing_windows.insert(*window);
            let slot = self.resolve_routable_slot(*window)?;
            expected_bindings
                .try_reserve(slot.viewports.len())
                .map_err(|_| WindowRegistryError::RelationshipCapacityExhausted)?;
            for viewport in &slot.viewports {
                if self.viewport_to_window.get(viewport).copied() != Some(*window) {
                    return Err(WindowRegistryError::InconsistentViewportBinding {
                        window: *window,
                        viewport: *viewport,
                    });
                }
                if !expected_bindings.insert((*window, *viewport)) {
                    return Err(WindowRegistryError::InconsistentViewportBinding {
                        window: *window,
                        viewport: *viewport,
                    });
                }
            }
        }

        // Closing retires the window generation, so a reverse-only binding
        // must fail the transaction rather than survive as a stale owner.
        for (viewport, window) in &self.viewport_to_window {
            if closing_windows.contains(window)
                && !expected_bindings.contains(&(*window, *viewport))
            {
                return Err(WindowRegistryError::InconsistentViewportBinding {
                    window: *window,
                    viewport: *viewport,
                });
            }
        }
        Ok(())
    }

    fn ensure_native_mappings_match(
        &self,
        close_order: &[WindowId],
    ) -> Result<(), WindowRegistryError> {
        for window in close_order {
            let slot = self.resolve_routable_slot(*window)?;
            let native_window = slot
                .native_window
                .ok_or(WindowRegistryError::StaleWindow { window: *window })?;
            if self.native_to_slot.get(&native_window).copied() != Some(window.slot()) {
                return Err(WindowRegistryError::InconsistentNativeWindowMapping {
                    native_window,
                    slot: window.slot(),
                });
            }
        }
        Ok(())
    }

    fn ensure_parent_chain_is_acyclic(
        &self,
        child: WindowId,
        parent: WindowId,
    ) -> Result<(), WindowRegistryError> {
        let mut ancestor = Some(parent);
        while let Some(current) = ancestor {
            if current == child {
                return Err(WindowRegistryError::WindowRelationshipCycle { child, parent });
            }
            ancestor = self
                .resolve_live_slot(current)?
                .parent
                .map(|relation| relation.window);
        }
        Ok(())
    }

    fn ensure_parent_contains_child(
        &self,
        parent: WindowId,
        child: WindowId,
    ) -> Result<(), WindowRegistryError> {
        let parent_slot = self.resolve_live_slot(parent)?;
        if parent_slot
            .children
            .iter()
            .filter(|candidate| **candidate == child)
            .count()
            != 1
        {
            return Err(WindowRegistryError::InconsistentWindowRelationship { parent, child });
        }
        Ok(())
    }

    fn detach_child_from_parent(
        &mut self,
        parent: WindowId,
        child: WindowId,
    ) -> Result<(), WindowRegistryError> {
        self.ensure_parent_contains_child(parent, child)?;
        let parent_slot = self.resolve_live_slot_mut(parent)?;
        parent_slot.children.retain(|candidate| *candidate != child);
        Ok(())
    }

    fn allocate_slot(&mut self) -> Result<u32, WindowRegistryError> {
        if let Some(slot_index) = self.reusable_slots.pop() {
            return Ok(slot_index);
        }
        let slot_index = u32::try_from(self.slots.len())
            .map_err(|_| WindowRegistryError::SlotCapacityExhausted)?;
        if slot_index == u32::MAX {
            return Err(WindowRegistryError::SlotCapacityExhausted);
        }
        self.slots
            .try_reserve(1)
            .map_err(|_| WindowRegistryError::SlotCapacityExhausted)?;
        self.slots.push(WindowRegistrySlot::default());
        Ok(slot_index)
    }

    fn resolve_live_slot(
        &self,
        window: WindowId,
    ) -> Result<&WindowRegistrySlot, WindowRegistryError> {
        if window.registry() != self.registry_id {
            return Err(WindowRegistryError::UnknownWindow { window });
        }
        let Some(slot) = self.slots.get(window.slot() as usize) else {
            return Err(WindowRegistryError::UnknownWindow { window });
        };
        if slot.generation.get() != window.generation() || slot.native_window.is_none() {
            return Err(WindowRegistryError::StaleWindow { window });
        }
        Ok(slot)
    }

    fn resolve_routable_slot(
        &self,
        window: WindowId,
    ) -> Result<&WindowRegistrySlot, WindowRegistryError> {
        let slot = self.resolve_live_slot(window)?;
        if slot.closing {
            return Err(WindowRegistryError::ClosingWindow { window });
        }
        Ok(slot)
    }

    fn resolve_routable_slot_mut(
        &mut self,
        window: WindowId,
    ) -> Result<&mut WindowRegistrySlot, WindowRegistryError> {
        let slot = self.resolve_live_slot_mut(window)?;
        if slot.closing {
            return Err(WindowRegistryError::ClosingWindow { window });
        }
        Ok(slot)
    }

    fn resolve_live_slot_mut(
        &mut self,
        window: WindowId,
    ) -> Result<&mut WindowRegistrySlot, WindowRegistryError> {
        if window.registry() != self.registry_id {
            return Err(WindowRegistryError::UnknownWindow { window });
        }
        let Some(slot) = self.slots.get_mut(window.slot() as usize) else {
            return Err(WindowRegistryError::UnknownWindow { window });
        };
        if slot.generation.get() != window.generation() || slot.native_window.is_none() {
            return Err(WindowRegistryError::StaleWindow { window });
        }
        Ok(slot)
    }
}
