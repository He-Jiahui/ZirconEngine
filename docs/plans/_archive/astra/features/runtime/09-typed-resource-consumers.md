---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
---

# Typed resource consumer repair

## Repairs

Texture cooking and cubemap validation use the canonical `depth_or_array_layers`
after restricting the image dimension to D2/Cube. Removed duplicate descriptor
layer comparisons without restoring the retired descriptor accessor.

Dynamic component diagnostics read plugin ownership from `ReflectTypePath`.
Asset migration explicitly maps durable artifact identity exhaustion to a typed,
transparent `AssetMigrationError` variant. It preserves the original error without
inventing a transaction path; a regression checks the mapping.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M12 | Canonical texture extent, reflection plugin owner and typed exhaustion error | implemented_pending_validation | Diagnosed from run `4e5e00e217074a20be419299442bfafa`; include with M10/M11 and existing support filters |

Tests and performance gates remain pending until the combined release batch runs.
