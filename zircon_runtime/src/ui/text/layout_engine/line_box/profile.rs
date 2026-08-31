pub(super) const ARABIC_TATWEEL_RECEIPT_COUNT_MISMATCH_CODE: usize = 14;

pub(super) struct ArabicTatweelLineProfile {
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    requested_count: usize,
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    probe_count: usize,
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    candidate_input_byte_count: usize,
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    safe_candidate_count: usize,
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    accepted_count: usize,
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    last_rejection_code: usize,
}

impl ArabicTatweelLineProfile {
    pub(super) fn new(requested_count: usize) -> Self {
        #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
        let _ = requested_count;
        Self {
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            requested_count,
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            probe_count: 0,
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            candidate_input_byte_count: 0,
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            safe_candidate_count: 0,
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            accepted_count: 0,
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            last_rejection_code: 0,
        }
    }

    pub(super) fn record_probe(&mut self, candidate_input_bytes: usize) {
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        {
            self.probe_count = self.probe_count.saturating_add(1);
            self.candidate_input_byte_count = self
                .candidate_input_byte_count
                .saturating_add(candidate_input_bytes);
        }
        #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
        let _ = candidate_input_bytes;
    }

    pub(super) fn record_safe_candidate(&mut self) {
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        {
            self.safe_candidate_count = self.safe_candidate_count.saturating_add(1);
        }
    }

    pub(super) fn record_rejection(&mut self, rejection_code: usize) {
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        {
            self.last_rejection_code = rejection_code;
        }
        #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
        let _ = rejection_code;
    }

    pub(super) fn record_accepted(&mut self, accepted_count: usize) {
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        {
            self.accepted_count = accepted_count;
        }
        #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
        let _ = accepted_count;
    }
}

impl Drop for ArabicTatweelLineProfile {
    fn drop(&mut self) {
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        {
            let budget = super::arabic_tatweel_budget_snapshot();
            crate::profile_counter!(
                "runtime",
                "text.runtime_budget.arabic_tatweels_per_line",
                budget.max_materialized_tatweels_per_line
            );
            crate::profile_counter!(
                "runtime",
                "text.runtime_budget.arabic_tatweel_fit_measurements_per_line",
                budget.max_fit_measurements_per_line
            );
            crate::profile_counter!(
                "runtime",
                "arabic_tatweel_requested_count",
                self.requested_count
            );
            crate::profile_counter!("runtime", "arabic_tatweel_probe_count", self.probe_count);
            crate::profile_counter!(
                "runtime",
                "arabic_tatweel_candidate_input_byte_count",
                self.candidate_input_byte_count
            );
            crate::profile_counter!(
                "runtime",
                "arabic_tatweel_safe_candidate_count",
                self.safe_candidate_count
            );
            crate::profile_counter!(
                "runtime",
                "arabic_tatweel_accepted_count",
                self.accepted_count
            );
            crate::profile_counter!(
                "runtime",
                "arabic_tatweel_last_rejection_code",
                self.last_rejection_code
            );
        }
    }
}
