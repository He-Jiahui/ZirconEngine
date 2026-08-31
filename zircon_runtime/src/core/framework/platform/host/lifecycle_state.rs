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
