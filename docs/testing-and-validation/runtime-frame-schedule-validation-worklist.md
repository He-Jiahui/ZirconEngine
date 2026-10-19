---
status: completed
---

# Runtime Frame Schedule Validation Worklist

## Frame authority and scheduling gates

One frame path connects App entry, explicit SystemStage ordering and extract submission. A FixedStepPlan repeats FixedFirst/FixedUpdate/FixedPostUpdate, consumes RuntimeTimeAdvance once and forbids a second WorldDriver advance_time_by(...) or raw-delta session tick. The dynamic session retains its documented UI extract side path rather than introducing a second time authority.

Keep the stage-order, fixed-step clamp/overstep, no-step and repeated-step behavior tests, parallel/serial equivalence, parallel report and disabled fallback tests below. The parallel executor preserves batch dependencies and run_batches compatibility.

schedule_frame_loop_source_inventory.py owns sources; schedule_frame_loop_anchor_inventory.py owns stage/behavior/doc/Cargo anchors; schedule_frame_loop_boundary owns risk and missing checks; Markdown rendering remains separate. The dated inventory metrics below are retained guard targets; their source identities and dates remain attached to each snapshot. They do not rerun the historical 77/77 schedule or 4/4 time evidence.

Post-completion failure still needs actual runtime_entry validation and Desktop/Editor/Headless idle, event-storm and duplicate-resize evidence; none reopens completed schedule milestones or asserts product acceptance here.

This worklist retains its declared status independently of module policy, the runtime index and archive evidence. No new test/product acceptance is recorded.

### Source and guard ownership

The finite source, guard and provenance inventory for these responsibilities is:

- `src/core/framework/scene/system_stage.rs`, `src/dynamic_api/session/profile.rs`, `schedule_frame_loop/mirror_docs.rs`.
- `src/dynamic_api/session/state.rs`, `src/core/runtime/handle/time.rs`, `src/core/runtime/time.rs`.
- `src/tests/time.rs`.

### Validation interfaces

Keep these existing validation and domain interfaces in the gate:

- `schedule_stage_plan_orders_steps_by_explicit_declaration_not_registration`, `schedule_parallel_executor_can_run_parallel_batches_serially_with_report`, `schedule_parallel_disabled_path_runs_serial_batches_with_fallback_counts`, `runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts`.
- `session_ui_extract_remains_documented_dynamic_session_side_path`, `representative_schedule_produces_multi_system_parallel_batches`, `schedule_parallel_execution_report_records_diagnostic_counts`, `parallel_and_serial_execution_reach_identical_world_state`.
- `schedule_parallel_report_keeps_run_batches_compatible`.

### Recorded targets

These retained targets belong to the original source snapshots, not a new measurement. The source dates/identities below keep distinct inventories separate.

- dynamic-session `.tick_time(...)` calls 1/1; source 2026-07-14:L46.
- `SystemStage` count and variants 9/9; source 2026-07-14:L46.
- `missing_behavior_test_anchors = []`; source 2026-07-14:L46.
- `behavior_test_anchor_count = 14`; source 2026-07-14:L46.
- `Runtime 03 guard anchors 14/14`; source 2026-07-14:L46.
- `guard/test files 11/11`; source 2026-07-14:L46.
- `fixed-loop stages 3/3`; source 2026-07-14:L46.
- `doc_anchors = 10/10`; source 2026-07-14:L46.
- `source files 22/22`; source 2026-07-14:L46.

### Contract details

The following original contract labels remain in scope; they do not establish new validation:

- no `WorldDriver` second `advance_time_by(...)` references, `no dynamic-session raw-delta level tick references`, `interface convergence`, `RenderExtract`.

### Clause provenance

These content identities preserve the source of this transfer. They are not fresh measurements, active-file dependencies or acceptance receipts.

- `03-schedule-and-frame-loop-alignment.md`, SHA-256 `1bb29f3a99827db5c098d19bccff2addd703b000b66799a56313b6a0cf49617f`, selected source lines 44-211.
