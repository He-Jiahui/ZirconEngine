use zircon_runtime_interface::ui::{
    dispatch::{UiPointerId, UiPointerSource},
    event_ui::UiNodeId,
    layout::UiPoint,
};

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
            pressed_buttons: 0,
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
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiActivePointerEntry {
    pub pointer_id: UiPointerId,
    pub source: UiPointerSource,
    pub last_point: Option<UiPoint>,
    pub pressed_buttons: u8,
    pub capture_target: Option<UiNodeId>,
    pub is_primary: bool,
}
