---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_runtime/06-platform-input-process-review.md
  - docs/plans/optimize/zircon_runtime/99zh-runtime-picking-pointer-ray-hit-hover-drag-drop-event-backend-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/406/2026-08-30-oit-compact-index-sort.md
---

# Runtime compile support repair

## Repairs

The platform host install path now retains an owned snapshot while returning the
same published value, matching the existing state publication path.

The hovered-hit capacity regression now exercises the real projection and checks
its complete output. The former untyped empty vector neither compiled nor tested
the production path. Ray-map benchmark IDs use the current `EntityId` alias, and
the OIT paired sampler infers its second generic argument.

Dynamic JSON conversion now handles general `List` and `Map` declarations through
the existing container conversion helpers with lossless JSON leaves. Named types
require a registered adapter and fail with a typed mismatch here. Regressions cover
mixed nested values, container shape mismatch, nested declarations and roundtrips.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M11 | Platform snapshot ownership, dynamic JSON conversion, hovered-hit projection regression, ray-map and OIT benchmark types | implemented_pending_validation | Static source/contracts and the 108-test runtime/editor batch pass; prior `4e5e00e217074a20be419299442bfafa` compile diagnosis was stale; refreshed managed Cargo batch is blocked by dirty external `E:/Git/zr_vm` (latest owner snapshot `87c112d27f25e2a10e2c078d61ef638954d0f8eb`, 112 + 88 = 200) |

No performance acceptance is claimed before the release benchmarks execute.
