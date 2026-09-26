use crate::core::framework::animation::AnimationPlaybackSettings;

use super::FrameDiagnostics;

/// 动画域的只读采样结果；由运行时采集器解析动画管理器后交给编辑器诊断面板。
/// unavailable 表示管理器未能解析，不能把缺少播放设置当成“动画已关闭”。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeAnimationDiagnostics {
    pub available: bool,
    pub playback_settings: Option<AnimationPlaybackSettings>,
    pub error: Option<String>,
}

impl RuntimeAnimationDiagnostics {
    /// 管理器解析失败时保留原因，供帧域状态和面板统一展示。
    pub fn unavailable(error: impl Into<String>) -> Self {
        Self {
            available: false,
            playback_settings: None,
            error: Some(error.into()),
        }
    }
}

impl FrameDiagnostics for RuntimeAnimationDiagnostics {
    fn diagnostics_domain(&self) -> &'static str {
        "animation"
    }

    fn diagnostics_available(&self) -> bool {
        self.available
    }

    fn diagnostics_error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}
