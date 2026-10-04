#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 控制整条选取管线与射线阶段；关闭总开关会清空事件状态，关闭射线阶段则不调用后端。
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
