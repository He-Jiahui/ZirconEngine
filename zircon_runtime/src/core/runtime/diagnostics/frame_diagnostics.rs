#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameDiagnosticsStatus<'a> {
    pub domain: &'static str,
    pub available: bool,
    pub error: Option<&'a str>,
}

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
