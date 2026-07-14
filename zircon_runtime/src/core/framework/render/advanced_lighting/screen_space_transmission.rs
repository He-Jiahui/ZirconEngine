use serde::{Deserialize, Serialize};

pub const MAX_SCREEN_SPACE_TRANSMISSION_STEPS: usize = 4;

/// Per-view screen-space specular transmission budget.
///
/// A zero step budget keeps the transmission draw but samples only the
/// environment fallback. Positive values request one scene-color copy per
/// depth-sorted draw partition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScreenSpaceTransmissionSettings {
    steps: u8,
}

impl ScreenSpaceTransmissionSettings {
    pub const fn new(steps: usize) -> Self {
        Self {
            steps: if steps > MAX_SCREEN_SPACE_TRANSMISSION_STEPS {
                MAX_SCREEN_SPACE_TRANSMISSION_STEPS as u8
            } else {
                steps as u8
            },
        }
    }

    pub const fn steps(self) -> usize {
        self.steps as usize
    }
}

impl Default for ScreenSpaceTransmissionSettings {
    fn default() -> Self {
        Self::new(1)
    }
}

#[cfg(test)]
mod tests {
    use super::{ScreenSpaceTransmissionSettings, MAX_SCREEN_SPACE_TRANSMISSION_STEPS};

    #[test]
    fn render_screen_space_transmission_settings_normalize_step_budget() {
        assert_eq!(ScreenSpaceTransmissionSettings::default().steps(), 1);
        assert_eq!(ScreenSpaceTransmissionSettings::new(0).steps(), 0);
        assert_eq!(
            ScreenSpaceTransmissionSettings::new(usize::MAX).steps(),
            MAX_SCREEN_SPACE_TRANSMISSION_STEPS
        );
    }
}
