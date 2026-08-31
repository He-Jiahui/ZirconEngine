/// Neutral reason why the platform event loop entered a host pass. Adapters
/// translate native event-loop causes into this fixed vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventLoopHostWakeReason {
    Initialization,
    Poll,
    ResumeDeadline,
    WaitCancelled,
    ProxyWake,
}

impl EventLoopHostWakeReason {
    pub const COUNT: usize = 5;
    pub const ALL: [Self; Self::COUNT] = [
        Self::Initialization,
        Self::Poll,
        Self::ResumeDeadline,
        Self::WaitCancelled,
        Self::ProxyWake,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::Initialization => 0,
            Self::Poll => 1,
            Self::ResumeDeadline => 2,
            Self::WaitCancelled => 3,
            Self::ProxyWake => 4,
        }
    }
}
