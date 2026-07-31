use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBatchRange {
    pub first_element: usize,
    pub element_count: usize,
}
