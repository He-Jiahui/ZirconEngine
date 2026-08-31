---
title: Runtime92 Owned Array Layer Descriptors
category: zircon_runtime
report_id: Runtime92-owned-array-layer-descriptors-2026-08-28
date: 2026-08-28
session_id: root-runtime92-owned-array-layer-descriptors-20260828
implementation_status: implementation_complete
validation_status: managed_validation_queued
---

# Runtime92 Owned Array Layer Descriptors

## Scope

`texture_asset_from_array_layers` owns its `Vec<TextureAsset>`, but the previous validation path
borrowed each layer and called `render_image_descriptor`. That API clones the full texture
descriptor, including its format string, usage vectors, asset-usage vectors, sampler, and metadata.
The first layer was cloned once to establish expected fields and then cloned again in the all-layer
loop, producing `layer_count + 1` full descriptor clones before payload interleaving.

Array assembly now takes each owned `Option<TextureAssetDescriptor>` with `Option::take`, normalizes
it through the same `into_render_image_descriptor` conversion, and validates it before moving to the
next layer. The first layer is projected and validated once; remaining validation begins at layer
one. Missing descriptors still use `TextureAssetDescriptor::from_payload`, and validation error
ordering, mip-major payload interleaving, format/color propagation, and array extent behavior remain
unchanged.

## Performance Evidence

The isolated Rust model uses 4,096 owned layer descriptors with a format string, two usage vectors,
and metadata bytes. It compares the former first-plus-all descriptor clones with the final
first-plus-remaining `Option::take` path. Allocations are measured by a scoped global allocator.
Each variant uses 21 paired samples after three warmups and was compiled with
`rustc --edition 2021 -C opt-level=3` on Windows.

| Metric | Cloned descriptors | Owned descriptors | Change |
|---|---:|---:|---:|
| Allocator calls | 16,388 | 0 | -100.000% |
| Requested bytes | 606,356 | 0 | -100.000% |
| P50 | 1,096,300 ns | 618,400 ns | -43.592% |
| P95 | 1,656,100 ns | 786,100 ns | -52.533% |

The baseline and optimized checksums both remained `33,730,635`. The acceptance gates were zero
descriptor-projection allocations and requested bytes, with P50 and P95 at or below 65% of
baseline; all four passed.

Model source:

- `.codex/state/session-coordinator/runtime92-owned-array-layer-descriptors-model.rs`

The model isolates descriptor projection and validation traversal. It does not replace managed
Cargo behavior tests, payload interleave profiling, or product-scale texture import measurements.

## Contracts And Validation

- `tools/tests/test_runtime92_owned_array_layer_descriptors_performance_contract.py` locks the
  owned mutable input, first-layer single projection, remaining-layer traversal from index one,
  `Option::take`, payload fallback, and absence of cloning `render_image_descriptor` calls.
- Initial TDD RED failed because array assembly accepted immutable layers and cloned render
  descriptors; the final implementation passes all four source-contract tests.
- Scoped `rustfmt --edition 2021 --check`, direct Python contract execution, and
  `git diff --check` pass.
- A focused Rust regression now verifies array payload order and output layer extent.
- Cargo compilation and focused texture-array behavior are submitted with the cubemap task in one
  managed asynchronous coordinator batch; no direct Cargo command was run.

## Recovery Batch 2026-08-31

- Ownership transfer apply: `ae5e7b260c914c82900ec2aa145c9a08`.
- Evidence paths transfer apply: `c3b30c6c260c4f9d90b7d001597ff53a`.
- Focused behavior tests share the `runtime92_owned_descriptors_recovery_batch_` filter with the
  cubemap task. Public release model: `tools/runtime92_owned_array_layer_descriptors_model.rs`.
- Managed batch script: `tools/zircon-validation-runtime92-owned-descriptors-recovery-batch.ps1`.
- Coordinator ticket: `pending_submission`; terminal allocations, bytes, P50/P95, and checksum are
  authoritative before record closeout.

## Remaining Parent-Plan Work

Runtime92 still requires the parent plan's texture schema, semantic artifact blocks, asynchronous
upload/generation ownership, complete residency and budget accounting, compressed partial
residency, virtual texture pipeline, device-loss handling, and product qualification. This slice
only removes redundant descriptor cloning from existing RGBA8 array assembly.
