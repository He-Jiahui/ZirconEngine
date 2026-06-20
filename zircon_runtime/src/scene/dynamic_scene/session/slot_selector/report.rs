use serde::{Deserialize, Serialize};

use super::super::RuntimeSessionSlotSummary;
use super::selector::RuntimeSessionSlotSelector;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSessionSlotSelectionReport {
    pub selector: RuntimeSessionSlotSelector,
    pub selected_slot_id: String,
    pub summary: RuntimeSessionSlotSummary,
}
