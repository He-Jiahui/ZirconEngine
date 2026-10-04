#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilityStatus<T> {
    Supported(T),
    FeatureDisabled { feature: &'static str },
    Unavailable { reason: &'static str },
}

impl<T> CapabilityStatus<T> {
    // 该谓词只回答静态目录是否选出后端，不代表运行时宿主已经安装或观测到它。
    pub const fn is_supported(&self) -> bool {
        matches!(self, Self::Supported(_))
    }

    pub(in crate::platform) const fn platform_disabled() -> Self {
        Self::FeatureDisabled {
            feature: "platform",
        }
    }
}

// 诊断字符串由此统一产生稳定的 supported:/feature_disabled:/unavailable: 前缀；
// backend enum 的 as_str token 因而成为外部契约，测试会锁定这些值。
pub(super) fn format_capability<T>(
    status: CapabilityStatus<T>,
    supported_value: impl FnOnce(T) -> &'static str,
) -> String {
    match status {
        CapabilityStatus::Supported(value) => format!("supported:{}", supported_value(value)),
        CapabilityStatus::FeatureDisabled { feature } => {
            format!("feature_disabled:{feature}")
        }
        CapabilityStatus::Unavailable { reason } => format!("unavailable:{reason}"),
    }
}
