//! 相机提交阶段将这里的“请求模式”解析为设备可执行模式，再由后处理栈决定终端通道与时序历史。
//! 保留请求与生效结果供诊断，图编译和 GPU 执行只应消费解析后的配置。

mod fallback;
mod mode;
mod settings;
mod taa_quality;

pub use fallback::{AntiAliasFallbackReason, AntiAliasFallbackReport};
pub use mode::AntiAliasMode;
pub use settings::AntiAliasSettings;
pub use taa_quality::TaaQualityPreset;
