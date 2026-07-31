---
related_code:
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material/material_readiness.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material/tests.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material/material_readiness.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material/tests.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
reference_sources:
  - dev/bevy/crates/bevy_asset/src/processor
  - dev/bevy/crates/bevy_asset/src/server
  - dev/Fyrox/fyrox-impl/src/resource
tests:
  - prepared_material_dependency_cache_uses_registry_identity_and_revision
  - prepared_material_cache_identity_rejects_revision_or_upload_support_change
  - Runtime 15 production file budget static scan
  - scoped rustfmt --edition 2021 --check
  - scoped git diff --check
doc_type: milestone-detail
status: implemented_validation_pending
---

# Frameworks06 Material Streamer Production Budget Hard Cut Batch38

Plan: `docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md`
Milestone: M1 follow-up / production module boundary convergence
Status: implemented_validation_pending
Date: 2026-07-22

## Scope Delivered

| Slice | Status | Evidence |
|---|---|---|
| Prepared material cache identity | implemented | `PreparedMaterial` stores the `TextureUploadSupport` used to prepare its GPU texture dependencies; a support-policy change invalidates the cache. |
| Material preparation orchestration owner | implemented | The production file retains material preparation, parent resolution, texture dependency snapshots, and readiness orchestration. |
| Material readiness helpers | implemented | Pure readiness, fallback, dependency-currentness, and texture-slot classification live in folder-backed `material_readiness.rs`. |
| Cache regression test owner | implemented | The registry identity/revision regression moved unchanged to folder-backed `resource_streamer_ensure_material/tests.rs`. |
| Target production-file budget | implemented | Canonical `source.lines().count()` reports 742 lines for `resource_streamer_ensure_material.rs`, below the 800-line limit. |
| Global production-file budget | pending | The canonical scan still reports two foreign owners: `scene_renderer/ui/render.rs` at 835 lines and `ui/surface/render/resolve.rs` at 859 lines. |
| Legacy layout | removed | The production implementation no longer embeds a test implementation block; no compatibility module or forwarding test owner was added. |
| Managed Cargo acceptance | pending | Focused Runtime material/resource-streamer tests must pass on an immutable validation source before this record becomes accepted. |

## Architecture Decision

Resource streaming is a long-lived graphics subsystem. Its GPU/asset orchestration, pure material
readiness policy, and regression fixtures therefore use separate folder-backed owners, matching
the repository's existing resource-streamer subtree and the resource-domain layouts used by Bevy
and Fyrox. The split changes no production API or runtime behavior.

The current source already contained the material dependency cache correction that compares
registry identity, revision, and texture upload support while deduplicating texture preparation.
This batch preserves that implementation byte-for-byte while moving its focused regression into
the test owner. `texture_support` is authoritative cache identity, not a compatibility field or a
fallback to the previous revision-only cache key. Acceptance must cover that behavior; the
structure-only change does not claim it passed Cargo by inspection.

## Static Evidence

- The initial PowerShell triage used `Measure-Object -Line`; review found that counter ignored blank
  lines and therefore was not equivalent to Runtime15's canonical `source.lines().count()` guard.
- Canonical post-change count for the target production owner: 742 lines, below the 800-line limit.
- Canonical full Runtime scan remains RED with two paths outside this exact5 manifest:
  `graphics/scene/scene_renderer/ui/render.rs` (835) and `ui/surface/render/resolve.rs` (859).
- The eight extracted helper definitions have zero residual definitions in the orchestration file
  and exactly one definition each in `material_readiness.rs`.
- Rust 1.94.1 scoped `rustfmt --edition 2021 --check` passed.
- Exact4 `git diff --check` passed.
- No old module alias, shim, or compatibility re-export is introduced.

## Remaining Acceptance

The record stays `implemented_validation_pending` until independent review and managed focused
Runtime validation complete. The two foreign global-budget owners require separate leased
follow-ups. Frameworks06 and Runtime15 remain open; this record does not claim either parent plan
complete or the global production-file gate GREEN.
