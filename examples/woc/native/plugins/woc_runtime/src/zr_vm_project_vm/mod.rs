mod codec;
mod exports;

use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use zr_vm_rust_binding::{
    CallUsage, CancellationToken, NativeModule, NativeModuleRegistration, ProjectSession,
    ProjectSessionCheckpoint, ProjectWorkspace, RunOptions, Runtime, RuntimeBuilder,
    RuntimeOptions,
};

use crate::{
    ClientPresentationProjection, CommittedSnapshot, TickBudgets, VmTickError, VmTickResult,
    WocProjectVm, WocReloadableVm,
};

/// Startup and export execution use separate cooperative budgets. Project
/// compilation and non-returning native callbacks still need host process limits.
#[derive(Clone, Debug)]
pub struct ZrVmProjectOptions {
    pub runtime: RuntimeOptions,
    pub startup_max_instructions: u64,
    pub startup_timeout: Duration,
    pub max_call_instructions: u64,
    pub lifecycle_timeout: Duration,
    pub cancellation: Option<CancellationToken>,
}

impl Default for ZrVmProjectOptions {
    fn default() -> Self {
        let budgets = TickBudgets::default();
        Self {
            runtime: RuntimeOptions {
                heap_limit_bytes: budgets.max_memory_bytes,
                pause_budget_us: budgets.max_gc_micros,
                remark_budget_us: budgets.max_gc_micros,
                worker_count: 0,
            },
            startup_max_instructions: 10_000_000,
            startup_timeout: Duration::from_secs(30),
            max_call_instructions: 10_000_000,
            lifecycle_timeout: Duration::from_micros(budgets.max_execution_micros),
            cancellation: None,
        }
    }
}

// Field order keeps callback registrations alive until all session/checkpoint
// owners have dropped, then unregisters them before destroying the runtime.
struct RuntimeResources {
    _registrations: Vec<NativeModuleRegistration>,
    _runtime: Runtime,
}

pub struct ZrVmProjectCheckpoint {
    vm: ProjectSessionCheckpoint,
    resources: Rc<RuntimeResources>,
}

/// A retained, thread-confined project session. Native modules are supplied by
/// the embedding host under its capability policy; all calls use the SDK budget.
pub struct ZrVmProjectVm {
    session: ProjectSession,
    resources: Rc<RuntimeResources>,
    options: ZrVmProjectOptions,
    schema: String,
    rollback_fault: Option<VmTickError>,
    last_call_usage: Option<CallUsage>,
}

impl ZrVmProjectVm {
    /// Loads the project in interpreter mode and verifies its actual schema.
    /// The caller activates the package before its first tick.
    pub fn load(
        project_path: impl AsRef<Path>,
        options: ZrVmProjectOptions,
        native_modules: impl IntoIterator<Item = NativeModule>,
    ) -> Result<Self, VmTickError> {
        let mut runtime = RuntimeBuilder::standard()
            .options(options.runtime)
            .build()
            .map_err(exports::binding_error)?;
        let mut registrations = Vec::new();
        for module in native_modules {
            registrations.push(
                runtime
                    .register_native_module(module)
                    .map_err(exports::binding_error)?,
            );
        }
        let workspace = ProjectWorkspace::open(project_path).map_err(exports::binding_error)?;
        if workspace.entry_module().map_err(exports::binding_error)? != "main" {
            return Err(VmTickError::Limited(
                "WOC project entry must be main".into(),
            ));
        }
        let session = workspace
            .start_session_with_budget(
                &mut runtime,
                &RunOptions::default(),
                &exports::startup_budget(&options),
            )
            .map_err(exports::call_error)?;
        let mut vm = Self {
            session,
            resources: Rc::new(RuntimeResources {
                _registrations: registrations,
                _runtime: runtime,
            }),
            options,
            schema: String::new(),
            rollback_fault: None,
            last_call_usage: None,
        };
        vm.schema = vm.observe(|vm| {
            let mut budget = vm.lifecycle_budget();
            let encoding = vm.call("stateEncoding", &[], &mut budget)?;
            if encoding.as_string().map_err(exports::binding_error)? != "vm-state-bytes-v1" {
                return Err(VmTickError::Limited(
                    "WOC package must declare vm-state-bytes-v1 lifecycle state".into(),
                ));
            }
            let value = vm.call("stateSchema", &[], &mut budget)?;
            let schema = value.as_string().map_err(exports::binding_error)?;
            codec::validate_schema(&schema)?;
            Ok(schema)
        })?;
        Ok(vm)
    }

    pub fn last_call_usage(&self) -> Option<CallUsage> {
        self.last_call_usage
    }

    /// Cancellation is one-shot. Install a fresh token before resuming a
    /// recovered session after its previous token was cancelled.
    pub fn set_cancellation(&mut self, cancellation: Option<CancellationToken>) {
        self.options.cancellation = cancellation;
    }

    fn ensure_healthy(&self) -> Result<(), VmTickError> {
        match &self.rollback_fault {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }

    fn restore_world_state(&mut self, state: &[u8]) -> Result<(), VmTickError> {
        codec::validate_world_state(state)?;
        self.transaction(|vm| {
            let mut budget = vm.lifecycle_budget();
            let arguments = [codec::lower_bytes(state)?];
            codec::require_success(vm.call("restoreState", &arguments, &mut budget)?)?;
            drop(arguments);
            let restored =
                vm.observe(|vm| codec::lift_bytes(&vm.call("saveState", &[], &mut budget)?))?;
            if restored != state {
                return Err(VmTickError::Transport(
                    "restoreState did not install the supplied canonical WOS state".into(),
                ));
            }
            Ok(())
        })
    }

    fn lifecycle(&mut self, export: &str) -> Result<(), VmTickError> {
        self.transaction(|vm| {
            let mut budget = vm.lifecycle_budget();
            codec::require_success(vm.call(export, &[], &mut budget)?)
        })
    }

    fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, VmTickError>,
    ) -> Result<T, VmTickError> {
        let checkpoint = self.checkpoint()?;
        match operation(self) {
            Ok(value) => Ok(value),
            Err(source) => Err(self.abort(&checkpoint, source)),
        }
    }

    fn observe<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, VmTickError>,
    ) -> Result<T, VmTickError> {
        let checkpoint = self.checkpoint()?;
        match operation(self) {
            Ok(value) => {
                self.rollback(&checkpoint)?;
                Ok(value)
            }
            Err(source) => Err(self.abort(&checkpoint, source)),
        }
    }

    fn abort(&mut self, checkpoint: &ZrVmProjectCheckpoint, source: VmTickError) -> VmTickError {
        match self.rollback(checkpoint) {
            Ok(()) => source,
            Err(rollback) => VmTickError::Rollback {
                source: Box::new(source),
                rollback: Box::new(rollback),
            },
        }
    }
}

impl WocProjectVm for ZrVmProjectVm {
    type Checkpoint = ZrVmProjectCheckpoint;

    fn checkpoint(&mut self) -> Result<Self::Checkpoint, VmTickError> {
        self.ensure_healthy()?;
        Ok(ZrVmProjectCheckpoint {
            vm: self.session.checkpoint().map_err(exports::binding_error)?,
            resources: Rc::clone(&self.resources),
        })
    }

    fn rollback(&mut self, checkpoint: &Self::Checkpoint) -> Result<(), VmTickError> {
        if !Rc::ptr_eq(&self.resources, &checkpoint.resources) {
            return Err(VmTickError::Transport(
                "checkpoint belongs to another project".into(),
            ));
        }
        match self
            .session
            .rollback(&checkpoint.vm)
            .map_err(exports::binding_error)
        {
            Ok(()) => {
                self.rollback_fault = None;
                Ok(())
            }
            Err(error) => {
                self.rollback_fault = Some(error.clone());
                Err(error)
            }
        }
    }

    fn install_full_snapshot(&mut self, snapshot: &CommittedSnapshot) -> Result<(), VmTickError> {
        if woc_protocol::fnv1a_bytes(&snapshot.state) != snapshot.state_digest
            || woc_protocol::fnv1a_bytes(&snapshot.presentation_payload)
                != snapshot.presentation_digest
        {
            return Err(VmTickError::Transport("snapshot digest mismatch".into()));
        }
        self.restore_world_state(&snapshot.state)
    }

    fn fixed_tick(
        &mut self,
        input_payload: &[u8],
        budgets: TickBudgets,
    ) -> Result<VmTickResult, VmTickError> {
        let input = codec::decode_tick_input(input_payload)?;
        self.transaction(|vm| {
            let mut budget = vm.export_budget(Duration::from_micros(budgets.max_execution_micros));
            budget.max_heap_bytes = Some(budgets.max_memory_bytes);
            budget.max_native_calls = Some(budgets.max_host_calls);
            budget.max_gc_time = Some(Duration::from_micros(budgets.max_gc_micros));
            let arguments = [codec::lower_bytes(input_payload)?];
            let output_payload =
                codec::lift_bytes(&vm.call("fixedTick", &arguments, &mut budget)?)?;
            drop(arguments);
            codec::validate_tick_output(&output_payload, input.tick)?;
            let simulation_usage = exports::tick_usage(vm.last_call_usage.ok_or_else(|| {
                VmTickError::Transport("ZrVM export returned without call telemetry".into())
            })?);
            let presentation_payload = codec::lift_presentation_bytes(&vm.call(
                "presentationProjection",
                &[],
                &mut budget,
            )?)?;
            ClientPresentationProjection::decode_json(&presentation_payload).map_err(|error| {
                VmTickError::Transport(format!(
                    "WOC presentationProjection violates the client contract: {error:?}"
                ))
            })?;
            let presentation_usage = exports::tick_usage(vm.last_call_usage.ok_or_else(|| {
                VmTickError::Transport("ZrVM export returned without call telemetry".into())
            })?);
            Ok(VmTickResult {
                output_payload,
                presentation_payload,
                usage: exports::merge_tick_usage(simulation_usage, presentation_usage),
            })
        })
    }
}

impl WocReloadableVm for ZrVmProjectVm {
    fn state_schema(&self) -> Result<String, VmTickError> {
        self.ensure_healthy()?;
        Ok(self.schema.clone())
    }

    fn save_state(&mut self) -> Result<Vec<u8>, VmTickError> {
        self.observe(|vm| {
            let mut budget = vm.lifecycle_budget();
            let state = codec::lift_bytes(&vm.call("saveState", &[], &mut budget)?)?;
            codec::validate_world_state(&state)?;
            Ok(state)
        })
    }

    fn deactivate(&mut self) -> Result<(), VmTickError> {
        self.lifecycle("deactivate")
    }

    fn activate(&mut self) -> Result<(), VmTickError> {
        self.lifecycle("activate")
    }

    fn restore_state(&mut self, state: &[u8]) -> Result<(), VmTickError> {
        self.restore_world_state(state)
    }
}
