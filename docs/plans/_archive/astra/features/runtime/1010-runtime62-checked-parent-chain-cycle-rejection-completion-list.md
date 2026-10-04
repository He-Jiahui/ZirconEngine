---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/62/2026-09-27-checked-parent-chain-cycle-rejection.md
implementation_files:
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/error.rs
tests:
  - zircon_runtime/src/scene/tests/derived_state/checked_parent_chain.rs
  - zircon_runtime/src/scene/world/hierarchy/checked_parent_chain_profile.rs
  - zircon_runtime/src/scene/tests/derived_state/work_counters.rs
  - zircon_runtime/src/scene/tests/derived_state/spawn_paths.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_derived_state.rs
---

# Runtime1010 checked parent chain cycle rejection

| Work | Implementation evidence | Remaining acceptance |
|---|---|---|
| Reject pre-existing cyclic ancestry before checked reparent mutates the World. | Constant-space Brent traversal returns `HierarchyParentChainCycle { start, repeated }`; the sole caller propagates the Result. Existing ancestor/self/missing/Static errors and both fact publications retain their routes. | Independent source review passed; grouped managed compile/behavior execution pending. Only the checked-reparent subpath is addressed; full RSH-P0-001/G03 remains open. |
| Preserve pending state on rejection and legitimate hierarchy edits. | Eight actual World tests cover four corrupt shapes, direct serde before flush, complete state/fact invariance, ancestor priority and valid/no-op/detach/rejection behavior. Tests were authored before the production guard. | No dynamic red/green execution is claimed. Existing raw-cycle flush-repair and hierarchy behavior regressions must also pass. |
| Provide comparable deep-chain Release evidence. | Real `set_parent_checked` and the complete frozen preimage run on valid depths 1/32/1K/100K; five warmup and 31 alternating sample pairs emit raw p50/p95/p99. Setup and restoration are untimed. | Measurements pending. No speedup or arbitrary ceiling is claimed; full G24 work/allocation/latency, product and cross-engine gates remain open. |
| Restore the existing folder-backed test gate in V2. | Seven preexisting work-counter tests and their helpers move verbatim into `work_counters.rs`. The parent has zero executable tests; the gate preserves previous ownership assertions and the below-800-line budget, adding both child owners for 48 declarations across its seven enumerated files. V1 production, cycle tests, profile and frozen evidence remain unchanged. | Static source-gate evaluation and exact block preservation are recorded separately for V2. Independent V2 source review passed; managed structure/behavior execution remains pending. |
| Correct the existing spawn-test source assumptions in V2. | `CR-SCENE-TEST-CONTRACT-0001` now reads `components/scene/identity.rs` and locally excludes `///` lines from its adjacency check. Actual spawn, owned-batch, preparation and commit slices preserve validation before final prevalidated publication; complete NodeKind/Copy, default-record, no-clone and ordinal requirements remain. Marker and foreign module notes are retained. | Static source checks and managed execution remain distinct; no dynamic pass is claimed. |
