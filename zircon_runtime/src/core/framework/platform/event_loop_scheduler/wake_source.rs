/// Fixed scheduler sources. Each owner coalesces its own payload and submits
/// one current deadline, keeping host selection allocation-free and bounded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventLoopWakeSource {
    FrameDemand,
    HostCommand,
    Timer,
    Proxy,
    Lifecycle,
    Input,
    Background,
}

impl EventLoopWakeSource {
    pub const COUNT: usize = 7;
    pub const ALL: [Self; Self::COUNT] = [
        Self::FrameDemand,
        Self::HostCommand,
        Self::Timer,
        Self::Proxy,
        Self::Lifecycle,
        Self::Input,
        Self::Background,
    ];

    pub const fn index(self) -> usize {
        match self {
            Self::FrameDemand => 0,
            Self::HostCommand => 1,
            Self::Timer => 2,
            Self::Proxy => 3,
            Self::Lifecycle => 4,
            Self::Input => 5,
            Self::Background => 6,
        }
    }
}
