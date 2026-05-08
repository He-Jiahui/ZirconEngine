use super::Pickable;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PickingSettings {
    pub enabled: bool,
    pub ray_map_enabled: bool,
    pub target_priority_enabled: bool,
    pub default_pickable: Pickable,
}

impl Default for PickingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            ray_map_enabled: true,
            target_priority_enabled: true,
            default_pickable: Pickable::default(),
        }
    }
}
