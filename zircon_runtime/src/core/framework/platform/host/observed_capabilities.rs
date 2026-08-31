#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformHostObservedCapabilities {
    event_loop: bool,
    windowing: bool,
    display_topology: bool,
}

impl PlatformHostObservedCapabilities {
    pub const fn new(event_loop: bool, windowing: bool, display_topology: bool) -> Self {
        Self {
            event_loop,
            windowing,
            display_topology,
        }
    }

    pub const fn event_loop(self) -> bool {
        self.event_loop
    }

    pub const fn windowing(self) -> bool {
        self.windowing
    }

    pub const fn display_topology(self) -> bool {
        self.display_topology
    }
}
