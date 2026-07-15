# Render 18 M3 volumetric fog component id public export closeout

Plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
Milestone: M3
Status: completed
Files: ["docs/plans/zircon_runtime/render/18/2026-07-15-m3-volumetric-fog-component-id-public-export-closeout.md","zircon_runtime/src/core/framework/render/advanced_lighting/mod.rs","zircon_runtime/src/core/framework/render/mod.rs"]
Date: 2026-07-15

## Scope delivered

- Preserved `advanced_lighting/volumetric.rs` as the single definition owner for `VOLUMETRIC_FOG_COMPONENT_ID`.
- Re-exported the existing component id through the private `advanced_lighting` boundary and the public `core::framework::render` facade.
- Kept both `mod.rs` files structural: no alias, duplicate definition, compatibility shim, or implementation logic was added.

## Fresh testing evidence

- `rustfmt --edition 2021 --config skip_children=true --check` passed for both touched facade files.
- `git diff --check` passed for both touched facade files.
- Managed Windows job `31cb6b6e3a8c4ee4930cec0f000695e1` ran `navigation_tick_mirror_` in `zircon_plugin_navigation_runtime`: 1 passed, 0 failed. This job had previously failed at compile time because `crate::core::framework::render::VOLUMETRIC_FOG_COMPONENT_ID` was unavailable.
- The plugin-structure audit remained clean: manifest, registration, capability, editor/runtime mirror, compatibility-shim, and skeleton migration-debt counts are zero.

## Review

- Independent review reported 0 Critical and 0 Important findings.
- The reviewer confirmed that the two-level re-export is the minimum complete facade path and that the internal `advanced_lighting` and `volumetric` modules remain private.

## Status and completed items

| Milestone | Item | Status | Evidence |
|---|---|---|---|
| M3 | Volumetric fog component id owner | completed | The definition remains unique in `advanced_lighting/volumetric.rs`. |
| M3 | Framework public facade | completed | Both required re-export layers expose the existing id without aliases or shims. |
| M3 | Focused Navigation consumer gate | completed | Managed Windows Navigation runtime test passed 1/1. |
| M3 | Independent review | completed | 0 Critical, 0 Important. |
