use serde::{Deserialize, Serialize};

use super::WindowMonitorSelection;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowPosition {
    #[default]
    Automatic,
    Centered,
    CenteredOn(WindowMonitorSelection),
    At {
        x: i32,
        y: i32,
    },
}

impl WindowPosition {
    pub const fn centered_on(monitor: WindowMonitorSelection) -> Self {
        Self::CenteredOn(monitor)
    }
}
