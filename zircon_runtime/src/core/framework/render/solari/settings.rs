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
