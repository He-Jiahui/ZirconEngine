/// 相机请求的抗锯齿策略；具体执行模式须结合设备能力和历史资源解析。
/// MSAA 改变场景图采样数，TAA 则占用时间重建阶段，二者不能按同一终端效果处理。
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

    pub const fn graph_sample_count(self) -> u32 {
        match self {
            Self::Msaa { samples } => {
                if samples > 1 {
                    samples
                } else {
                    1
                }
            }
            Self::Off
            | Self::Auto
            | Self::Fxaa
            | Self::Taa
            | Self::Smaa
            | Self::Cas
            | Self::Dlss => 1,
        }
    }
}
