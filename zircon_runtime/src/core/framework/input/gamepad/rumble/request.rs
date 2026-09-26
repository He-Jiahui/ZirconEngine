use serde::{Deserialize, Serialize};

use super::super::device::GamepadId;
use super::GamepadRumbleIntensity;

/// 动作侧发往宿主设备驱动的命令；管理器排空请求后才由宿主执行，不代表振动已生效。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum GamepadRumbleRequest {
    Add {
        gamepad: GamepadId,
        intensity: GamepadRumbleIntensity,
        duration_millis: u32,
    },
    Stop {
        gamepad: GamepadId,
    },
}

impl GamepadRumbleRequest {
    pub const fn add(
        gamepad: GamepadId,
        intensity: GamepadRumbleIntensity,
        duration_millis: u32,
    ) -> Self {
        Self::Add {
            gamepad,
            intensity,
            duration_millis,
        }
    }

    pub const fn stop(gamepad: GamepadId) -> Self {
        Self::Stop { gamepad }
    }

    pub const fn gamepad(self) -> GamepadId {
        match self {
            Self::Add { gamepad, .. } | Self::Stop { gamepad } => gamepad,
        }
    }
}
