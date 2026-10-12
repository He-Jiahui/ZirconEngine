use super::HostRedrawRequest;

impl HostRedrawRequest {
    pub(crate) fn merge(self, next: Self) -> Self {
        let interactive_frame_update = self.prefers_interactive_frame_update();
        let next_interactive_frame_update = next.prefers_interactive_frame_update();
        let merged = match (self, next) {
            (
                Self::FrameUpdate { .. },
                Self::FrameUpdate {
                    scenario: next_scenario,
                    ..
                },
            ) => Self::FrameUpdate {
                scenario: next_scenario,
                interactive_frame_update: false,
            },
            (
                Self::FrameUpdate { scenario, .. },
                Self::Full {
                    frame_update,
                    scenario: next_scenario,
                    ..
                },
            ) => Self::Full {
                frame_update: true,
                interactive_frame_update: false,
                scenario: if frame_update {
                    next_scenario
                } else {
                    scenario
                },
            },
            (
                Self::FrameUpdate { scenario, .. },
                Self::Region {
                    damage,
                    frame_update,
                    scenario: next_scenario,
                    ..
                },
            ) => Self::Region {
                damage,
                frame_update: true,
                interactive_frame_update: false,
                scenario: if frame_update {
                    next_scenario
                } else {
                    scenario
                },
            },
            (Self::Full { .. }, Self::FrameUpdate { scenario, .. }) => Self::Full {
                frame_update: true,
                interactive_frame_update: false,
                scenario,
            },
            (Self::Region { damage, .. }, Self::FrameUpdate { scenario, .. }) => Self::Region {
                damage,
                frame_update: true,
                interactive_frame_update: false,
                scenario,
            },
            (
                Self::Full {
                    frame_update,
                    scenario,
                    ..
                },
                Self::Full {
                    frame_update: next,
                    scenario: next_scenario,
                    ..
                },
            ) => Self::Full {
                frame_update: frame_update || next,
                interactive_frame_update: false,
                scenario: if next { next_scenario } else { scenario },
            },
            (
                Self::Full {
                    frame_update,
                    scenario,
                    ..
                },
                Self::Region {
                    frame_update: next,
                    scenario: next_scenario,
                    ..
                },
            ) => Self::Full {
                frame_update: frame_update || next,
                interactive_frame_update: false,
                scenario: if next { next_scenario } else { scenario },
            },
            (
                Self::Region { frame_update, .. },
                Self::Full {
                    frame_update: next,
                    scenario,
                    ..
                },
            ) => Self::Full {
                frame_update: frame_update || next,
                interactive_frame_update: false,
                scenario,
            },
            (
                Self::Region {
                    damage: current,
                    frame_update,
                    scenario,
                    ..
                },
                Self::Region {
                    damage: next,
                    frame_update: next_update,
                    scenario: next_scenario,
                    ..
                },
            ) => Self::Region {
                damage: current.merge(next),
                frame_update: frame_update || next_update,
                interactive_frame_update: false,
                scenario: if next_update { next_scenario } else { scenario },
            },
            (Self::None, next @ Self::Full { .. }) => next,
            (Self::None, next @ Self::Region { .. }) => next,
            (Self::None, next @ Self::FrameUpdate { .. }) => next,
            (current, Self::None) => current,
        };
        if interactive_frame_update || next_interactive_frame_update {
            merged.into_interactive_frame_update()
        } else {
            merged
        }
    }
}
