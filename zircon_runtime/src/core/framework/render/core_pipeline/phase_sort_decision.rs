use serde::{Deserialize, Serialize};

use super::RenderPhaseSortDecisionField;

/// The first lane that orders one render phase breakdown before another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderPhaseSortDecision {
    pub field: RenderPhaseSortDecisionField,
    pub left_value: i128,
    pub right_value: i128,
    pub left_before_right: bool,
}

impl RenderPhaseSortDecision {
    pub const fn new(
        field: RenderPhaseSortDecisionField,
        left_value: i128,
        right_value: i128,
    ) -> Self {
        Self::from_order_values(field, left_value, right_value, left_value, right_value)
    }

    pub const fn from_order_values(
        field: RenderPhaseSortDecisionField,
        left_value: i128,
        right_value: i128,
        left_order_value: i128,
        right_order_value: i128,
    ) -> Self {
        Self {
            field,
            left_value,
            right_value,
            left_before_right: left_order_value < right_order_value,
        }
    }
}
