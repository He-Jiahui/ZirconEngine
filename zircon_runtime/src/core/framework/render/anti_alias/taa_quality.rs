/// 与 TAA resolve 参数上传共享的稳定档位编号；画质配置可在解析后保留该偏好。
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaaQualityPreset {
    Low,
    #[default]
    Medium,
    High,
}

impl TaaQualityPreset {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}
