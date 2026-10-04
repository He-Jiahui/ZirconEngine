---
record_kind: milestone
status: implemented_pending_validation
created_at: 2026-09-18
plan: docs/plans/optimize/zircon_plugins/02-neural-model-onnx-inference-post-process-editor-product-integration-review.md
milestone: Neural P1-50 checked GPU dispatch batch/channel admission
session: astra-neural-dispatch-admission-20260919
---

# Neural P1-50 checked GPU dispatch batch/channel admission

## Finding and scope

The Neural GPU planner used `saturating_mul(batch, channels)` for the third
dispatch dimension of Conv, Pool, and Upsample passes. A valid tensor shape
whose batch/channel product exceeded `u32::MAX` therefore became a silently
clamped `u32::MAX` dispatch instead of a typed admission failure. The planner
now also accepts an optional per-axis device limit and rejects fixed dispatch
groups that exceed it before pass descriptors are published. Device probing,
real buffer allocation, shader compilation, and submission remain separate
P1-45/P1-51 work.

## Implementation and tests

- `NnGraphBuildError::DispatchDimensionOverflow` carries the operator and the
  batch/channel operands that could not be represented by the dispatch ABI.
- Conv2d, MaxPool2d/AvgPool2d, and Upsample2d now use one checked helper for
  their batch/channel dispatch dimension; no saturating product remains in the
  GPU planner.
- `NnGraphExecutor::with_dispatch_limits` carries the device's three dispatch
  group limits into plan admission. `DispatchDimensionLimitExceeded` reports
  the operator, computed groups, and configured limits; the default executor
  remains backward-compatible and defers device checks to the render path.
- `gpu_plan.rs` constructs bounded descriptor-only models with
  `batch == channels == 65_536` and verifies the typed error for all three
  planner paths without allocating tensor-sized storage.
- A focused pool regression configures `[3, 3, 3]` limits and rejects the
  computed `[4, 4, 1]` dispatch before descriptor creation.
- Existing dirty changes in the leased files (weight buffer ranges and
  Reshape element-count admission) were preserved; this slice only adds the
  overflow contract.

## Evidence boundary

- TDD RED was established by adding the three typed-error expectations before
  the new error variant/helper existed.
- `rustfmt --edition 2021 --check`, scoped `git diff --check`, the existing
  Neural stack-bounded admission suite (`7/7`), and a focused source contract
  confirming three checked helper call sites and zero `saturating_mul` calls
  pass.
- The managed Windows validation attempt used
  `validate-matrix.ps1 -Package zircon_plugin_neural_runtime -LibTests
  -TestFilter nn_graph_executor_rejects_batch_channel_dispatch_overflow` and
  was rejected before Cargo with `unmanaged_artifacts_detected` for the
  coordinator-reserved `D:\ZirconBuilds\mvp-test-fixtures-28916` path. The
  external `E:\Git\zr_vm` worktree is also still dirty. No Cargo, native GPU,
  shader, or product acceptance is claimed.

The current coordinator artifact audit still reports that same unregistered
fixture path. A dry-run for the new device-limit regression constructs the
expected check/test commands, but no duplicate real Cargo request was submitted
while the admission blocker is unchanged.

Post-edit SHA-256 fingerprints:

```text
zircon_plugins/neural/runtime/src/gpu/graph_executor.rs 8867744AB1289F0F8F92A3A2EFFEE4D5144625EDFA11FB50AEA4740CCBDE40FF
zircon_plugins/neural/runtime/src/tests/gpu_plan.rs 32C3CE74CF8D4ACB3364DC7A02DE8918E4DFD3A29A1FE34510D57B896877DBD2
```

This record does not claim full P1-50 closure or Windows acceptance. No commit
was created.
