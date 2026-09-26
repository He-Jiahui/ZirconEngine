use serde::{Deserialize, Serialize};

use super::GamepadId;

/// 设备连接生命周期消息；断开时输入管理器释放该设备的按钮和轴，不将它误作窗口失焦。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadConnectionInfo {
    pub gamepad: GamepadId,
    pub connected: bool,
    pub name: Option<String>,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
}
