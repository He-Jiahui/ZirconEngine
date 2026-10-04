---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09b/2026-08-26-particle-upload-linear-difference.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/graphics/visibility/planning/build_particle_upload_plan.rs
tests:
  - zircon_runtime/src/graphics/visibility/planning/build_particle_upload_plan.rs
---

# Runtime955 · particle-upload linear difference

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09b particle upload planning | Current and previous ascending unique emitter vectors are diffed with a two-pointer linear helper. The temporary ordered membership trees and probes are gone; current, dirty, removed, and full-rebuild output ordering is preserved. | In-file behavior/source contracts cover emitter, dirty, removed, and helper ownership. Current-source Release owner evidence measured `legacy_p95_ns=11,222,500` versus `optimized_p95_ns=406,800` (`96.38%` reduction); managed Cargo, allocator, and product gates remain pending. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/graphics/visibility/planning/build_particle_upload_plan.rs` | `94D645C650C64D8B94DC0195197A3F17A0AAF19B4FED63B9A77955F0DA1C866D` |

## Validation handoff

The owner is included in the grouped Runtime Release selector. Managed Cargo,
Release P95, allocator, and product gates remain pending; tooling remains
deferred.
