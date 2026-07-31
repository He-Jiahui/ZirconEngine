---
related_code:
  - zircon_editor/src/core/script_build/mod.rs
  - zircon_editor/src/core/script_build/request.rs
  - zircon_editor/src/core/script_build/orchestrator.rs
  - zircon_editor/src/core/script_build/tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor/13-script-compilation-management.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - zircon_editor/src/core/script_build/tests.rs
  - tools/tests/test_editor13_script_build_orchestrator_contract.py
doc_type: module-detail
---

# Editor Script Build Orchestration

`core::script_build` is the headless-safe policy owner between editor triggers and a future script compiler executor. It does not spawn threads, call the VM, publish diagnostics, or enter Play. Callers feed deterministic millisecond timestamps and execute the returned step dispatches through the job/VM layer owned by later Plan13 milestones.

Watch changes use a sliding 300 ms deadline and a `BTreeSet<PathBuf>`, so duplicate paths collapse and the resulting incremental list is stable. A Command or Play request immediately consumes the pending watch batch. Up to 20 paths are carried by `ScriptBuildStep::CompileModules`; a larger batch becomes an empty module list, which is the single contract for a full module compile.

Each request always dispatches `CompileModules`, `ValidateLedger`, then `RefreshBindings`. Only one step can be in flight. `complete(dispatch, Succeeded)` advances to the next step; the orchestrator verifies both the request id and step index carried by the original dispatch. A mismatched request, stale step, or completion without an in-flight step returns a typed error without mutating the active request. Any failed step drops the remaining steps, queued requests, and unbatched watch paths. A Play request reports `resume_play=true` only after its third step succeeds.

`ScriptBuildStepDispatch` is a linear execution ticket: it is not cloneable, the executor takes it by value, and
`complete(dispatch, outcome)` consumes the same ticket when the side effect finishes. This prevents one compile
or binding-refresh step from being submitted to multiple workers and then merely rejecting the second completion.

Command, Play, and due Watch admission reserve a request id before consuming pending watch state. Exhausted id
space returns `ScriptBuildEnqueueError::RequestIdExhausted`; queued requests, full-rebuild sentinel, pending paths,
and deadline remain unchanged. `take_ready` therefore returns a typed `Result<Option<_>, _>` rather than hiding
admission failure as an empty queue.

`ScriptBuildSnapshot` is the read-only projection for later status-bar and Play-state consumers. It reports phase, active request id, queued request count, pending watch count/deadline, and the last terminal outcome. Diagnostics DTOs, event-bus publication, real VM execution, EditorJob integration, and commandlet wiring remain outside this M1.1 boundary.
