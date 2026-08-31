use zircon_runtime_interface::ui::event_ui::UiNodeId;

/// Stable external model identity associated with a logical virtual-list item.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UiVirtualListItemKey(u128);

impl UiVirtualListItemKey {
    pub const fn new(value: u128) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u128 {
        self.0
    }
}

/// Stable logical identity independent of the physical row slot realizing it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UiVirtualListItemIdentity {
    pub owner_id: UiNodeId,
    pub logical_index: usize,
    pub item_key: UiVirtualListItemKey,
}

/// Current physical realization of a logical item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiVirtualListNodeBinding {
    pub owner_id: UiNodeId,
    pub slot_index: usize,
    pub slot_root_id: UiNodeId,
    pub logical_index: usize,
    pub item_key: UiVirtualListItemKey,
    /// Monotonic owner generation captured only when this slot assignment changes.
    pub assignment_generation: u64,
}

impl UiVirtualListNodeBinding {
    pub const fn item_identity(self) -> UiVirtualListItemIdentity {
        UiVirtualListItemIdentity {
            owner_id: self.owner_id,
            logical_index: self.logical_index,
            item_key: self.item_key,
        }
    }
}
