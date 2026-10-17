use super::RetainedEditorHost;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn drain_initial_asset_refresh_events(&mut self) {
        // Bootstrap loads snapshots and the default scene; queued events from
        // that work should not replay as a second full presentation rebuild.
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "startup_drain_initial_asset_refresh_events"
        );
        let bootstrap_asset_count = self.asset_change_events.len();
        let mut asset_count = 0usize;
        for _ in 0..bootstrap_asset_count {
            use zircon_runtime::asset::watch::AssetEventTryRecvError;
            match self.asset_change_events.try_recv() {
                Ok(_) => asset_count += 1,
                Err(AssetEventTryRecvError::Lagged(_)) => {
                    let _snapshot = self.asset_change_events.resync();
                    self.asset_refresh_accumulator
                        .request_asset_reconciliation(std::time::Instant::now());
                    break;
                }
                Err(AssetEventTryRecvError::SequenceExhausted) => {
                    if self
                        .asset_refresh_accumulator
                        .latch_asset_sequence_exhaustion()
                    {
                        self.asset_refresh_accumulator
                            .request_asset_reconciliation(std::time::Instant::now());
                    }
                    break;
                }
                Err(AssetEventTryRecvError::Empty | AssetEventTryRecvError::Disconnected) => break,
            }
        }
        let editor_count = self.editor_asset_change_events.discard_pending();
        let bootstrap_resource_count = self.resource_change_events.len();
        let mut resource_count = 0usize;
        for _ in 0..bootstrap_resource_count {
            if self.resource_change_events.try_recv().is_err() {
                break;
            }
            resource_count += 1;
        }
        #[cfg(not(feature = "profiling"))]
        let _ = (asset_count, editor_count, resource_count);
        zircon_runtime::profile_counter!(
            "editor",
            "ui.startup.drained_asset_change_count",
            asset_count
        );
        zircon_runtime::profile_counter!(
            "editor",
            "ui.startup.drained_editor_asset_change_count",
            editor_count
        );
        zircon_runtime::profile_counter!(
            "editor",
            "ui.startup.drained_resource_change_count",
            resource_count
        );
    }
}
