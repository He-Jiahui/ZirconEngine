/// profile 选择实验路径时仍需独立门控；默认关闭以免能力齐全即自动启用。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SolariSettings {
    pub experimental_enabled: bool,
}

impl SolariSettings {
    pub const fn new() -> Self {
        Self {
            experimental_enabled: false,
        }
    }

    pub const fn experimental_enabled() -> Self {
        Self {
            experimental_enabled: true,
        }
    }

    pub const fn with_experimental_enabled(mut self, enabled: bool) -> Self {
        self.experimental_enabled = enabled;
        self
    }
}
