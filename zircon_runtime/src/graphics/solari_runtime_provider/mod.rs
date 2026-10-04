//! Solari provider 注册与可用性报告边界；质量配置在提交前读取状态，实验开关另行裁决。
mod provider;
mod provider_registration;

pub use provider::SolariRuntimeProvider;
pub use provider_registration::SolariRuntimeProviderRegistration;
