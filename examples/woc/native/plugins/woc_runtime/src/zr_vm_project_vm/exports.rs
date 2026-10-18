use std::time::Duration;

use zr_vm_rust_binding::{
    CallBudget, CallError, CallTermination, CallUsage, Error, ExecutionDeadline, Value,
};
use zr_vm_rust_binding_sys::ZrRustBindingStatus;

use super::{ZrVmProjectOptions, ZrVmProjectVm};
use crate::{TickBudgets, TickUsage, VmExecutionTermination, VmTickError};

pub(super) fn startup_budget(options: &ZrVmProjectOptions) -> CallBudget {
    CallBudget {
        max_instructions: Some(options.startup_max_instructions),
        deadline: Some(ExecutionDeadline::after(options.startup_timeout)),
        cancellation: options.cancellation.clone(),
        max_heap_bytes: Some(options.runtime.heap_limit_bytes),
        max_native_calls: Some(TickBudgets::default().max_host_calls),
        max_gc_time: Some(Duration::from_micros(options.runtime.pause_budget_us)),
    }
}

impl ZrVmProjectVm {
    pub(super) fn export_budget(&self, timeout: Duration) -> CallBudget {
        CallBudget {
            max_instructions: Some(self.options.max_call_instructions),
            deadline: Some(ExecutionDeadline::after(timeout)),
            cancellation: self.options.cancellation.clone(),
            max_heap_bytes: Some(self.options.runtime.heap_limit_bytes),
            max_native_calls: Some(TickBudgets::default().max_host_calls),
            max_gc_time: Some(Duration::from_micros(self.options.runtime.pause_budget_us)),
        }
    }

    pub(super) fn lifecycle_budget(&self) -> CallBudget {
        self.export_budget(self.options.lifecycle_timeout)
    }

    pub(super) fn call(
        &mut self,
        export: &str,
        arguments: &[Value],
        budget: &mut CallBudget,
    ) -> Result<Value, VmTickError> {
        let result = self
            .session
            .call_module_export_with_budget("main", export, arguments, budget);
        let usage = match &result {
            Ok(result) => result.usage,
            Err(error) => error.usage,
        };
        self.last_call_usage = Some(usage);
        if let Some(remaining) = &mut budget.max_instructions {
            *remaining = remaining.saturating_sub(usage.executed_instructions);
        }
        if let Some(remaining) = &mut budget.max_native_calls {
            *remaining = remaining.saturating_sub(usage.native_calls);
        }
        if let Some(remaining) = &mut budget.max_gc_time {
            *remaining = remaining.saturating_sub(usage.gc_time);
        }
        result.map(|result| result.value).map_err(call_error)
    }
}

pub(super) fn binding_error(error: Error) -> VmTickError {
    match error.status {
        ZrRustBindingStatus::ZR_RUST_BINDING_STATUS_UNSUPPORTED
        | ZrRustBindingStatus::ZR_RUST_BINDING_STATUS_NOT_FOUND => {
            VmTickError::Limited(error.to_string())
        }
        ZrRustBindingStatus::ZR_RUST_BINDING_STATUS_RUNTIME_ERROR => {
            VmTickError::Trap(error.to_string())
        }
        _ => VmTickError::Transport(error.to_string()),
    }
}

pub(super) fn call_error(error: CallError) -> VmTickError {
    let reason = match error.termination {
        Some(CallTermination::InstructionLimit) => VmExecutionTermination::InstructionLimit,
        Some(CallTermination::Deadline) => VmExecutionTermination::Deadline,
        Some(CallTermination::Cancelled) => VmExecutionTermination::Cancelled,
        Some(CallTermination::HeapLimit) => VmExecutionTermination::HeapLimit,
        Some(CallTermination::NativeCallLimit) => VmExecutionTermination::NativeCallLimit,
        Some(CallTermination::GcTimeLimit) => VmExecutionTermination::GcTimeLimit,
        None => return binding_error(error.error),
    };
    VmTickError::Terminated {
        reason,
        executed_instructions: error.usage.executed_instructions,
        elapsed_micros: u64::try_from(error.usage.elapsed.as_micros()).unwrap_or(u64::MAX),
        usage: tick_usage(error.usage),
    }
}

pub(super) fn tick_usage(usage: CallUsage) -> TickUsage {
    TickUsage {
        execution_micros: u64::try_from(usage.elapsed.as_micros()).unwrap_or(u64::MAX),
        memory_bytes: usage.peak_heap_bytes,
        host_calls: usage.native_calls,
        gc_micros: u64::try_from(usage.gc_time.as_micros()).unwrap_or(u64::MAX),
    }
}

pub(super) fn merge_tick_usage(first: TickUsage, second: TickUsage) -> TickUsage {
    TickUsage {
        execution_micros: first
            .execution_micros
            .saturating_add(second.execution_micros),
        // Heap telemetry is a per-call high-water mark over the same retained VM.
        memory_bytes: first.memory_bytes.max(second.memory_bytes),
        host_calls: first.host_calls.saturating_add(second.host_calls),
        gc_micros: first.gc_micros.saturating_add(second.gc_micros),
    }
}
