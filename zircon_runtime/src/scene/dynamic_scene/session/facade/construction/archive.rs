use super::super::super::construction;
use super::super::super::*;

impl RuntimeSessionArchive {
    pub fn empty() -> Self {
        construction::empty()
    }

    pub fn from_slots(slots: Vec<RuntimeSessionSlot>) -> Result<Self, RuntimeSessionArchiveError> {
        construction::from_slots(slots)
    }
}

impl Default for RuntimeSessionArchive {
    fn default() -> Self {
        Self::empty()
    }
}
