mod fallback;
mod mode;
mod settings;
mod taa_quality;

pub use fallback::{AntiAliasFallbackReason, AntiAliasFallbackReport};
pub use mode::AntiAliasMode;
pub use settings::AntiAliasSettings;
pub use taa_quality::TaaQualityPreset;
