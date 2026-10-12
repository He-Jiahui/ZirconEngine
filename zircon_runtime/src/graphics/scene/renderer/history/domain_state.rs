use crate::core::framework::render::{RenderHistoryDomainStatus, RenderHistoryDomainsReport};

pub(crate) use crate::core::framework::render::{
    RenderHistoryDomain as SceneHistoryDomain,
    RenderHistoryDomainResetReason as SceneHistoryResetReason,
};

const SPATIAL_HISTORY_DOMAINS: [SceneHistoryDomain; SceneHistoryDomain::COUNT - 1] = [
    SceneHistoryDomain::TaaSceneColor,
    SceneHistoryDomain::HybridGlobalIllumination,
    SceneHistoryDomain::AmbientOcclusion,
    SceneHistoryDomain::ScreenSpaceReflection,
    SceneHistoryDomain::HzbFurthest,
    SceneHistoryDomain::VolumetricScattering,
];

const fn domain_bit(domain: SceneHistoryDomain) -> u8 {
    1 << domain as u8
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SceneHistoryDomainState {
    generation: u64,
    valid: bool,
    last_successful_frame: Option<u64>,
    reset_reason: Option<SceneHistoryResetReason>,
}

impl Default for SceneHistoryDomainState {
    fn default() -> Self {
        Self {
            generation: 0,
            valid: false,
            last_successful_frame: None,
            reset_reason: Some(SceneHistoryResetReason::NeverProduced),
        }
    }
}

impl SceneHistoryDomainState {
    const fn generation(self) -> u64 {
        self.generation
    }

    const fn is_valid(self) -> bool {
        self.valid
    }

    const fn last_successful_frame(self) -> Option<u64> {
        self.last_successful_frame
    }

    const fn reset_reason(self) -> Option<SceneHistoryResetReason> {
        self.reset_reason
    }

    fn commit_success(&mut self, frame_generation: u64) {
        self.generation = self.generation.wrapping_add(1);
        self.valid = true;
        self.last_successful_frame = Some(frame_generation);
        self.reset_reason = None;
    }

    fn invalidate(&mut self, reason: SceneHistoryResetReason) {
        self.valid = false;
        self.reset_reason = Some(reason);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SceneHistoryAvailability {
    valid_bits: u8,
}

impl SceneHistoryAvailability {
    pub(crate) const fn is_available(self, domain: SceneHistoryDomain) -> bool {
        self.valid_bits & domain_bit(domain) != 0
    }

    fn remove(&mut self, domain: SceneHistoryDomain) {
        self.valid_bits &= !domain_bit(domain);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SceneHistoryDomainStates {
    states: [SceneHistoryDomainState; SceneHistoryDomain::COUNT],
}

impl SceneHistoryDomainStates {
    fn state(&self, domain: SceneHistoryDomain) -> SceneHistoryDomainState {
        self.states[domain.index()]
    }

    fn availability(&self) -> SceneHistoryAvailability {
        let mut valid_bits = 0;
        for domain in SceneHistoryDomain::ALL {
            if self.state(domain).is_valid() {
                valid_bits |= domain_bit(domain);
            }
        }
        SceneHistoryAvailability { valid_bits }
    }

    fn report(
        &self,
        frame_reset_reasons: [Option<SceneHistoryResetReason>; SceneHistoryDomain::COUNT],
    ) -> RenderHistoryDomainsReport {
        let mut report_states = [RenderHistoryDomainStatus::default(); SceneHistoryDomain::COUNT];
        for domain in SceneHistoryDomain::ALL {
            let state = self.state(domain);
            report_states[domain.index()] = RenderHistoryDomainStatus::new(
                state.generation(),
                state.is_valid(),
                state.last_successful_frame(),
                state.reset_reason(),
                frame_reset_reasons[domain.index()],
            );
        }
        RenderHistoryDomainsReport::new(true, report_states)
    }

    fn commit_success(&mut self, domain: SceneHistoryDomain, frame_generation: u64) {
        self.states[domain.index()].commit_success(frame_generation);
    }

    fn invalidate(&mut self, domain: SceneHistoryDomain, reason: SceneHistoryResetReason) {
        self.states[domain.index()].invalidate(reason);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SceneHistoryWriteIntent {
    requested_bits: u8,
    written_bits: u8,
}

impl SceneHistoryWriteIntent {
    pub(crate) fn record(&mut self, domain: SceneHistoryDomain, written: bool) {
        self.requested_bits |= domain_bit(domain);
        if written {
            self.written_bits |= domain_bit(domain);
        }
    }

    pub(crate) const fn was_written(self, domain: SceneHistoryDomain) -> bool {
        self.written_bits & domain_bit(domain) != 0
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.requested_bits |= other.requested_bits;
        self.written_bits |= other.written_bits;
    }

    const fn was_requested(self, domain: SceneHistoryDomain) -> bool {
        self.requested_bits & domain_bit(domain) != 0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SceneHistoryFrameTransaction {
    availability: SceneHistoryAvailability,
    reset_reasons: [Option<SceneHistoryResetReason>; SceneHistoryDomain::COUNT],
    writes: SceneHistoryWriteIntent,
}

impl SceneHistoryFrameTransaction {
    pub(crate) fn begin(states: &SceneHistoryDomainStates) -> Self {
        Self {
            availability: states.availability(),
            reset_reasons: [None; SceneHistoryDomain::COUNT],
            writes: SceneHistoryWriteIntent::default(),
        }
    }

    pub(crate) const fn unavailable() -> Self {
        Self {
            availability: SceneHistoryAvailability { valid_bits: 0 },
            reset_reasons: [None; SceneHistoryDomain::COUNT],
            writes: SceneHistoryWriteIntent {
                requested_bits: 0,
                written_bits: 0,
            },
        }
    }

    pub(crate) const fn availability(&self) -> SceneHistoryAvailability {
        self.availability
    }

    pub(crate) fn invalidate(
        &mut self,
        domain: SceneHistoryDomain,
        reason: SceneHistoryResetReason,
    ) {
        self.availability.remove(domain);
        self.reset_reasons[domain.index()] = Some(reason);
    }

    pub(crate) fn invalidate_spatial(&mut self, reason: SceneHistoryResetReason) {
        for domain in SPATIAL_HISTORY_DOMAINS {
            self.invalidate(domain, reason);
        }
    }

    pub(crate) fn invalidate_all(&mut self, reason: SceneHistoryResetReason) {
        for domain in SceneHistoryDomain::ALL {
            self.invalidate(domain, reason);
        }
    }

    pub(crate) fn absorb_writes(&mut self, writes: SceneHistoryWriteIntent) {
        self.writes.merge(writes);
    }

    pub(crate) const fn domain_was_written(&self, domain: SceneHistoryDomain) -> bool {
        self.writes.was_written(domain)
    }

    // 成功写入优先于本帧失效；请求过但未写入的域变为 SourceUnavailable，
    // 未触及的域保持原状态，提交本事务不等于 GPU 已完成。
    pub(crate) fn commit(
        self,
        states: &mut SceneHistoryDomainStates,
        frame_generation: u64,
    ) -> RenderHistoryDomainsReport {
        let mut frame_reset_reasons = [None; SceneHistoryDomain::COUNT];
        for domain in SceneHistoryDomain::ALL {
            let index = domain.index();
            if self.writes.was_written(domain) {
                frame_reset_reasons[index] = self.reset_reasons[index];
                states.commit_success(domain, frame_generation);
            } else if self.writes.was_requested(domain) {
                let reason = SceneHistoryResetReason::SourceUnavailable;
                frame_reset_reasons[index] = Some(reason);
                states.invalidate(domain, reason);
            } else if let Some(reason) = self.reset_reasons[index] {
                frame_reset_reasons[index] = Some(reason);
                states.invalidate(domain, reason);
            }
        }
        states.report(frame_reset_reasons)
    }
}

#[cfg(test)]
#[path = "tests/domain_state.rs"]
mod tests;
