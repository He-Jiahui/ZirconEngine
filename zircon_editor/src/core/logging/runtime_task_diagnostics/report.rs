#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct RuntimeTaskDiagnosticProjectionReport {
    observation_count: usize,
    gap_record_count: usize,
    dropped_observation_count: u64,
    has_more: bool,
}

impl RuntimeTaskDiagnosticProjectionReport {
    pub(super) const fn new(
        observation_count: usize,
        gap_record_count: usize,
        dropped_observation_count: u64,
        has_more: bool,
    ) -> Self {
        Self {
            observation_count,
            gap_record_count,
            dropped_observation_count,
            has_more,
        }
    }

    pub(crate) const fn observation_count(self) -> usize {
        self.observation_count
    }

    pub(crate) const fn gap_record_count(self) -> usize {
        self.gap_record_count
    }

    pub(crate) const fn dropped_observation_count(self) -> u64 {
        self.dropped_observation_count
    }

    pub(crate) const fn has_more(self) -> bool {
        self.has_more
    }
}
