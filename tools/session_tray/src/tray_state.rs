use serde::{Deserialize, Serialize};

use crate::coordinator_client::Health;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupervisionState {
    Starting,
    Healthy,
    Degraded,
    Draining,
    Stopping,
    Offline,
    Recovering,
    ReadOnly,
    IdentityMismatch,
    FatalIntegrityError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrayVisualState {
    Starting,
    Healthy,
    Busy,
    Degraded,
    Draining,
    Stopping,
    Offline,
    Recovering,
    ReadOnly,
    IdentityMismatch,
    FatalIntegrityError,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MenuEnablement {
    pub open_console: bool,
    pub start: bool,
    pub drain: bool,
    pub resume: bool,
    pub stop: bool,
    pub restart: bool,
    pub force_stop: bool,
    pub diagnostics: bool,
    pub exit_tray: bool,
}

impl TrayVisualState {
    pub fn key(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Healthy => "healthy",
            Self::Busy => "busy",
            Self::Degraded => "degraded",
            Self::Draining => "draining",
            Self::Stopping => "stopping",
            Self::Offline => "offline",
            Self::Recovering => "recovering",
            Self::ReadOnly => "read_only",
            Self::IdentityMismatch => "identity_mismatch",
            Self::FatalIntegrityError => "fatal_integrity_error",
        }
    }

    pub fn from_health(health: &Health) -> Self {
        match health.supervision.state {
            SupervisionState::Starting => Self::Starting,
            SupervisionState::Healthy if health.supervision.busy => Self::Busy,
            SupervisionState::Healthy => Self::Healthy,
            SupervisionState::Degraded => Self::Degraded,
            SupervisionState::Draining => Self::Draining,
            SupervisionState::Stopping => Self::Stopping,
            SupervisionState::Offline => Self::Offline,
            SupervisionState::Recovering => Self::Recovering,
            SupervisionState::ReadOnly => Self::ReadOnly,
            SupervisionState::IdentityMismatch => Self::IdentityMismatch,
            SupervisionState::FatalIntegrityError => Self::FatalIntegrityError,
        }
    }

    pub fn menu(self, identity_verified: bool) -> MenuEnablement {
        let online = !matches!(self, Self::Offline | Self::IdentityMismatch);
        MenuEnablement {
            open_console: online && identity_verified,
            start: matches!(self, Self::Offline) && identity_verified,
            drain: matches!(self, Self::Healthy | Self::Busy | Self::Degraded) && identity_verified,
            resume: matches!(self, Self::Draining) && identity_verified,
            stop: matches!(
                self,
                Self::Healthy | Self::Busy | Self::Degraded | Self::Draining
            ) && identity_verified,
            restart: matches!(
                self,
                Self::Healthy | Self::Busy | Self::Degraded | Self::Draining
            ) && identity_verified,
            force_stop: matches!(self, Self::Degraded | Self::Draining) && identity_verified,
            diagnostics: true,
            exit_tray: true,
        }
    }

    pub fn tooltip(self) -> &'static str {
        match self {
            Self::Starting => "Zircon Coordinator：启动中",
            Self::Healthy => "Zircon Coordinator：健康",
            Self::Busy => "Zircon Coordinator：有活动任务",
            Self::Degraded => "Zircon Coordinator：降级",
            Self::Draining => "Zircon Coordinator：排空中",
            Self::Stopping => "Zircon Coordinator：停止中",
            Self::Offline => "Zircon Coordinator：离线",
            Self::Recovering => "Zircon Coordinator：恢复中",
            Self::ReadOnly => "Zircon Coordinator：只读",
            Self::IdentityMismatch => "Zircon Coordinator：身份不匹配",
            Self::FatalIntegrityError => "Zircon Coordinator：完整性错误",
        }
    }

    pub fn icon_rgba(self, size: u32) -> Vec<u8> {
        let color = self.icon_color();
        let mut pixels = vec![0; (size * size * 4) as usize];
        let center = (size as f32 - 1.0) / 2.0;
        let radius = (size as f32 * 0.42).max(1.0);
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                if dx * dx + dy * dy <= radius * radius {
                    let offset = ((y * size + x) * 4) as usize;
                    pixels[offset..offset + 4].copy_from_slice(&color);
                }
            }
        }
        pixels
    }

    fn icon_color(self) -> [u8; 4] {
        match self {
            Self::Starting => [66, 133, 244, 255],
            Self::Healthy => [35, 166, 90, 255],
            Self::Busy => [246, 173, 0, 255],
            Self::Degraded => [230, 126, 34, 255],
            Self::Draining => [142, 68, 173, 255],
            Self::Stopping => [211, 47, 47, 255],
            Self::Offline => [117, 117, 117, 255],
            Self::Recovering => [0, 151, 167, 255],
            Self::ReadOnly => [84, 110, 122, 255],
            Self::IdentityMismatch => [194, 24, 91, 255],
            Self::FatalIntegrityError => [127, 0, 0, 255],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_operations_are_disabled_by_state_and_identity() {
        let healthy = TrayVisualState::Healthy.menu(true);
        assert!(healthy.drain);
        assert!(healthy.stop);
        assert!(!healthy.start);
        assert!(!healthy.resume);

        let draining = TrayVisualState::Draining.menu(true);
        assert!(draining.resume);
        assert!(!draining.drain);

        let mismatch = TrayVisualState::IdentityMismatch.menu(false);
        assert!(!mismatch.stop);
        assert!(!mismatch.force_stop);
        assert!(mismatch.exit_tray);
    }

    #[test]
    fn every_visual_state_has_a_visible_distinct_status_icon() {
        let states = [
            TrayVisualState::Starting,
            TrayVisualState::Healthy,
            TrayVisualState::Busy,
            TrayVisualState::Degraded,
            TrayVisualState::Draining,
            TrayVisualState::Stopping,
            TrayVisualState::Offline,
            TrayVisualState::Recovering,
            TrayVisualState::ReadOnly,
            TrayVisualState::IdentityMismatch,
            TrayVisualState::FatalIntegrityError,
        ];
        let mut colors = std::collections::HashSet::new();
        for state in states {
            let pixels = state.icon_rgba(16);
            assert_eq!(16 * 16 * 4, pixels.len());
            assert!(pixels.chunks_exact(4).any(|pixel| pixel[3] == 255));
            colors.insert(state.icon_color());
        }
        assert_eq!(states.len(), colors.len());
    }
}
