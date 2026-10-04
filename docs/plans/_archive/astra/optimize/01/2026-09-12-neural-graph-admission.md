---
record_kind: milestone
status: implemented_pending_validation
created_at: 2026-09-12
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: Neural P0-05 bounded graph topology admission
session: astra-neural-graph-admission-20260912
---

# Neural P0-05 bounded graph topology admission

## Finding and scope

The Neural review identified that `.znn` model validation accepted forward
tensor reads, multiple producers for one tensor, and writes into Input/Weight
descriptors. That allowed a structurally decodable graph to reach CPU and GPU
paths with different late failures or aliasing behavior.

This slice stays below the full `ValidatedNnGraph` compiler boundary. It adds
the minimum shared admission invariant to `NnModelAsset::validate()` while
leaving operator arity, dtype/shape/broadcast inference, output reachability,
backend capability, and execution-plan ownership for the follow-up IR work.

## Implementation and tests

- `NnModelValidationError` now exposes typed failures for
  `TensorUsedBeforeProduced`, `InvalidOutputTensorKind`, and
  `DuplicateTensorProducer`.
- After all tensor-reference and attribute checks pass, validation walks the
  declared op order with one producer table. Input/Weight tensors remain legal
  sources; Intermediate/Output tensors must already have a producer before a
  read and may be produced only once. Zero-op weight-only resource models keep
  their existing admission behavior.
- `model_asset.rs` adds regressions for forward reads, duplicate producers,
  source writes, and the zero-op resource model.
- The GPU planner now rejects a `Reshape` whose source and destination have
  different element counts before recording a resource alias; `gpu_plan.rs`
  covers the mismatch as `NnGraphBuildError::InvalidShape`.

## Evidence boundary

- TDD RED source probe confirmed the regression names were present before the
  new error variants and topology pass existed.
- Post-edit topology/reshape source-contract probes, `rustfmt --edition 2021
  --check`, and scoped `git diff --check` pass (Git reports only the
  repository's LF-to-CRLF notice).
- An isolated `rustc --edition 2021 --emit=metadata` harness type-checks the
  edited validator against the surrounding model contracts.
- Existing dependency-independent Neural Python contracts pass: `21/21`.
- No Cargo, native, GPU, or product command ran. Managed Windows validation is
  still blocked by the dirty external `E:\\Git\\zr_vm` worktree.

Post-edit SHA-256 fingerprints:

```text
zircon_plugins/neural/runtime/src/model/validate.rs 5EC37E515696AB08023F8343934A4C8339610D45EDC343FC358F4C402B740767
zircon_plugins/neural/runtime/src/tests/model_asset.rs 54B4FDA876F385ED1E6BAA5CD4A77710830FF04C0A89D952002423327A6EC6C3
zircon_plugins/neural/runtime/src/gpu/graph_executor.rs B1539B3C553388AB43381657E04CB42B946F2957BC2D254F45307951F8415E04
zircon_plugins/neural/runtime/src/tests/gpu_plan.rs 080334A373EBB84E2721541C1C3BB02E5FB0BBD8219F3E018B24F1589B4C383F
```

This record does not claim full P0-05 closure or Windows acceptance. The
remaining compiled-IR and CPU/GPU parity work stays open under the Neural
review's M3/M4 path. No commit was created.
