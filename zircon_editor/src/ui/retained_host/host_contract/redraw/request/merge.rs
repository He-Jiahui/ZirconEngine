use super::super::super::frame_geometry::union_frame;
use super::HostRedrawRequest;

impl HostRedrawRequest {
    pub(crate) fn merge(self, next: Self) -> Self {
        match (self, next) {
            (
                Self::FrameUpdate { .. },
                Self::FrameUpdate {
                    scenario: next_scenario,
                },
            ) => Self::FrameUpdate {
                scenario: next_scenario,
            },
            (
                Self::FrameUpdate { scenario },
                Self::Full {
                    frame_update,
                    scenario: next_scenario,
                },
            ) => Self::Full {
                frame_update: true,
                scenario: if frame_update {
                    next_scenario
                } else {
                    scenario
                },
            },
            (
                Self::FrameUpdate { scenario },
                Self::Region {
                    frame,
                    frame_update,
                    scenario: next_scenario,
                },
            ) => Self::Region {
                frame,
                frame_update: true,
                scenario: if frame_update {
                    next_scenario
                } else {
                    scenario
                },
            },
            (Self::Full { .. }, Self::FrameUpdate { scenario }) => Self::Full {
                frame_update: true,
                scenario,
            },
            (Self::Region { frame, .. }, Self::FrameUpdate { scenario }) => Self::Region {
                frame,
                frame_update: true,
                scenario,
            },
            (
                Self::Full {
                    frame_update,
                    scenario,
                },
                Self::Full {
                    frame_update: next,
                    scenario: next_scenario,
                },
            ) => Self::Full {
                frame_update: frame_update || next,
                scenario: if next { next_scenario } else { scenario },
            },
            (
                Self::Full {
                    frame_update,
                    scenario,
                },
                Self::Region {
                    frame_update: next,
                    scenario: next_scenario,
                    ..
                },
            ) => Self::Full {
                frame_update: frame_update || next,
                scenario: if next { next_scenario } else { scenario },
            },
            (
                Self::Region { frame_update, .. },
                Self::Full {
                    frame_update: next,
                    scenario,
                },
            ) => Self::Full {
                frame_update: frame_update || next,
                scenario,
            },
            (
                Self::Region {
                    frame: current,
                    frame_update,
                    scenario,
                },
                Self::Region {
                    frame: next,
                    frame_update: next_update,
                    scenario: next_scenario,
                },
            ) => Self::Region {
                frame: union_frame(&current, &next),
                frame_update: frame_update || next_update,
                scenario: if next_update { next_scenario } else { scenario },
            },
            (Self::None, next @ Self::Full { .. }) => next,
            (Self::None, next @ Self::Region { .. }) => next,
            (Self::None, next @ Self::FrameUpdate { .. }) => next,
            (current, Self::None) => current,
        }
    }
}
