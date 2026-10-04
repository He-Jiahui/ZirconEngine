use std::fmt::Debug;

use crate::core::framework::render::{SolariProviderAvailability, SolariRuntimeStatus};

/// Solari 插件的状态查询接口；框架在解析每帧质量配置时按已选 provider 身份生成可用性报告。
/// 默认 Ready 只表示 provider 未声明不可用，实验门槛和设备能力仍由上层检查。
pub trait SolariRuntimeProvider: Debug + Send + Sync {
    fn runtime_status(&self) -> SolariRuntimeStatus {
        SolariRuntimeStatus::Ready
    }

    fn runtime_status_message(&self) -> Option<&str> {
        None
    }

    fn availability(&self, provider_id: &str) -> SolariProviderAvailability {
        match self.runtime_status() {
            SolariRuntimeStatus::Ready => SolariProviderAvailability::ready(provider_id),
            SolariRuntimeStatus::Unavailable => SolariProviderAvailability::unavailable(
                provider_id,
                self.runtime_status_message()
                    .unwrap_or("Solari provider is unavailable"),
            ),
            _ => SolariProviderAvailability::unavailable(
                provider_id,
                self.runtime_status_message()
                    .unwrap_or("Solari provider did not report a ready runtime status"),
            ),
        }
    }
}
