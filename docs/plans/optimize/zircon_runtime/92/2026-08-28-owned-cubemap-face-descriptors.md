---
title: Runtime92 Owned Cubemap Face Descriptors
category: zircon_runtime
report_id: Runtime92-owned-cubemap-face-descriptors-2026-08-28
date: 2026-08-28
session_id: root-runtime92-owned-array-layer-descriptors-20260828
implementation_status: implementation_complete
validation_status: managed_validation_queued
---

# Runtime92 Owned Cubemap Face Descriptors

## Scope

`texture_asset_from_cubemap_faces` owns all six decoded `TextureAsset` values, but the previous
validation path borrowed them through `render_image_descriptor`. That cloned the full descriptor,
including its format string, usage vectors, asset-usage vectors, sampler, and metadata. The first
face was cloned once to establish expected values and then cloned again inside the six-face loop,
for seven complete descriptor clones per cubemap assembly.

Cubemap assembly now takes each owned `Option<TextureAssetDescriptor>` with `Option::take` and
normalizes it through the same `into_render_image_descriptor` conversion. The first face retains
the original shape-first error ordering and is projected once; the remaining loop begins at face
one. Missing descriptors still use `TextureAssetDescriptor::from_payload`, while mip-major payload
interleaving, square-face checks, format/color propagation, and cube extent behavior remain
unchanged. Diagnostic strings are cloned only when constructing a format mismatch error.

## Performance Evidence

The isolated Rust model processes 4,096 six-face cubemaps whose descriptors contain a format
string, two usage vectors, and metadata bytes. It compares the former seven descriptor clones per
cubemap with the final six ownership moves. Allocations are measured by a scoped global allocator.
Each variant uses 21 paired samples after three warmups and was compiled with
`rustc --edition 2021 -C opt-level=3` on Windows.

| Metric | Cloned descriptors | Owned descriptors | Change |
|---|---:|---:|---:|
| Allocator calls | 114,688 | 0 | -100.000% |
| Requested bytes | 4,214,784 | 0 | -100.000% |
| P50 | 9,026,800 ns | 5,342,200 ns | -40.818% |
| P95 | 13,077,800 ns | 6,885,300 ns | -47.351% |

The baseline and optimized checksums both remained `236,085,248`. The acceptance gates were zero
descriptor-projection allocations and requested bytes, with P50 and P95 at or below 65% of
baseline; all four passed.

Model source:

- `.codex/state/session-coordinator/runtime92-owned-cubemap-face-descriptors-model.rs`

The model isolates descriptor projection and face validation traversal. It does not replace managed
Cargo behavior tests, payload interleave profiling, or product-scale cubemap import measurements.

## Contracts And Validation

- `tools/tests/test_runtime92_owned_cubemap_face_descriptors_performance_contract.py` locks the
  owned mutable input, first-face single projection, remaining-face traversal from index one,
  `Option::take`, payload fallback, and absence of cloning `render_image_descriptor` calls.
- Initial TDD RED failed because cubemap assembly accepted immutable faces and cloned render
  descriptors; the final implementation passes all four source-contract tests.
- Scoped `rustfmt --edition 2021 --check`, direct Python contract execution, and
  `git diff --check` pass.
- A focused Rust regression now verifies cubemap face order and six-face output extent.
- Cargo compilation and focused cubemap behavior are submitted with the array-layer task in one
  managed asynchronous coordinator batch; no direct Cargo command was run.

## Recovery Batch 2026-08-31

- Ownership transfer apply: `ae5e7b260c914c82900ec2aa145c9a08`.
- Evidence paths transfer apply: `c3b30c6c260c4f9d90b7d001597ff53a`.
- Focused behavior tests share the `runtime92_owned_descriptors_recovery_batch_` filter with the
  array-layer task. Public release model: `tools/runtime92_owned_cubemap_face_descriptors_model.rs`.
- Managed batch script: `tools/zircon-validation-runtime92-owned-descriptors-recovery-batch.ps1`.
- Coordinator ticket: `pending_submission`; terminal allocations, bytes, P50/P95, and checksum are
  authoritative before record closeout.

## Remaining Parent-Plan Work

Runtime92 still requires the parent plan's texture schema, semantic artifact blocks, asynchronous
upload/generation ownership, complete residency and budget accounting, compressed partial
residency, virtual texture pipeline, device-loss handling, and product qualification. This slice
only removes redundant descriptor cloning from existing RGBA8 cubemap assembly.
