use woc_protocol::{fnv1a_bytes, FNV1A_OFFSET};
use woc_runtime::{
    CommittedSnapshot, RuntimeRole, RuntimeStatus, TickBudgets, VmReloadStage, VmTickError,
    VmTickResult, WocProjectVm, WocReloadableVm, WocTransactionalRuntime,
};

struct ReloadVm {
    name: &'static str,
    schema: &'static str,
    state: Vec<u8>,
    active: bool,
    fail_activate: bool,
    fail_restore: bool,
}

impl ReloadVm {
    fn old() -> Self {
        Self {
            name: "old",
            schema: "schema/1",
            state: b"old-state".to_vec(),
            active: true,
            fail_activate: false,
            fail_restore: false,
        }
    }

    fn replacement() -> Self {
        Self {
            name: "new",
            schema: "schema/2",
            state: Vec::new(),
            active: false,
            fail_activate: false,
            fail_restore: false,
        }
    }
}

impl WocProjectVm for ReloadVm {
    fn fixed_tick(
        &mut self,
        _input_payload: &[u8],
        _budgets: TickBudgets,
    ) -> Result<VmTickResult, VmTickError> {
        Err(VmTickError::Transport(
            "hot reload fixture does not tick".to_string(),
        ))
    }
}

impl WocReloadableVm for ReloadVm {
    fn state_schema(&self) -> Result<String, VmTickError> {
        Ok(self.schema.to_string())
    }

    fn save_state(&mut self) -> Result<Vec<u8>, VmTickError> {
        Ok(self.state.clone())
    }

    fn deactivate(&mut self) -> Result<(), VmTickError> {
        self.active = false;
        Ok(())
    }

    fn activate(&mut self) -> Result<(), VmTickError> {
        if self.fail_activate {
            return Err(VmTickError::Trap("activate failed".to_string()));
        }
        self.active = true;
        Ok(())
    }

    fn restore_state(&mut self, state: &[u8]) -> Result<(), VmTickError> {
        if self.fail_restore {
            return Err(VmTickError::Trap("restore failed".to_string()));
        }
        self.state = state.to_vec();
        Ok(())
    }
}

#[test]
fn hot_reload_migrates_between_schemas_and_commits_generation_at_tick_boundary() {
    let mut runtime = WocTransactionalRuntime::new(
        RuntimeRole::Offline,
        ReloadVm::old(),
        TickBudgets::default(),
    );
    let old_projection = b"old-generation-projection".to_vec();
    runtime
        .install_full_snapshot(CommittedSnapshot {
            generation: 0,
            tick: 7,
            state: Vec::new(),
            state_digest: FNV1A_OFFSET,
            event_digest: FNV1A_OFFSET,
            presentation_digest: fnv1a_bytes(&old_projection),
            presentation_payload: old_projection,
        })
        .expect("install old generation projection");

    let generation = runtime
        .hot_reload(ReloadVm::replacement(), |old_schema, new_schema, state| {
            assert_eq!(old_schema, "schema/1");
            assert_eq!(new_schema, "schema/2");
            let mut migrated = state.to_vec();
            migrated.extend_from_slice(b"-migrated");
            Ok(migrated)
        })
        .expect("reload must commit");

    assert_eq!(generation, 1);
    assert_eq!(runtime.committed().generation, 1);
    assert_eq!(runtime.vm().name, "new");
    assert!(runtime.vm().active);
    assert_eq!(runtime.vm().state, b"old-state-migrated");
    assert!(runtime.committed().presentation_payload.is_empty());
    assert_eq!(runtime.committed().presentation_digest, FNV1A_OFFSET);
    assert_eq!(runtime.status(), &RuntimeStatus::Running);
}

#[test]
fn failed_new_generation_reactivates_old_vm_and_rolls_back_generation() {
    for stage in [VmReloadStage::Activate, VmReloadStage::Restore] {
        let mut replacement = ReloadVm::replacement();
        replacement.fail_activate = stage == VmReloadStage::Activate;
        replacement.fail_restore = stage == VmReloadStage::Restore;
        let mut runtime = WocTransactionalRuntime::new(
            RuntimeRole::Offline,
            ReloadVm::old(),
            TickBudgets::default(),
        );

        let error = runtime
            .hot_reload(replacement, |_old, _new, state| Ok(state.to_vec()))
            .expect_err("injected reload failure must roll back");

        assert_eq!(error.stage, stage);
        assert!(error.rollback_error.is_none());
        assert_eq!(runtime.committed().generation, 0);
        assert_eq!(runtime.vm().name, "old");
        assert!(runtime.vm().active);
        assert_eq!(runtime.vm().state, b"old-state");
        assert_eq!(runtime.status(), &RuntimeStatus::Running);
    }
}
