---
related_code:
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/gc.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/dispatch.rs
  - tools/tests/test_plugins08_vm_active_interface_snapshot.py
plan_sources:
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/performance/01/2026-07-22-runtime-script-static-review.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Developer/HotReload/Private/HotReload.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Modules/ModuleManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp
  - dev/Fyrox/fyrox-impl/src/plugin/dylib.rs
tests:
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/dispatch.rs::panicked_export_restores_the_active_instance
  - python -B -m unittest tools.tests.test_plugins08_vm_active_interface_snapshot -v
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/script_vm_tests.rs::runtime_15_script_vm_hot_reload_coordinator_tests_are_folder_backed
  - cargo +1.94.1 test -p zircon_runtime --lib hot_reload_coordinator --locked --jobs 1 -- --nocapture --test-threads=1
doc_type: implementation-plan
status: source_implemented_static_passed_managed_validation_pending
---

# Runtime VM slot invocation lease architecture and profiling plan

## Current-source finding

The current `HotReloadCoordinator` already preserves several important invariants: staged reflection publication, state-schema migration, generation-scoped host registrations, failed-reload rollback, cooperative GC deadlines, and O(1)-amortized pending-GC membership. These owners must remain authoritative.

The current invocation/lifecycle boundary is nevertheless not a complete runtime transaction:

- `call_slot_export` removes the only `VmPluginInstance` from the global slot table, calls foreign backend code, and restores it only on the ordinary return path. A panic therefore leaves an `Active` slot with no instance. Every later export, GC step, reload, and unload then observes a permanently unavailable instance.
- `load_package`, `hot_reload`, and `unload_slot` share one coordinator-wide `lifecycle_guard`. Unrelated packages are serialized even though their state and generation are independent.
- Export and GC do not acquire a typed per-slot invocation lease. Reload does not close new admission and drain already admitted work; it races by attempting to take the instance and reports a generic unavailable-instance failure.
- `hot_reload_coordinator.rs` is 858 lines. GC queue/schedule/execution policy has grown back into the lifecycle root, exceeding the repository's 800-line review budget and failing the existing folder-backed owner guard.

This is primarily a correctness and ownership problem. No micro-optimization is authorized from this review.

## Algorithm and lock topology baseline

| Path | Current complexity | Current synchronization | Structural risk |
| --- | --- | --- | --- |
| slot export | O(1) table lookup | global slot-table mutex, then unguarded foreign call with instance removed | panic can orphan instance; reload cannot quiesce |
| reload/unload | O(1) slot lookup plus backend/state/reflection work | one global lifecycle mutex for every slot | unrelated slots serialize; no deadline-bound drain |
| active/list projection | O(S), list/name lookup also sorts O(S log S) | global slot-table mutex | acceptable for control-plane snapshots; not a frame hot path |
| cooperative GC admission | O(1) amortized queue membership and indexed due schedule | global GC-step guard plus short queue/table locks | correct bounded queue shape; execution owner is mixed into lifecycle root |

The static review does not establish wall-time, CPU, power, allocator, or cross-engine performance. Those claims remain prohibited until a current-source dynamic matrix is captured.

## Reference-engine decision

Unreal is the primary lifecycle reference. Its hot-reload path first validates the affected package set, builds dependent modules, compiles to a rolling candidate module name, and only then enters the rebind/reinstance phase. It records operation duration and emits an explicit reload-complete boundary. The useful rule for Zircon is candidate preparation before publication and a named transition boundary; the old Unreal hot-reload implementation is not copied as a memory-safety model.

Fyrox is a secondary Rust landing reference only. Its dynamic-plugin state explicitly becomes `Unloaded` before copying and loading the replacement library. That makes ownership visible, but it discards last-good availability and has no bounded quiescence contract. Zircon must retain its stronger staged reflection/state rollback and add per-slot admission/drain rather than adopting this unload-first behavior.

## Chosen architecture

### M0: invariant repair and owner restoration

1. Move cooperative GC execution and policy helpers to the existing folder-backed coordinator owner `hot_reload_coordinator/gc.rs`. Keep the coordinator root as the public facade and shared lock owner.
2. Contain an export panic only long enough to restore the exact instance to its slot, then resume the original unwind. Do not swallow or relabel the backend panic.
3. Add a behavioral regression proving a panicked export leaves the slot `Active`, preserves its generation, and permits the next export to use the same instance.

This slice changes neither public API nor scheduling policy. Its producer and ordinary success-path complexity remain O(1), and it adds no allocation, thread, queue, or compatibility facade.

### M1: measured per-slot invocation lease

After the profiling matrix below exists, replace the global remove/restore protocol with one stable per-slot owner:

- a generation-bearing slot state owns admission, the exclusive mutable backend instance, active invocation state, and a condition variable or Runtime task terminal;
- reload/unload atomically close new admission for only the affected slot, wait for pre-close work to drain to a declared deadline, prepare the candidate generation, publish once, then retire the previous generation;
- export, GC, debug pause, and state capture use the same typed lease family; no caller creates a second mutex, worker, or queue;
- unrelated slots remain independently executable; same-slot mutable calls stay serialized because `VmPluginInstance` is mutable;
- panic-safe lease drop restores accounting exactly once, while a backend panic remains observable as the original panic/terminal fact.

The public transition API must eventually return a typed report distinguishing admission closed, quiesce timeout, preparation failure, publication conflict, rollback failure, and retirement leak. A string-only busy retry is not the target design.

## Profiling matrix before M1

Run only through the managed Windows validator, with build/trace artifacts under `D:` or `E:`:

| Matrix | Required measurements | Acceptance question |
| --- | --- | --- |
| slots 1/100/10k, active calls 0/1/1k | slot-table and lifecycle-lock wait, unavailable-instance rejects, allocations, wall time | is the global lifecycle mutex or table projection the dominant control-plane cost? |
| callback body 0/1/100 ms, reload same/different slot | admission-close latency, drain latency, unrelated-slot progress, reload wall | does per-slot ownership remove cross-slot serialization without changing same-slot safety? |
| export panic 1/100, GC panic 1/100 | restored instances, terminal state, retained registrations, RSS/handles | does every unwind preserve slot conservation and avoid retained-state growth? |
| reload loop 1/100/1k generations | publish/rollback duration, stale handles, resident generation count, RSS | does old-generation retirement remain bounded? |

Use WPR CPU/context-switch sampling when host policy permits and add low-cardinality coordinator counters before changing M1 synchronization. If WPR is rejected, record the exact host error and retain process/lock/counter samples. Do not infer power or Unreal/Fyrox parity from compile-inclusive Cargo wall time.

## Milestone status

| Milestone | State | Completed work | Remaining work |
| --- | --- | --- | --- |
| Review and direction | `completed` | current-source lifecycle/GC/lock review; UE primary and Fyrox secondary comparison; complexity and profiling matrix | none |
| M0 owner/invariant repair | `source_implemented_static_passed` | GC execution moved to a 134-line named owner; coordinator root reduced from 858 to 738 lines; export panic restores the instance before resuming unwind; 73-line behavior regression added; focused static contract 9/9, scoped rustfmt, trailing-whitespace scan, and tracked diff check pass | managed Rust behavior and structure gates |
| M1 per-slot lease | `pending_profile` | target state and acceptance metrics defined | managed dynamic baseline, API decision, implementation and product validation |

## Current validation evidence

- `python -B -m unittest tools.tests.test_plugins08_vm_active_interface_snapshot -v`: 9/9 passed in 0.028 s. The guard now reads `gc_step` and `refresh_gc_schedule` from the named GC owner and verifies export restoration precedes unwind propagation.
- `rustfmt --edition 2021 --check` passed for the coordinator root, GC owner, test mount, and panic regression.
- The scoped trailing-whitespace scan passed. `git diff --check` passed for tracked files with only the repository's LF-to-CRLF warning.
- The current owner sizes are coordinator root 738 lines, GC owner 134 lines, and panic regression 73 lines. The root is back below the 800-line review budget.
- The managed `hot_reload_coordinator` and `structure_convention` Cargo gates were not run in this slice. The latter remains a coordinator-managed Cargo command, as confirmed by the convention dry-run. The panic behavior test is present but is not reported as executed.

No performance, power, MVP acceptance, milestone commit, or cross-engine parity is claimed by this record.
