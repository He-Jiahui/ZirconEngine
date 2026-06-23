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
}

impl UiActivePointerTable {
    pub fn entries(&self) -> &[UiActivePointerEntry] {
        &self.entries
    }

    pub fn entry(&self, pointer_id: UiPointerId) -> Option<&UiActivePointerEntry> {
        self.entries
            .iter()
            .find(|entry| entry.pointer_id == pointer_id)
    }

    pub fn entry_mut(&mut self, pointer_id: UiPointerId) -> Option<&mut UiActivePointerEntry> {
        self.entries
            .iter_mut()
            .find(|entry| entry.pointer_id == pointer_id)
    }

    pub fn upsert(
        &mut self,
        pointer_id: UiPointerId,
        source: UiPointerSource,
        is_primary: bool,
    ) -> &mut UiActivePointerEntry {
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.pointer_id == pointer_id)
        {
            let entry = &mut self.entries[index];
            entry.source = source;
            entry.is_primary = is_primary;
            return entry;
        }

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
        self.entries.last_mut().expect("entry was just pushed")
    }

    pub fn remove(&mut self, pointer_id: UiPointerId) -> Option<UiActivePointerEntry> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.pointer_id == pointer_id)?;
        Some(self.entries.remove(index))
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn record_point(&mut self, pointer_id: UiPointerId, point: UiPoint) {
        if let Some(entry) = self.entry_mut(pointer_id) {
            entry.last_point = Some(point);
        }
    }

    pub fn set_hovered_path(&mut self, pointer_id: UiPointerId, hovered: Vec<UiNodeId>) {
        if let Some(entry) = self.entry_mut(pointer_id) {
            entry.hovered = hovered;
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
mod tests {
    use zircon_runtime_interface::ui::{
        dispatch::{UiPointerId, UiPointerSource},
        event_ui::UiNodeId,
        layout::UiPoint,
        surface::UiPointerButton,
    };

    use super::UiActivePointerTable;

    #[test]
    fn active_pointer_table_keeps_hover_press_and_capture_per_pointer() {
        let first_pointer = UiPointerId::new(1);
        let second_pointer = UiPointerId::new(2);
        let mut table = UiActivePointerTable::default();

        table.upsert(first_pointer, UiPointerSource::Touch, true);
        table.record_point(first_pointer, UiPoint::new(10.0, 12.0));
        table.set_hovered_path(first_pointer, vec![UiNodeId::new(3), UiNodeId::new(1)]);
        table.press_button(
            first_pointer,
            Some(UiPointerButton::Primary),
            Some(UiNodeId::new(3)),
        );
        table.set_capture_target(first_pointer, Some(UiNodeId::new(3)));

        table.upsert(second_pointer, UiPointerSource::Touch, false);
        table.record_point(second_pointer, UiPoint::new(80.0, 18.0));
        table.set_hovered_path(second_pointer, vec![UiNodeId::new(4), UiNodeId::new(1)]);
        table.press_button(
            second_pointer,
            Some(UiPointerButton::Primary),
            Some(UiNodeId::new(4)),
        );
        table.set_capture_target(second_pointer, Some(UiNodeId::new(4)));

        let first = table.entry(first_pointer).unwrap();
        assert_eq!(first.last_point, Some(UiPoint::new(10.0, 12.0)));
        assert_eq!(first.hovered, vec![UiNodeId::new(3), UiNodeId::new(1)]);
        assert_eq!(first.pressed_buttons, 0b001);
        assert_eq!(first.pressed_target, Some(UiNodeId::new(3)));
        assert_eq!(first.capture_target, Some(UiNodeId::new(3)));
        assert!(first.is_primary);

        let second = table.entry(second_pointer).unwrap();
        assert_eq!(second.last_point, Some(UiPoint::new(80.0, 18.0)));
        assert_eq!(second.hovered, vec![UiNodeId::new(4), UiNodeId::new(1)]);
        assert_eq!(second.pressed_buttons, 0b001);
        assert_eq!(second.pressed_target, Some(UiNodeId::new(4)));
        assert_eq!(second.capture_target, Some(UiNodeId::new(4)));
        assert!(!second.is_primary);
    }

    #[test]
    fn active_pointer_table_release_clears_only_matching_pointer_button_state() {
        let first_pointer = UiPointerId::new(1);
        let second_pointer = UiPointerId::new(2);
        let mut table = UiActivePointerTable::default();

        table.upsert(first_pointer, UiPointerSource::Mouse, true);
        table.upsert(second_pointer, UiPointerSource::Mouse, false);
        table.press_button(
            first_pointer,
            Some(UiPointerButton::Primary),
            Some(UiNodeId::new(3)),
        );
        table.press_button(
            second_pointer,
            Some(UiPointerButton::Primary),
            Some(UiNodeId::new(4)),
        );

        table.release_button(first_pointer, Some(UiPointerButton::Primary));

        assert_eq!(table.entry(first_pointer).unwrap().pressed_buttons, 0);
        assert_eq!(table.entry(first_pointer).unwrap().pressed_target, None);
        assert_eq!(table.entry(second_pointer).unwrap().pressed_buttons, 0b001);
        assert_eq!(
            table.entry(second_pointer).unwrap().pressed_target,
            Some(UiNodeId::new(4))
        );
    }
}
