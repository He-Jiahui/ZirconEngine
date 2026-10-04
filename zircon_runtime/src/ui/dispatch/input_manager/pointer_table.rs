use std::collections::HashMap;

use zircon_runtime_interface::ui::{
    dispatch::{UiPointerId, UiPointerSource},
    event_ui::UiNodeId,
    layout::UiPoint,
    surface::UiPointerButton,
};

const POINTER_BUTTON_PRIMARY_MASK: u8 = 0b001;
const POINTER_BUTTON_SECONDARY_MASK: u8 = 0b010;
const POINTER_BUTTON_MIDDLE_MASK: u8 = 0b100;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiActivePointerTable {
    entries: Vec<UiActivePointerEntry>,
    indices_by_pointer_id: HashMap<UiPointerId, usize>,
    // Primary touch/pen membership is queried for every newly-seen pointer.
    // Keep the small source-specific counters beside the ordered table so that
    // admitting a pointer does not rescan all active entries.
    primary_touch_count: usize,
    primary_pen_count: usize,
}

impl UiActivePointerTable {
    pub fn entries(&self) -> &[UiActivePointerEntry] {
        &self.entries
    }

    pub fn entry(&self, pointer_id: UiPointerId) -> Option<&UiActivePointerEntry> {
        let index = self.indices_by_pointer_id.get(&pointer_id).copied()?;
        self.entries.get(index)
    }

    pub fn entry_mut(&mut self, pointer_id: UiPointerId) -> Option<&mut UiActivePointerEntry> {
        let index = self.indices_by_pointer_id.get(&pointer_id).copied()?;
        self.entries.get_mut(index)
    }

    /// Reports primary membership for sources that participate in touch-like
    /// arbitration; Mouse and Unknown intentionally bypass this check.
    pub fn has_primary_for_source(&self, source: UiPointerSource) -> bool {
        match source {
            UiPointerSource::Touch => self.primary_touch_count != 0,
            UiPointerSource::Pen => self.primary_pen_count != 0,
            UiPointerSource::Mouse | UiPointerSource::Unknown => false,
        }
    }

    pub fn upsert(
        &mut self,
        pointer_id: UiPointerId,
        source: UiPointerSource,
        is_primary: bool,
    ) -> &mut UiActivePointerEntry {
        if let Some(index) = self.indices_by_pointer_id.get(&pointer_id).copied() {
            let (previous_source, previous_is_primary) = {
                let entry = &self.entries[index];
                (entry.source, entry.is_primary)
            };
            if previous_is_primary && (previous_source != source || !is_primary) {
                self.remove_primary_source(previous_source);
            }
            if is_primary && (previous_source != source || !previous_is_primary) {
                self.add_primary_source(source);
            }
            let entry = &mut self.entries[index];
            entry.source = source;
            entry.is_primary = is_primary;
            return entry;
        }

        let index = self.entries.len();
        self.entries.push(UiActivePointerEntry {
            pointer_id,
            source,
            last_point: None,
            hovered: Vec::new(),
            pressed_buttons: 0,
            pressed_target: None,
            capture_target: None,
            is_primary,
        });
        self.indices_by_pointer_id.insert(pointer_id, index);
        if is_primary {
            self.add_primary_source(source);
        }
        self.entries.get_mut(index).expect("entry was just pushed")
    }

    pub fn remove(&mut self, pointer_id: UiPointerId) -> Option<UiActivePointerEntry> {
        let index = self.indices_by_pointer_id.remove(&pointer_id)?;
        let removed = self.entries.remove(index);
        if removed.is_primary {
            self.remove_primary_source(removed.source);
        }
        for (moved_index, moved_entry) in self.entries.iter().enumerate().skip(index) {
            self.indices_by_pointer_id
                .insert(moved_entry.pointer_id, moved_index);
        }
        Some(removed)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.indices_by_pointer_id.clear();
        self.primary_touch_count = 0;
        self.primary_pen_count = 0;
    }

    fn add_primary_source(&mut self, source: UiPointerSource) {
        match source {
            UiPointerSource::Touch => {
                self.primary_touch_count = self.primary_touch_count.saturating_add(1);
            }
            UiPointerSource::Pen => {
                self.primary_pen_count = self.primary_pen_count.saturating_add(1);
            }
            UiPointerSource::Mouse | UiPointerSource::Unknown => {}
        }
    }

    fn remove_primary_source(&mut self, source: UiPointerSource) {
        match source {
            UiPointerSource::Touch => {
                self.primary_touch_count = self.primary_touch_count.saturating_sub(1);
            }
            UiPointerSource::Pen => {
                self.primary_pen_count = self.primary_pen_count.saturating_sub(1);
            }
            UiPointerSource::Mouse | UiPointerSource::Unknown => {}
        }
    }

    pub fn record_point(&mut self, pointer_id: UiPointerId, point: UiPoint) {
        if let Some(entry) = self.entry_mut(pointer_id) {
            entry.last_point = Some(point);
        }
    }

    pub fn set_hovered_path(&mut self, pointer_id: UiPointerId, hovered: impl AsRef<[UiNodeId]>) {
        let hovered = hovered.as_ref();
        self.set_hovered_path_iter(pointer_id, hovered.iter().copied());
    }

    pub fn set_hovered_path_iter(
        &mut self,
        pointer_id: UiPointerId,
        hovered: impl Iterator<Item = UiNodeId> + Clone,
    ) {
        if let Some(entry) = self.entry_mut(pointer_id) {
            if !entry.hovered.iter().copied().eq(hovered.clone()) {
                entry.hovered.clear();
                entry.hovered.extend(hovered);
            }
        }
    }

    pub fn press_button(
        &mut self,
        pointer_id: UiPointerId,
        button: Option<UiPointerButton>,
        target: Option<UiNodeId>,
    ) {
        let Some(mask) = pointer_button_mask(button) else {
            return;
        };
        if let Some(entry) = self.entry_mut(pointer_id) {
            entry.pressed_buttons |= mask;
            entry.pressed_target = target;
        }
    }

    pub fn release_button(&mut self, pointer_id: UiPointerId, button: Option<UiPointerButton>) {
        let Some(mask) = pointer_button_mask(button) else {
            return;
        };
        if let Some(entry) = self.entry_mut(pointer_id) {
            entry.pressed_buttons &= !mask;
            if entry.pressed_buttons == 0 {
                entry.pressed_target = None;
            }
        }
    }

    pub fn set_capture_target(
        &mut self,
        pointer_id: UiPointerId,
        capture_target: Option<UiNodeId>,
    ) {
        if let Some(entry) = self.entry_mut(pointer_id) {
            entry.capture_target = capture_target;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiActivePointerEntry {
    pub pointer_id: UiPointerId,
    pub source: UiPointerSource,
    pub last_point: Option<UiPoint>,
    pub hovered: Vec<UiNodeId>,
    pub pressed_buttons: u8,
    pub pressed_target: Option<UiNodeId>,
    pub capture_target: Option<UiNodeId>,
    pub is_primary: bool,
}

fn pointer_button_mask(button: Option<UiPointerButton>) -> Option<u8> {
    match button {
        Some(UiPointerButton::Primary) => Some(POINTER_BUTTON_PRIMARY_MASK),
        Some(UiPointerButton::Secondary) => Some(POINTER_BUTTON_SECONDARY_MASK),
        Some(UiPointerButton::Middle) => Some(POINTER_BUTTON_MIDDLE_MASK),
        None => None,
    }
}

#[cfg(test)]
#[path = "tests/pointer_table.rs"]
mod tests;

#[cfg(test)]
#[path = "pointer_table/tests/hovered_path_tests.rs"]
mod hovered_path_tests;

#[cfg(test)]
#[path = "pointer_table/tests/index_tests.rs"]
mod index_tests;
