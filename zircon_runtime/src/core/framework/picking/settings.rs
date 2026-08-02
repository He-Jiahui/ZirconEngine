#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PickingSettings {
    pub enabled: bool,
    pub ray_map_enabled: bool,
}

impl Default for PickingSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl PickingSettings {
    pub const DEFAULT: Self = Self {
        enabled: true,
        ray_map_enabled: true,
    };
}
