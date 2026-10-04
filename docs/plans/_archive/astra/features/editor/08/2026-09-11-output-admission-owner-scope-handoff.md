---
record_kind: dependency_handoff
status: blocked_owner_scope
created_at: 2026-09-11
plan: docs/plans/astra/features/editor/08-output-admission.md
milestone: EDITOR-OUTPUT-ADMISSION-M22
session: astra-editor-output-admission-owner-handoff-20260911-01a090c1
---

# Editor output-admission P1 owner-scope handoff

## Decision

The bounded P1 audit selected `08-output-admission.md` (M22), but no safe
implementation slice remains available in this checkout. The production
transport repair and its regressions are already staged/dirty under the
archived output-admission owner, while neighboring wizard/controller and
view-model paths are owned by other archived plans. This lane therefore made
no source or test edits and records an owner handoff only.

The P1 plan remains `in_progress`, with M22 marked
`implemented_pending_validation`. The goal authorization allows listed P1
repairs, but does not authorize taking over existing foreign dirty bytes.
ED-A5, ED-A6, and the independent export-failure P0 lane were not touched.

## Contract and current-source evidence

| Boundary | Current bytes | Ownership evidence |
| --- | --- | --- |
| Output admission policy | `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/event_transport.rs:6-34` defines a 192-slot channel, reserves 128 pending output slots plus `2 * ExportStage::ALL.len() + 3` control slots, coalesces only when pending output reaches 128, and uses `try_send` for all events. | Staged new file; no executable owner in the current matrix. |
| Controller channel | `.../wizard/controller.rs:21-52` owns the receiver and constructs the bounded channel; `:72-89` drains remaining events at completion. | Modified under archived `astra-editor-output-admission-20260905`; matrix request `6199377ddb3742aba8935c1b3c59c46c`, hash `ac86369d201d85ff3a7c25e2fb0c7b08ec7a1eecefe99c77009eb6281d7e127a`, with stale attribution/no live lease. |
| Job producer | `.../wizard/controller/job.rs:44-91` routes every emitted event through `send_job_event`, carries the coalesced count, and converts terminal snapshots into job results. | Modified under the same archived owner; matrix request `5eaa81f7f6ce4e4abe720820b12eab9a`, hash `b9b87274ed86b3ff1802fc7d3abf0d09f8385a1efc8b28c35753bafa0608b789`, with stale attribution/no live lease. |
| Admission regressions | Staged `.../wizard/controller/event_transport/astra_admission_tests.rs:45-227` covers immediate drain, partial drain, absent consumer, all terminal kinds, and disconnected receiver. | Staged addition is not independently attributable in the current matrix; it must be reconciled with the transport owner. |
| View-model consumer | `.../wizard/view_model.rs:199-210` drains the receiver and applies events; its modified bytes are separately attributed to archived `astra-editor-artifact-projection-20260905` (matrix request `0789956b00504d40b9fb79a4dd180620`, hash `f74d59a0c96ae68a5de7318f438a46b5162eb1f540c7b28e175b225319b16aa0). | Cross-owner dependency; do not absorb into M22 without reconciliation. |

The staged transport file replaced the former per-stage fixed counter in
`controller/job.rs` (the old implementation and tests are visible in the
unstaged diff). This confirms that the requested repair is already materially
present, not an untouched lowest-layer bug. The current checkout also has
dirty `wizard/streaming_output_tests.rs` under archived owner
`astra-editor-output-admission-20260905` (matrix request
`5537f4c7d3224262bb629efa1078770e`, hash
`343f1db275f83250d445b63ef9f939b5d682b7537319dd5a906c6da05dcbef14`).

## Why implementation is blocked

M22 spans channel capacity, producer admission, terminal reservation, and
consumer drain semantics. The producer and transport are already staged while
the controller/job and view-model are dirty under stale or archived owners.
Changing only one path could duplicate the staged repair, invalidate the
terminal-capacity invariant, or mismatch the consumer's event projection.
The coordinator baseline is degraded and reports stale attribution,
`owner_not_executable`, and `live_lease_missing` on behavior-bearing paths.
No source lease can safely be claimed for a new implementation in this lane.

## Dependency-ready owner scope

1. **M22 transport owner:** reconcile the existing staged/dirty diff, then
   claim this atomic source/test set:
   `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/controller/event_transport.rs`,
   `.../wizard/controller/job.rs`,
   `.../wizard/controller/event_transport/astra_admission_tests.rs`, and
   `.../wizard/streaming_output_tests.rs`. Preserve the 128/19 capacity
   invariant, output coalescing monotonicity, disconnected-send behavior, and
   terminal ordering while resolving any stale old tests.
2. **Consumer owner:** coordinate with the existing artifact/output projection
   owner before changing `.../wizard/controller.rs` or
   `.../wizard/view_model.rs`; only include them if the reconciled event drain
   contract proves a required change.
3. After ownership restoration, run the plan-declared focused managed tests and
   product/editor validation. Static source inspection or staged tests are not
   acceptance. This handoff ran no Cargo, native, product, or benchmark
   command.

## Coordination receipt

Session `astra-editor-output-admission-owner-handoff-20260911-01a090c1` owns
only this record. The record lease is released after attribution; the session
then waits for validation. No production/test source lease was claimed, and
acceptance remains pending.
