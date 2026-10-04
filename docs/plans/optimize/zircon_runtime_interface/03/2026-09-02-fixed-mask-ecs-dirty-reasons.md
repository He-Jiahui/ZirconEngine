record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-02
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/ecs/compute.rs
related_tests:
  - zircon_runtime_interface/src/ui/ecs/compute/dirty_reason_performance_tests.rs
  - tools/tests/test_runtime_interface03_ecs_dirty_reason_mask_performance_contract.py
  - zircon_runtime_interface/src/ui/ecs/compute/dirty_reason_performance_tests.rs::runtime_interface03_batch72_73_fixed_ecs_dirty_reason_mask_release_benchmark
---

# Fixed-mask ECS dirty reason aggregation

## Scope

Full ECS schedule-impact projection previously maintained one `BTreeSet<UiPipelineDirtyReason>` per
pipeline-stage bucket. The reason domain contains exactly 15 enum variants, so each node-stage
admission paid ordered-tree lookup and node-allocation costs for a fixed, tiny key space. Buckets
and focused stage projection now aggregate into a 16-bit mask and materialize the final owned vector
once. The explicit output table preserves the former derived-`Ord` declaration order, uniqueness,
stage admission, and final receipt shape.

## Verification

- TDD RED: the focused contracts failed while production still referenced
  `BTreeSet<UiPipelineDirtyReason>` and had no fixed-mask benchmark module.
- Focused Batch72-73 static performance contracts after implementation: `6/6` passed.
- Full RuntimeInterface03 static performance contracts: `156/156` passed.
- Input-routing receipt and editor asset-palette companion contracts: `12/12` passed.
- Combined local static regression: `168/168` passed.
- Existing production behavior coverage still asserts canonical schedule ordering, reason ordering,
  and node deduplication; the new oracle test also compares mask output with the former BTreeSet for
  4,096 fully dirty nodes.
- Rust 1.94.1 formatting, Python compile checks, and scoped diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

The combined Batch72-73 managed request `runtime-interface03-batch72-73-20260902-r1` was rejected
before ticket creation or Cargo launch with `validation_ticket_external_worktree_dirty` for external
repository `E:\Git\zr_vm`. No managed compile, behavior, benchmark, or terminal performance receipt
exists for this batch.

Copy-complete ownership receipt: exact-path lease request
`916a92ee14eb44d1a7cc2d00c42a8985`; baseline attribution request
`d8d29e229cd744c29dc04369e1a32f47` (`attributed`).

No commit, push, or WeCom notification is permitted until managed validation is terminal-successful
and the coordinator finalizes the attributed union.

## Performance contract

The ignored release benchmark aggregates eight active dirty reasons across 4,096 fully dirty nodes
64 times over 11 alternating samples. It compares the former BTreeSet with the fixed mask and
requires at least 20% P95 improvement. Exact terminal nanosecond values must come from the managed
Windows receipt before integration, push, or WeCom reporting.
