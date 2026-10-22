use super::HostRedrawRequest;

impl HostRedrawRequest {
    /// Drain committed damage into this present, retaining any next runtime tick.
    /// Requests needing another frame update must publish their geometry first.
    pub(crate) fn split_ready_present(self) -> (Self, Self) {
        if self.requires_frame_update() {
            return (self, Self::None);
        }
        match self {
            Self::Region {
                damage,
                runtime_tick,
                scenario,
                ..
            } => (
                if runtime_tick {
                    Self::RuntimeTick
                } else {
                    Self::None
                },
                Self::Region {
                    damage,
                    runtime_tick: false,
                    frame_update: false,
                    interactive_frame_update: false,
                    scenario,
                },
            ),
            ready @ Self::Full { .. } => (Self::None, ready),
            work => (work, Self::None),
        }
    }
}

#[cfg(test)]
#[path = "tests/ready_present.rs"]
mod tests;
