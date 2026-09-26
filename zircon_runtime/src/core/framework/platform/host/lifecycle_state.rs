/// 宿主控制面状态；只有 Ready 可使已观测的能力进入运行时可用状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformHostLifecycleState {
    Uninstalled,
    Starting,
    Ready,
    Degraded,
    Quiescing,
    Quiesced,
    Failed,
    Stopped,
}
