---
record_kind: structural_maintenance_manifest
status: implementation_complete_secondary_review_findings_forward_fixed_second_review_complete_coordinator_staging_pending_managed_validation_pending
created_at: 2026-08-10
owner_plan: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
related_code:
  - zircon_runtime/src/ui/surface/render/cache.rs
  - zircon_runtime/src/ui/surface/render/cache/tests/mod.rs
  - zircon_runtime/src/ui/surface/render/cache/tests/update.rs
  - zircon_runtime/src/ui/surface/render/cache/tests/geometry_patch.rs
  - zircon_runtime/src/ui/surface/render/cache/tests/damage.rs
related_conventions:
  - docs/plans/engine-code-structure-convention.md
---

# Text09 UI Render Cache Test-Owner Structure Manifest

## Scope Delivered

`UiSurfaceRenderCache` is shared retained-render infrastructure and protects the Text layout
geometry-patch boundary. Its 846-line source file contained 327 lines of behavioral tests, which
violated the folder-backed test-owner rule. The implementation now stops at 519 lines and mounts
`cache/tests/mod.rs` explicitly under `#[cfg(test)]`. The behavior tree retains private parent
access through `super::*`: `update` owns reuse and compatibility, `geometry_patch` owns the four
patch/re-extract cases, and `damage` owns shared-frame deduplication.

The move is structural only. It does not change cache serialization, command reuse, damage
tracking, local re-extraction, or the rule that a command carrying `text_layout` rejects a
geometry-only patch rather than mutating the retained extract.

## Static Evidence

- `rustfmt --edition 2021 --check` passes for both owners.
- Scoped `git diff --check` passes; only the repository's existing CRLF notice is emitted.
- Static module checks confirm the explicit child mount, seven moved tests, the three behavior
  leaves, and retention of the text-layout rejection and shared-damage regressions.

The first independent review identified the untracked-child staging risk and the mixed behavior
leaf. The latter is forward-fixed by the three-leaf tree above. The required second independent
review found no P0/P2 and confirmed the exact module path, private imports, one-to-one behavior
coverage, and R4.2 ownership split. Its remaining P1 is the integration condition that every
listed file, including this untracked manifest, must be staged atomically with `cache.rs`; it is
recorded in the active session handoff and is not a code or validation failure. No Cargo, WGPU,
profiler, or framebuffer command was run. Managed validation remains separate from this
source-only completion record.
