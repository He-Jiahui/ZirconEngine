use crate::core::framework::render::RenderStats;

use super::FrameDiagnostics;

/// 渲染框架解析后的只读采样结果；available 表示服务可解析，stats 仍可能因查询失败为空。
/// 编辑器应分别处理服务缺失和统计数据暂不可用，不能仅凭 available 读取 stats。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeRenderDiagnostics {
    pub available: bool,
    pub stats: Option<RenderStats>,
    pub virtual_geometry_debug_available: bool,
    pub error: Option<String>,
}

impl RuntimeRenderDiagnostics {
    /// 渲染服务无法解析时统一保留错误并清空依赖该服务的采样值。
    pub fn unavailable(error: impl Into<String>) -> Self {
        Self {
            available: false,
            stats: None,
            virtual_geometry_debug_available: false,
            error: Some(error.into()),
        }
    }
}

impl FrameDiagnostics for RuntimeRenderDiagnostics {
    fn diagnostics_domain(&self) -> &'static str {
        "render"
    }

    fn diagnostics_available(&self) -> bool {
        self.available
    }

    fn diagnostics_error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}
