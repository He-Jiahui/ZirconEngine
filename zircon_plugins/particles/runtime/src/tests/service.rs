use super::*;

mod snapshot {
    use super::*;

    #[test]
    fn diagnostic_sequence_exhaustion_never_reuses_the_last_sequence() {
        let manager = ParticlesManager::default();
        {
            let mut state = manager.lock_state();
            state.next_diagnostic_sequence = Some(u64::MAX);
            state.push_diagnostic(ParticleRuntimeDiagnostic::warning(None, "last sequence"));
            state.push_diagnostic(ParticleRuntimeDiagnostic::warning(
                None,
                "must be dropped after exhaustion",
            ));
        }

        let snapshot = manager.snapshot();
        assert_eq!(snapshot.diagnostic_sequence, u64::MAX);
        assert_eq!(snapshot.diagnostics.len(), 1);
        assert_eq!(snapshot.dropped_diagnostics, 1);

        let page = manager.diagnostics_page(u64::MAX - 1, usize::MAX);
        assert_eq!(page.entries.len(), 1);
        assert_eq!(page.entries[0].sequence, u64::MAX);
    }
}
