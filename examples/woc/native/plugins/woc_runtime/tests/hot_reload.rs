use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use woc_protocol::{event_stream_digest, fnv1a_bytes, FixedTickInput, WorldSnapshot, FNV1A_OFFSET};
use woc_runtime::{
    CommittedSnapshot, RuntimeRole, RuntimeStatus, TickBudgets, TickUsage, VmReloadStage,
    VmTickError, VmTickResult, WocProjectVm, WocReloadableVm, WocTransactionalRuntime,
};

struct ReloadVm {
    name: &'static str,
    schema: &'static str,
    state: Vec<u8>,
    active: bool,
    fail_activate: bool,
    fail_restore: bool,
    fail_deactivate: bool,
    fail_save: bool,
    fail_checkpoint_rollback: bool,
    hidden_mutations: u64,
    replacement_cleanup_calls: Arc<AtomicUsize>,
    observed_inputs: Vec<FixedTickInput>,
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
            fail_deactivate: false,
            fail_save: false,
            fail_checkpoint_rollback: false,
            hidden_mutations: 0,
            replacement_cleanup_calls: Arc::new(AtomicUsize::new(0)),
            observed_inputs: Vec::new(),
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
            fail_deactivate: false,
            fail_save: false,
            fail_checkpoint_rollback: false,
            hidden_mutations: 0,
            replacement_cleanup_calls: Arc::new(AtomicUsize::new(0)),
            observed_inputs: Vec::new(),
        }
    }
}

impl WocProjectVm for ReloadVm {
    type Checkpoint = Vec<u8>;

    fn checkpoint(&mut self) -> Result<Vec<u8>, VmTickError> {
        let mut checkpoint = (self.state.len() as u64).to_le_bytes().to_vec();
        checkpoint.extend_from_slice(&self.state);
        checkpoint.push(self.active as u8);
        checkpoint.extend_from_slice(&self.hidden_mutations.to_le_bytes());
        checkpoint.extend_from_slice(&(self.observed_inputs.len() as u64).to_le_bytes());
        Ok(checkpoint)
    }

    fn rollback(&mut self, checkpoint: &Self::Checkpoint) -> Result<(), VmTickError> {
        if self.fail_checkpoint_rollback {
            return Err(VmTickError::Trap("checkpoint rollback failed".to_string()));
        }
        let state_len = checkpoint
            .get(..8)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u64::from_le_bytes)
            .and_then(|length| usize::try_from(length).ok())
            .ok_or_else(|| VmTickError::Transport("invalid checkpoint length".to_string()))?;
        let state_end = 8_usize.saturating_add(state_len);
        self.state = checkpoint
            .get(8..state_end)
            .ok_or_else(|| VmTickError::Transport("invalid checkpoint state".to_string()))?
            .to_vec();
        self.active = checkpoint.get(state_end).copied() == Some(1);
        self.hidden_mutations = checkpoint
            .get(state_end.saturating_add(1)..state_end.saturating_add(9))
            .and_then(|bytes| bytes.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or_else(|| VmTickError::Transport("invalid checkpoint metadata".to_string()))?;
        let input_count = checkpoint
            .get(state_end.saturating_add(9)..state_end.saturating_add(17))
            .and_then(|bytes| bytes.try_into().ok())
            .map(u64::from_le_bytes)
            .and_then(|count| usize::try_from(count).ok())
            .ok_or_else(|| VmTickError::Transport("invalid checkpoint inputs".to_string()))?;
        self.observed_inputs.truncate(input_count);
        Ok(())
    }

    fn install_full_snapshot(&mut self, snapshot: &CommittedSnapshot) -> Result<(), VmTickError> {
        self.state = snapshot.state.clone();
        Ok(())
    }

    fn fixed_tick(
        &mut self,
        input_payload: &[u8],
        _budgets: TickBudgets,
    ) -> Result<VmTickResult, VmTickError> {
        let input = FixedTickInput::decode_payload(input_payload)
            .map_err(|error| VmTickError::Transport(format!("invalid fixed input: {error:?}")))?;
        self.observed_inputs.push(input.clone());
        let mut state = input.committed_state.clone();
        state.extend_from_slice(b"-next");
        self.state = state.clone();
        let output = WorldSnapshot {
            tick: input.tick,
            state_digest: fnv1a_bytes(&state),
            event_digest: event_stream_digest(&[]),
            state,
            events: Vec::new(),
        };
        Ok(VmTickResult {
            output_payload: output
                .encode_payload()
                .map_err(|error| VmTickError::Transport(format!("encode output: {error:?}")))?,
            presentation_payload: Vec::new(),
            usage: TickUsage::default(),
        })
    }
}

impl WocReloadableVm for ReloadVm {
    fn state_schema(&self) -> Result<String, VmTickError> {
        Ok(self.schema.to_string())
    }

    fn save_state(&mut self) -> Result<Vec<u8>, VmTickError> {
        self.hidden_mutations += 1;
        if self.fail_save {
            return Err(VmTickError::Trap("save failed".to_string()));
        }
        Ok(self.state.clone())
    }

    fn deactivate(&mut self) -> Result<(), VmTickError> {
        self.hidden_mutations += 1;
        if self.name == "new" {
            self.replacement_cleanup_calls
                .fetch_add(1, Ordering::SeqCst);
        }
        self.active = false;
        if self.fail_deactivate {
            return Err(VmTickError::Trap("deactivate failed".to_string()));
        }
        Ok(())
    }

    fn activate(&mut self) -> Result<(), VmTickError> {
        self.hidden_mutations += 1;
        self.active = true;
        if self.fail_activate {
            return Err(VmTickError::Trap("activate failed".to_string()));
        }
        Ok(())
    }

    fn restore_state(&mut self, state: &[u8]) -> Result<(), VmTickError> {
        self.hidden_mutations += 1;
        if self.fail_restore {
            return Err(VmTickError::Trap("restore failed".to_string()));
        }
        self.state = state.to_vec();
        Ok(())
    }
}

fn install_old_snapshot(runtime: &mut WocTransactionalRuntime<ReloadVm>) {
    let old_projection = b"old-generation-projection".to_vec();
    runtime
        .install_full_snapshot(CommittedSnapshot {
            generation: 0,
            tick: 7,
            state: b"old-state".to_vec(),
            state_digest: fnv1a_bytes(b"old-state"),
            event_digest: FNV1A_OFFSET,
            presentation_digest: fnv1a_bytes(&old_projection),
            presentation_payload: old_projection,
        })
        .expect("install old generation projection");
}

#[test]
fn hot_reload_migrates_between_schemas_and_commits_generation_at_tick_boundary() {
    let mut runtime = WocTransactionalRuntime::new(
        RuntimeRole::Offline,
        ReloadVm::old(),
        TickBudgets::default(),
    );
    install_old_snapshot(&mut runtime);

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
    assert_eq!(runtime.committed().state, b"old-state-migrated");
    assert_eq!(
        runtime.committed().state_digest,
        fnv1a_bytes(b"old-state-migrated")
    );
    assert!(runtime.committed().presentation_payload.is_empty());
    assert_eq!(runtime.committed().presentation_digest, FNV1A_OFFSET);
    assert_eq!(runtime.status(), &RuntimeStatus::Running);

    runtime.tick(vec![]).expect("next generation tick");
    assert_eq!(runtime.vm().observed_inputs.len(), 1);
    assert_eq!(
        runtime.vm().observed_inputs[0].committed_state,
        b"old-state-migrated"
    );
    assert_eq!(runtime.vm().observed_inputs[0].generation, 1);
    assert_eq!(
        runtime.vm().observed_inputs[0].committed_state_digest,
        fnv1a_bytes(b"old-state-migrated")
    );
    assert_eq!(runtime.committed().tick, 8);
    assert_eq!(runtime.committed().state, b"old-state-migrated-next");
}

#[test]
fn migration_failure_restores_saved_vm_state_and_preserves_committed_wrapper() {
    let replacement = ReloadVm::replacement();
    let cleanup_calls = replacement.replacement_cleanup_calls.clone();
    let mut runtime = WocTransactionalRuntime::new(
        RuntimeRole::Offline,
        ReloadVm::old(),
        TickBudgets::default(),
    );
    install_old_snapshot(&mut runtime);
    let before = runtime.committed().clone();

    let error = runtime
        .hot_reload(replacement, |_old, _new, _state| {
            Err(VmTickError::Limited("migration rejected".to_string()))
        })
        .expect_err("migration failure must restore the old generation");

    assert_eq!(error.stage, VmReloadStage::Migrate);
    assert!(matches!(
        error.source,
        VmTickError::Limited(ref reason) if reason == "migration rejected"
    ));
    assert!(error.rollback_error.is_none());
    assert!(error.replacement_cleanup_error.is_none());
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 0);
    assert_eq!(runtime.committed(), &before);
    assert_eq!(runtime.vm().name, "old");
    assert!(runtime.vm().active);
    assert_eq!(runtime.vm().state, b"old-state");
    assert_eq!(runtime.vm().hidden_mutations, 0);
    assert_eq!(runtime.status(), &RuntimeStatus::Running);
}

#[test]
fn save_failure_restores_hidden_vm_state_from_the_opaque_checkpoint() {
    let mut old = ReloadVm::old();
    old.fail_save = true;
    let mut runtime =
        WocTransactionalRuntime::new(RuntimeRole::Offline, old, TickBudgets::default());
    install_old_snapshot(&mut runtime);
    let before = runtime.committed().clone();

    let error = runtime
        .hot_reload(ReloadVm::replacement(), |_old, _new, state| {
            Ok(state.to_vec())
        })
        .expect_err("save failure must restore its hidden mutation");

    assert_eq!(error.stage, VmReloadStage::Save);
    assert!(matches!(
        error.source,
        VmTickError::Trap(ref reason) if reason == "save failed"
    ));
    assert!(error.rollback_error.is_none());
    assert!(error.checkpoint_rollback_error.is_none());
    assert_eq!(runtime.committed(), &before);
    assert_eq!(runtime.vm().state, b"old-state");
    assert!(runtime.vm().active);
    assert_eq!(runtime.vm().hidden_mutations, 0);
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
        install_old_snapshot(&mut runtime);
        let before = runtime.committed().clone();
        let cleanup_calls = replacement.replacement_cleanup_calls.clone();

        let error = runtime
            .hot_reload(replacement, |_old, _new, state| Ok(state.to_vec()))
            .expect_err("injected reload failure must roll back");

        assert_eq!(error.stage, stage);
        assert!(error.rollback_error.is_none());
        assert!(error.replacement_cleanup_error.is_none());
        assert_eq!(cleanup_calls.load(Ordering::SeqCst), 1);
        assert_eq!(runtime.committed(), &before);
        assert_eq!(runtime.vm().name, "old");
        assert!(runtime.vm().active);
        assert_eq!(runtime.vm().state, b"old-state");
        assert_eq!(runtime.vm().hidden_mutations, 0);
        assert_eq!(runtime.status(), &RuntimeStatus::Running);
    }
}

#[test]
fn activation_failure_reports_replacement_cleanup_failure_and_restores_old_runtime() {
    let mut replacement = ReloadVm::replacement();
    replacement.fail_activate = true;
    replacement.fail_deactivate = true;
    let cleanup_calls = replacement.replacement_cleanup_calls.clone();
    let mut runtime = WocTransactionalRuntime::new(
        RuntimeRole::Offline,
        ReloadVm::old(),
        TickBudgets::default(),
    );
    install_old_snapshot(&mut runtime);
    let before = runtime.committed().clone();

    let error = runtime
        .hot_reload(replacement, |_old, _new, state| Ok(state.to_vec()))
        .expect_err("partial activation must clean up and roll back");

    assert_eq!(error.stage, VmReloadStage::Activate);
    assert!(matches!(
        error.source,
        VmTickError::Trap(ref reason) if reason == "activate failed"
    ));
    assert!(matches!(
        error.replacement_cleanup_error,
        Some(VmTickError::Trap(ref reason)) if reason == "deactivate failed"
    ));
    assert!(error.rollback_error.is_none());
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 1);
    assert_eq!(runtime.committed(), &before);
    assert_eq!(runtime.vm().name, "old");
    assert!(runtime.vm().active);
    assert_eq!(runtime.vm().state, b"old-state");
    assert_eq!(runtime.vm().hidden_mutations, 0);
    assert!(matches!(
        runtime.status(),
        RuntimeStatus::Paused(fault)
            if matches!(
                fault.kind,
                woc_runtime::WocTickFaultKind::Vm(VmTickError::Trap(ref reason))
                    if reason == "activate failed"
            ) && matches!(
                fault.rollback_error,
                Some(VmTickError::Trap(ref reason)) if reason == "deactivate failed"
            )
    ));
}

#[test]
fn old_canonical_restore_failure_is_reported_and_runtime_does_not_resume() {
    let mut old = ReloadVm::old();
    old.fail_restore = true;
    let mut replacement = ReloadVm::replacement();
    replacement.fail_activate = true;
    let mut runtime =
        WocTransactionalRuntime::new(RuntimeRole::Offline, old, TickBudgets::default());
    install_old_snapshot(&mut runtime);
    let before = runtime.committed().clone();

    let error = runtime
        .hot_reload(replacement, |_old, _new, state| Ok(state.to_vec()))
        .expect_err("failed old restore must keep the runtime stopped");

    assert_eq!(error.stage, VmReloadStage::Activate);
    assert!(matches!(
        error.rollback_error,
        Some(VmTickError::Trap(ref reason)) if reason == "restore failed"
    ));
    assert!(error.checkpoint_rollback_error.is_none());
    assert_eq!(runtime.committed(), &before);
    assert_eq!(runtime.vm().name, "old");
    assert!(runtime.vm().active);
    assert_eq!(runtime.vm().hidden_mutations, 0);
    assert!(matches!(
        runtime.status(),
        RuntimeStatus::Paused(fault)
            if matches!(
                fault.kind,
                woc_runtime::WocTickFaultKind::Vm(VmTickError::Trap(ref reason))
                    if reason == "activate failed"
            ) && matches!(
                fault.rollback_error,
                Some(VmTickError::Trap(ref reason)) if reason == "restore failed"
            )
    ));
}

#[test]
fn old_activate_failure_is_reported_and_runtime_does_not_resume() {
    let mut old = ReloadVm::old();
    old.fail_activate = true;
    let mut replacement = ReloadVm::replacement();
    replacement.fail_restore = true;
    let mut runtime =
        WocTransactionalRuntime::new(RuntimeRole::Offline, old, TickBudgets::default());
    install_old_snapshot(&mut runtime);
    let before = runtime.committed().clone();

    let error = runtime
        .hot_reload(replacement, |_old, _new, state| Ok(state.to_vec()))
        .expect_err("failed old activation must keep the runtime stopped");

    assert_eq!(error.stage, VmReloadStage::Restore);
    assert!(matches!(
        error.rollback_error,
        Some(VmTickError::Trap(ref reason)) if reason == "activate failed"
    ));
    assert!(error.checkpoint_rollback_error.is_none());
    assert_eq!(runtime.committed(), &before);
    assert_eq!(runtime.vm().name, "old");
    assert!(runtime.vm().active);
    assert_eq!(runtime.vm().hidden_mutations, 0);
    assert!(matches!(runtime.status(), RuntimeStatus::Paused(_)));
}

#[test]
fn mutating_old_deactivation_failure_is_fully_restored() {
    let mut old = ReloadVm::old();
    old.fail_deactivate = true;
    let mut runtime =
        WocTransactionalRuntime::new(RuntimeRole::Offline, old, TickBudgets::default());
    install_old_snapshot(&mut runtime);
    let before = runtime.committed().clone();

    let error = runtime
        .hot_reload(ReloadVm::replacement(), |_old, _new, state| {
            Ok(state.to_vec())
        })
        .expect_err("old deactivation failure must roll back");

    assert_eq!(error.stage, VmReloadStage::Deactivate);
    assert!(matches!(
        error.source,
        VmTickError::Trap(ref reason) if reason == "deactivate failed"
    ));
    assert!(error.rollback_error.is_none());
    assert!(error.checkpoint_rollback_error.is_none());
    assert!(error.replacement_cleanup_error.is_none());
    assert_eq!(runtime.committed(), &before);
    assert_eq!(runtime.vm().name, "old");
    assert!(runtime.vm().active);
    assert_eq!(runtime.vm().hidden_mutations, 0);
    assert_eq!(runtime.status(), &RuntimeStatus::Running);
}

#[test]
fn failed_opaque_checkpoint_rollback_is_reported_and_runtime_does_not_resume() {
    let mut old = ReloadVm::old();
    old.fail_checkpoint_rollback = true;
    let mut runtime =
        WocTransactionalRuntime::new(RuntimeRole::Offline, old, TickBudgets::default());
    install_old_snapshot(&mut runtime);
    let before = runtime.committed().clone();

    let error = runtime
        .hot_reload(ReloadVm::replacement(), |_old, _new, _state| {
            Err(VmTickError::Limited("migration rejected".to_string()))
        })
        .expect_err("failed checkpoint rollback must keep the runtime stopped");

    assert_eq!(error.stage, VmReloadStage::Migrate);
    assert!(matches!(
        error.checkpoint_rollback_error,
        Some(VmTickError::Trap(ref reason)) if reason == "checkpoint rollback failed"
    ));
    assert_eq!(runtime.committed(), &before);
    assert!(matches!(
        runtime.status(),
        RuntimeStatus::Paused(fault)
            if matches!(
                fault.kind,
                woc_runtime::WocTickFaultKind::Vm(VmTickError::Limited(ref reason))
                    if reason == "migration rejected"
            ) && matches!(
                fault.rollback_error,
                Some(VmTickError::Trap(ref reason))
                    if reason == "checkpoint rollback failed"
            )
    ));
}

#[test]
fn generation_exhaustion_rejects_reload_before_either_vm_is_mutated() {
    let replacement = ReloadVm::replacement();
    let cleanup_calls = replacement.replacement_cleanup_calls.clone();
    let mut runtime = WocTransactionalRuntime::new(
        RuntimeRole::Offline,
        ReloadVm::old(),
        TickBudgets::default(),
    );
    runtime
        .install_full_snapshot(CommittedSnapshot {
            generation: u64::MAX,
            tick: 7,
            state: b"old-state".to_vec(),
            state_digest: fnv1a_bytes(b"old-state"),
            event_digest: FNV1A_OFFSET,
            presentation_digest: FNV1A_OFFSET,
            presentation_payload: Vec::new(),
        })
        .expect("install exhausted generation");
    let before = runtime.committed().clone();

    let error = runtime
        .hot_reload(replacement, |_old, _new, state| Ok(state.to_vec()))
        .expect_err("generation exhaustion must reject before mutation");

    assert_eq!(error.stage, VmReloadStage::Generation);
    assert!(matches!(
        error.source,
        VmTickError::Limited(ref reason) if reason == "generation exhausted"
    ));
    assert_eq!(runtime.committed(), &before);
    assert_eq!(runtime.vm().hidden_mutations, 0);
    assert_eq!(cleanup_calls.load(Ordering::SeqCst), 0);
    assert_eq!(runtime.status(), &RuntimeStatus::Running);
}
