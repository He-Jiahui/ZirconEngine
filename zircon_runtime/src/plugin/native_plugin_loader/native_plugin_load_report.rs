//! 原生插件发现与加载结果的拥有者：先收集候选、已加载实例和诊断，再按需生成供运行时与编辑器共用的只读投影。
//! 所有集合变更都经过本模块，使缓存投影始终对应同一代报告。

mod diagnostics;
mod manifests;
mod projection;
mod registrations;

pub use projection::NativePluginLoadProjection;

#[cfg(test)]
#[path = "native_plugin_load_report/tests/cases.rs"]
mod tests;

use std::sync::OnceLock;

use super::{LoadedNativePlugin, NativePluginCandidate};

/// 一次发现或加载操作的原始结果；调用方可读取候选和错误，派生清单通过 `projection` 延迟生成。
#[derive(Default)]
pub struct NativePluginLoadReport {
    discovered: Vec<NativePluginCandidate>,
    loaded: Vec<LoadedNativePlugin>,
    diagnostics: Vec<String>,
    projection: OnceLock<NativePluginLoadProjection>,
}

impl std::fmt::Debug for NativePluginLoadReport {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativePluginLoadReport")
            .field("discovered", &self.discovered)
            .field("loaded", &self.loaded)
            .field("diagnostics", &self.diagnostics)
            .finish_non_exhaustive()
    }
}

impl NativePluginLoadReport {
    pub(in crate::plugin::native_plugin_loader) fn diagnostic_only(
        diagnostic: impl Into<String>,
    ) -> Self {
        Self {
            diagnostics: vec![diagnostic.into()],
            ..Self::default()
        }
    }

    pub(in crate::plugin::native_plugin_loader) fn from_discovered(
        discovered: Vec<NativePluginCandidate>,
    ) -> Self {
        Self {
            discovered,
            ..Self::default()
        }
    }

    pub(in crate::plugin::native_plugin_loader) fn from_discovery(
        discovered: Vec<NativePluginCandidate>,
        diagnostics: Vec<String>,
    ) -> Self {
        Self {
            discovered,
            diagnostics,
            ..Self::default()
        }
    }

    pub(in crate::plugin::native_plugin_loader) fn from_loaded(
        loaded: Vec<LoadedNativePlugin>,
    ) -> Self {
        Self {
            loaded,
            ..Self::default()
        }
    }

    /// 加载器临时取走候选进行 ABI 检查，随后须用 `restore_discovered` 放回，供清单与编辑器继续投影。
    pub(in crate::plugin::native_plugin_loader) fn take_discovered(
        &mut self,
    ) -> Vec<NativePluginCandidate> {
        self.invalidate_projection();
        std::mem::take(&mut self.discovered)
    }

    /// 热更新只接受纯发现报告；已经包含加载结果时返还完整报告，避免误丢失诊断与实例。
    pub(in crate::plugin::native_plugin_loader) fn try_into_discovered(
        self,
    ) -> Result<Vec<NativePluginCandidate>, Self> {
        if self.loaded.is_empty() {
            Ok(self.discovered)
        } else {
            Err(self)
        }
    }

    /// 在加载尝试后恢复候选，使失败实例仍保留可供状态展示和后续重试的发现上下文。
    pub(in crate::plugin::native_plugin_loader) fn restore_discovered(
        &mut self,
        discovered: Vec<NativePluginCandidate>,
    ) {
        self.invalidate_projection();
        self.discovered = discovered;
    }

    /// 将已加载实例的所有权移交给 live host；调用前建立的投影随之失效。
    pub(in crate::plugin::native_plugin_loader) fn take_loaded(
        &mut self,
    ) -> Vec<LoadedNativePlugin> {
        self.invalidate_projection();
        std::mem::take(&mut self.loaded)
    }

    pub(in crate::plugin::native_plugin_loader) fn push_loaded(
        &mut self,
        loaded: LoadedNativePlugin,
    ) {
        self.invalidate_projection();
        self.loaded.push(loaded);
    }

    pub(in crate::plugin::native_plugin_loader) fn push_diagnostic(
        &mut self,
        diagnostic: impl Into<String>,
    ) {
        self.invalidate_projection();
        self.diagnostics.push(diagnostic.into());
    }

    pub fn discovered(&self) -> &[NativePluginCandidate] {
        &self.discovered
    }

    pub fn loaded(&self) -> &[LoadedNativePlugin] {
        &self.loaded
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub(crate) fn into_loaded(self) -> Vec<LoadedNativePlugin> {
        self.loaded
    }

    /// 仅概括发现和加载阶段的原始诊断；描述符、入口及 shader 注册诊断由各自投影另行提供。
    pub fn has_failures(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    /// Freezes the derived manifest and diagnostic indexes on first use. Report mutations must
    /// use this owner's controlled APIs so a later mutation invalidates the frozen generation.
    pub fn projection(&self) -> &NativePluginLoadProjection {
        self.projection
            .get_or_init(|| NativePluginLoadProjection::new(self))
    }

    fn invalidate_projection(&mut self) {
        self.projection.take();
    }
}
