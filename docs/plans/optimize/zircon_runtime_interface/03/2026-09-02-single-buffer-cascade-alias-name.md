record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-02
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/design_tokens/cascade_registry.rs
related_tests:
  - zircon_runtime_interface/src/ui/design_tokens/cascade_registry/custom_property_alias_performance_tests.rs
  - tools/tests/test_runtime_interface03_cascade_alias_name_performance_contract.py
  - zircon_runtime_interface/src/ui/design_tokens/cascade_registry/custom_property_alias_performance_tests.rs::runtime_interface03_batch72_73_single_buffer_cascade_alias_name_release_benchmark
---

# Single-buffer cascade alias name encoding

## Scope

CSS custom-property aliases previously evaluated `canonical_name.replace('.', "-")` before
formatting the required `--` prefix. That created a full-length temporary `String` for every
canonical design-token key in addition to the final alias allocation. Alias name encoding now
reserves the exact final byte capacity and appends dot-delimited UTF-8 slices directly into the
destination buffer. Borrowed BTree key projection, reference values, ordering, empty components,
and non-ASCII token bytes are unchanged.

## Verification

- TDD RED: the focused contracts failed because the single-buffer encoder and release benchmark
  marker did not exist and alias projection still used `replace` plus `format`.
- Focused Batch72-73 static performance contracts after implementation: `6/6` passed.
- Full RuntimeInterface03 static performance contracts: `156/156` passed.
- Input-routing receipt and editor asset-palette companion contracts: `12/12` passed.
- Combined local static regression: `168/168` passed.
- Rust behavior coverage compares the new encoder with the former replacing implementation for
  ordinary, repeated-dot, leading/trailing-dot, empty, and Unicode names.
- Rust 1.94.1 formatting, Python compile checks, and scoped diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

The combined Batch72-73 managed request `runtime-interface03-batch72-73-20260902-r1` was rejected
before ticket creation or Cargo launch with `validation_ticket_external_worktree_dirty` for external
repository `E:\Git\zr_vm`. No managed compile, behavior, benchmark, or terminal performance receipt
exists for this batch.

Copy-complete ownership receipt: exact-path lease request
`916a92ee14eb44d1a7cc2d00c42a8985`; baseline attribution request
`d8d29e229cd744c29dc04369e1a32f47` (`attributed`). The union includes the pre-existing borrowed
cascade-alias and stack color-token encoding tests, guards, and records required by the shared
`cascade_registry.rs` blob.

No commit, push, or WeCom notification is permitted until managed validation is terminal-successful
and the coordinator finalizes the attributed union.

## Performance contract

The ignored release benchmark encodes 4,096 canonical names 64 times over 11 alternating samples.
It compares the former `replace` plus `format` construction with the single-buffer encoder and
requires at least 20% P95 improvement. Exact terminal nanosecond values must come from the managed
Windows receipt before integration, push, or WeCom reporting.
