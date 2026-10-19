---
status: in_progress
---

# Runtime Task Validation Worklist

Current subsystem policy: [JobSystem](../crates/zircon_runtime/core/job_system.md).

## Task authority and lifecycle

EngineTaskGraph owns budgeted Io/AsyncCompute/Compute pools for each CoreRuntime. TaskDescriptor.kind selects its domain; TaskGraphScope owns admission, scope drain and close/join. JobScheduler is constructed from the active pool, not a hidden private scheduler. Editor settings persistence receives that route explicitly; removed default/process_io constructors must not return.

ScheduleParallelExecutor models batch dependencies through the task graph. Direct rayon access is restricted to approved core task primitives; parallel_frustum remains a tracked render exception until its separate cutover is validated. The rayon_is_only_reachable_through_core_task_primitives negative self-check remains required.

job_system inventory, source_helpers.rs, split_layout.rs and mirror_docs.rs retain separate responsibilities and runtime index/interface convergence mirrors. The 2026-07-28 direct_rayon_paths = 2 snapshot is historical, not permission for additional direct callers. Blocking stream and retained-result owners must bound admission/bytes/readers, remain visible to shutdown accounting and avoid private threads; managed thread-peak, worker-join and power acceptance remains pending.

This worklist retains its declared status independently of module policy, the runtime index and archive evidence. No new test/product acceptance is recorded.

### Source and guard ownership

The finite source, guard and provenance inventory for these responsibilities is:

- `job_system/mirror_docs.rs`, `parallel_frustum.rs`.

### Validation interfaces

Keep these existing validation and domain interfaces in the gate:

- `runtime_11_job_system_mirror_docs_match_structure_audit_counts`.

### Contract details

The following original contract labels remain in scope; they do not establish new validation:

- `TaskPoolKind`.

### Clause provenance

These content identities preserve the source of this transfer. They are not fresh measurements, active-file dependencies or acceptance receipts.

- `11-job-system-task-model.md`, SHA-256 `da080f3bd2caf28058fb23c5f6ffb39b6811c4dea530e441b635026b6e02a8f4`, selected source lines 83-378.
