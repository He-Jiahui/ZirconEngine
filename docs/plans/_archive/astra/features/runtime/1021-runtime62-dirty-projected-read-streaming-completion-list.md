---
doc_type: milestone-detail
related_code:
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/tests/derived_state/projected_reads.rs
  - zircon_runtime/tests/runtime_scene_projected_read_performance.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime1021-dirty-projected-read-streaming.md
tests:
  - zircon_runtime/src/scene/tests/derived_state/projected_reads.rs
  - zircon_runtime/tests/runtime_scene_projected_read_performance.rs
---

# Runtime1021: Runtime62 dirty projected-read streaming

Status: `candidate_static_review_complete_managed_validation_pending`.

- [x] Replace projected-read implementation-shape assertions with dirty, flushed, deep-chain, self-parent, missing-parent, and corrupt-cycle behavior checks.
- [x] Stream dirty world-matrix composition from child to parent with local-matrix pre-multiplication and no lineage `Vec`.
- [x] Replace the dirty active-chain `HashSet` with constant-space slow/fast parent traversal; keep self-parent and missing-parent as terminating edges and fail closed on multi-node cycles.
- [x] Add an ignored Windows-only Release integration profile with a real global allocator counter, raw latency samples, p50/p95/p99, and per-batch allocation evidence at depths 1, 32, and 1,024.
- [x] Record limits explicitly: dirty queries remain O(depth), Floyd adds parent probes, no latency threshold or measured result is available, and RSH-G19 / G24 remain open.
- [x] Preserve the frozen Batch U preimages: `derived_state.rs` SHA-256 `8224b647d0c087fab10c3aaf9f64e8d92b870a8ccaab82cf02cdccda8db91089`; `projected_reads.rs` SHA-256 `7595effcc55799c7bd0dd589c329c87f11f3ca934e96475e383f964a36785ddb`; `work_counters.rs` SHA-256 `4a8fb637262a979f865b2bdec44befe9ce23f55c3d50bae816461b0a4beb8678`.
- [x] Seal candidate hashes in `.codex/state/session-coordinator/async-validation-batches/2026-09-28-runtime1021-dirty-projected-read-source-manifest.json` and the reversible source/document patch in `docs/plans/optimize/zircon_runtime/62/2026-09-28-runtime1021-dirty-projected-read-streaming.inverse.patch`; leave Batch U attribution and coordinator lease receipts pending.
- [x] Complete scoped Rustfmt, tracked-file diff checks, direct newline/trailing-whitespace checks for all touched artifacts, and inverse-patch applicability check.
- [ ] Obtain managed compilation and run `scene::tests::derived_state::projected_reads` acceptance for the changed unit-test owner.
- [ ] Run the ignored profile with the managed Windows Release validator and retain raw latency/allocation output; pending evidence is not a pass.
- [ ] Compare latency against an equivalent baseline before making a performance-improvement claim.
- [ ] Close RSH-G19 only after generation-aware or equivalent O(1) ordinary reads are implemented and accepted; this slice only targets heap scratch removal.
- [ ] Complete RSH-G24's 1K/100K/1M raw work, allocation, and latency samples with explicit ceilings.

No Cargo command, coordinator status query, coordinator tooling action, commit, push, or external notification was performed for this slice.
