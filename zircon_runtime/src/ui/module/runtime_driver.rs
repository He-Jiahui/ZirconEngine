use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use crate::core::{CoreError, CoreHandle, CoreResult};

use super::UI_RUNTIME_DRIVER_NAME;

/// Configure before activation; changes take effect on the next activation.
/// Core 创建 UI 驱动时读取的配置键；参见 [`UiConfig`] 与 [`UiRuntimeDriver::config`]。
pub const UI_CONFIG_KEY: &str = "ui";

/// UI 驱动每次构造时保存的启用配置。
///
/// # 使用与约束
/// Core 激活创建驱动时读取此值；缺少整份配置时默认启用。
/// 已有实例保留配置快照，后续 Core 配置写入由下一次激活读取。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiConfig {
    /// 准入开放时返回给项目加载方的开关；`false` 使该方跳过 UI 根加载。
    pub enabled: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Activation-scoped project admission. Surface ownership remains with its host.
/// 供 Core 一次激活使用的项目准入状态及配置快照。
///
/// # 生命周期
/// 项目加载先查询本驱动；停止清理关闭该实例的新准入，重新激活创建新实例。
/// 宿主继续拥有表面；已准入工作的排空不由本驱动完成。
#[derive(Debug)]
pub struct UiRuntimeDriver {
    config: UiConfig,
    admission_open: AtomicBool,
}

impl Default for UiRuntimeDriver {
    fn default() -> Self {
        Self::new(UiConfig::default())
    }
}

impl UiRuntimeDriver {
    fn new(config: UiConfig) -> Self {
        Self {
            config,
            admission_open: AtomicBool::new(true),
        }
    }

    /// 服务工厂创建实例时读取一次 `core` 配置；缺失时采用默认值。
    /// 反序列化失败返回 `ConfigParse`，不回退为启用；返回驱动拥有配置且不保留 Core 借用。
    pub(super) fn from_core(core: &CoreHandle) -> CoreResult<Self> {
        let config = match core.load_config_value(UI_CONFIG_KEY) {
            Some(value) => serde_json::from_value(value).map_err(|error| {
                CoreError::ConfigParse(UI_CONFIG_KEY.to_owned(), error.to_string())
            })?,
            None => UiConfig::default(),
        };
        Ok(Self::new(config))
    }

    /// 借用该驱动构造时保存的配置；引用随本次 `self` 借用有效，不重新读取 Core。
    pub fn config(&self) -> &UiConfig {
        &self.config
    }

    /// Checks admission before asset/font/layout work. Already admitted work is not drained here.
    /// 读取本实例的准入门与固定配置，供项目 UI 加载入口决定是否继续。
    ///
    /// # 返回
    /// 已关闭返回带驱动名的 `ServiceUnavailable`；开放时的 `Ok(enabled)` 保留配置值。
    /// `Ok(false)` 由项目加载方解释为跳过 UI 根，`Ok(true)` 后仍由该方执行后续工作。
    pub fn admit_project(&self) -> CoreResult<bool> {
        if !self.admission_open.load(Ordering::Acquire) {
            return Err(CoreError::ServiceUnavailable(
                UI_RUNTIME_DRIVER_NAME.to_owned(),
            ));
        }
        Ok(self.config.enabled)
    }

    // 清理回调经共享引用关闭这个实例，后续激活由新驱动重新建立准入。
    pub(super) fn close_admission(&self) {
        self.admission_open.store(false, Ordering::Release);
    }
}
