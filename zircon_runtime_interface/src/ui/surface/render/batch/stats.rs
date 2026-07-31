use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBatchStats {
    pub element_count: usize,
    pub batch_count: usize,
    pub draw_call_count: usize,
}
