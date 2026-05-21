#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilityStatus<T> {
    Supported(T),
    FeatureDisabled { feature: &'static str },
    Unavailable { reason: &'static str },
}

impl<T> CapabilityStatus<T> {
    pub const fn is_supported(&self) -> bool {
        matches!(self, Self::Supported(_))
    }
}

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
