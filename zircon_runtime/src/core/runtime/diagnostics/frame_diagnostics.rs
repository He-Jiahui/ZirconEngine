/// 帧域服务的借用状态视图；错误文本由原快照持有，调用者不能在快照销毁后保留它。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameDiagnosticsStatus<'a> {
    pub domain: &'static str,
    pub available: bool,
    pub error: Option<&'a str>,
}

/// 让渲染、物理和动画各自提供同一面板状态契约；是否可用由采集域判定。
pub trait FrameDiagnostics {
    fn diagnostics_domain(&self) -> &'static str;

    fn diagnostics_available(&self) -> bool {
        true
    }

    fn diagnostics_error(&self) -> Option<&str> {
        None
    }

    fn frame_diagnostics_status(&self) -> FrameDiagnosticsStatus<'_> {
        FrameDiagnosticsStatus {
            domain: self.diagnostics_domain(),
            available: self.diagnostics_available(),
            error: self.diagnostics_error(),
        }
    }
}
