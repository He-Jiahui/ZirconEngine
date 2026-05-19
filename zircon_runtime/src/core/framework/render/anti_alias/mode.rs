#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AntiAliasMode {
    Off,
    Auto,
    Fxaa,
    Msaa { samples: u32 },
    Taa,
    Smaa,
    Cas,
    Dlss,
}

impl AntiAliasMode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Auto => "auto",
            Self::Fxaa => "fxaa",
            Self::Msaa { .. } => "msaa",
            Self::Taa => "taa",
            Self::Smaa => "smaa",
            Self::Cas => "cas",
            Self::Dlss => "dlss",
        }
    }

    pub const fn requires_history(self) -> bool {
        matches!(self, Self::Taa)
    }

    pub const fn is_post_process(self) -> bool {
        matches!(
            self,
            Self::Fxaa | Self::Smaa | Self::Taa | Self::Cas | Self::Dlss
        )
    }
}
