use super::HostRedrawRequest;

impl HostRedrawRequest {
    pub(crate) fn merge(self, next: Self) -> Self {
        // Runtime demand must survive merging viewport damage without promoting
        // every animated image to a whole-workbench maintenance update.
        let runtime_tick = self.has_runtime_tick_demand() || next.has_runtime_tick_demand();
        let interactive_frame_update = self.prefers_interactive_frame_update();
        let next_interactive_frame_update = next.prefers_interactive_frame_update();
        let mut merged = match (self, next) {
            (Self::RuntimeTick, Self::None | Self::RuntimeTick)
            | (Self::None, Self::RuntimeTick) => Self::RuntimeTick,
            (Self::RuntimeTick, current) | (current, Self::RuntimeTick) => {
                match current {
                    Self::Region {
                        damage,
                        frame_update,
                        interactive_frame_update,
                        scenario,
                        ..
                    } => Self::Region {
                        damage,
                        frame_update,
                        interactive_frame_update,
                        scenario,
                        runtime_tick: true,
                    },
                    Self::Full {
                        interactive_frame_update,
                        scenario,
                        ..
                    } => Self::Full {
                        runtime_tick,
                        frame_update: true,
                        interactive_frame_update,
                        scenario,
                    },
                    other => other, // Maintenance already includes the runtime pump.
                }
            }
            (
                Self::FrameUpdate { .. },
                Self::FrameUpdate {
                    scenario: next_scenario,
                    ..
                },
            ) => Self::FrameUpdate {
                runtime_tick,
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
                runtime_tick,
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
                runtime_tick,
                frame_update: true,
                interactive_frame_update: false,
                scenario: if frame_update {
                    next_scenario
                } else {
                    scenario
                },
            },
            (Self::Full { .. }, Self::FrameUpdate { scenario, .. }) => Self::Full {
                runtime_tick,
                frame_update: true,
                interactive_frame_update: false,
                scenario,
            },
            (Self::Region { damage, .. }, Self::FrameUpdate { scenario, .. }) => Self::Region {
                damage,
                runtime_tick,
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
                runtime_tick,
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
                runtime_tick,
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
                runtime_tick,
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
                runtime_tick,
                frame_update: frame_update || next_update,
                interactive_frame_update: false,
                scenario: if next_update { next_scenario } else { scenario },
            },
            (Self::None, next @ Self::Full { .. }) => next,
            (Self::None, next @ Self::Region { .. }) => next,
            (Self::None, next @ Self::FrameUpdate { .. }) => next,
            (current, Self::None) => current,
        };
        match &mut merged {
            Self::FrameUpdate {
                runtime_tick: demand,
                ..
            }
            | Self::Full {
                runtime_tick: demand,
                ..
            }
            | Self::Region {
                runtime_tick: demand,
                ..
            } => *demand = runtime_tick,
            Self::None | Self::RuntimeTick => {}
        }
        if runtime_tick && merged.requires_frame_update() {
            // The interactive commit omits the runtime pump. When both are due,
            // use maintenance (which includes that pump) so Continuous cannot stall.
            match &mut merged {
                Self::FrameUpdate {
                    interactive_frame_update,
                    ..
                }
                | Self::Full {
                    interactive_frame_update,
                    ..
                }
                | Self::Region {
                    interactive_frame_update,
                    ..
                } => {
                    *interactive_frame_update = false;
                }
                Self::None | Self::RuntimeTick => {}
            }
            merged
        } else if interactive_frame_update || next_interactive_frame_update {
            merged.into_interactive_frame_update()
        } else {
            merged
        }
    }
}
