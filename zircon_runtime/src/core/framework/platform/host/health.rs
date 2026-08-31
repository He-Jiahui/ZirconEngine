#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformHostHealth {
    Unknown,
    Healthy,
    Degraded,
    Failed,
}
