use serde::{Deserialize, Serialize};

use crate::core::math::UVec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowVideoMode {
    pub physical_size: UVec2,
    pub bit_depth: Option<u16>,
    pub refresh_rate_millihertz: Option<u32>,
}

impl WindowVideoMode {
    pub fn new(physical_width: u32, physical_height: u32) -> Self {
        Self {
            physical_size: UVec2::new(physical_width.max(1), physical_height.max(1)),
            bit_depth: None,
            refresh_rate_millihertz: None,
        }
    }

    pub fn with_bit_depth(mut self, bit_depth: u16) -> Self {
        self.bit_depth = (bit_depth > 0).then_some(bit_depth);
        self
    }

    pub fn with_refresh_rate_millihertz(mut self, refresh_rate_millihertz: u32) -> Self {
        self.refresh_rate_millihertz =
            (refresh_rate_millihertz > 0).then_some(refresh_rate_millihertz);
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowVideoModeSelection {
    #[default]
    Current,
    Specific(WindowVideoMode),
}
