use serde::{Deserialize, Serialize};

use super::GamepadId;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadConnectionInfo {
    pub gamepad: GamepadId,
    pub connected: bool,
    pub name: Option<String>,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
}
