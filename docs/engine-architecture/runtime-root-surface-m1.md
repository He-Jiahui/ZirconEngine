---
related_code:
  - zircon_runtime/src/lib.rs
  - docs/engine-architecture/runtime-reference-engine-evidence.md
  - docs/engine-architecture/runtime-architecture-review-m0.md
  - docs/engine-architecture/runtime-interface-convergence.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
implementation_files:
  - docs/engine-architecture/runtime-root-surface-m1.md
  - docs/engine-architecture/runtime-architecture-review-m0.md
  - docs/engine-architecture/runtime-interface-convergence.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/sessions/20260604-1232-runtime-architecture-review.md
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - docs/engine-architecture/runtime-reference-engine-evidence.md
tests:
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - git diff --check -- .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py docs/engine-architecture/runtime-root-surface-m1.md docs/engine-architecture/runtime-architecture-review-m0.md docs/engine-architecture/runtime-interface-convergence.md .codex/sessions/20260604-1232-runtime-architecture-review.md
doc_type: milestone-detail
---

# Runtime Root Surface M1 Gate

## Purpose

This document turns the M1 root-surface target into an auditable gate. The goal is not to remove every `pub mod` from `zircon_runtime/src/lib.rs`; the goal is to keep only stable facades and deliberate runtime module entry points at the root, while making migration debt explicit.

The reference-engine evidence is in `docs/engine-architecture/runtime-reference-engine-evidence.md`:

- Bevy supports separating app composition, plugin contracts, schedules, and ECS query/cache owners.
- Fyrox supports keeping runtime engine ownership separate from editor state.
- Unreal supports explicit Runtime, Editor, Programs, plugin, and module boundaries.

## Current Gate Output

The structural audit now reports a `root_surface_audit.m1_gate_status` value. Current status is:

`classified-and-clear`

M1 gate status `classified-and-clear`

Current numeric evidence:

- `public_module_count = 19`
- `public_use_location_count = 2`
- `module_decision_count = 19`
- `public_use_decision_count = 2`
- `unclassified_public_module_count = 0`
- `unclassified_public_use_location_count = 0`
- `root_surface_migration_debt_count = 0`
- `crate_visible_graphics_reexport_count = 0`

The current public module classification is:

- `stable-facade`: `core`, `dynamic_api`, `engine_module`, `prelude`
- `namespace-entry`: `builtin`, `plugin`
- `runtime-module-entry`: `animation`, `asset`, `diagnostic_log`, `foundation`, `input`, `navigation`, `platform`, `scene`, `script`, `ui`
- `graphics-rhi-deferred`: `graphics`, `render_graph`, `rhi`

The current root `pub use` classification is:

- `crate::core::resource`: stable facade for the runtime-internal resource spine.
- `zircon_runtime_reflection_macros`: stable developer-facing macro facade.

The current migration debt list is empty.

`rhi_wgpu` backend root public exposure is removed. The backend module is now a crate-private owner behind `rhi`; direct external `zircon_runtime::rhi_wgpu` paths must not return.

The builtin root facade exposure is removed. Runtime assembly helper types such as `RuntimeTargetMode` and `RuntimePluginId` stay under `zircon_runtime::builtin` instead of the crate root or `zircon_runtime::prelude`.

The current M3 closure anchor is `crate-visible graphics re-export fan-out removed by Runtime 02 M3`; the root-surface audit must keep `crate_visible_graphics_reexport_count = 0`.

## M1 Decision Rules

Stable facades are allowed only when they expose the converged runtime spine, descriptor contracts, ABI surface, macro facade, or a deliberately curated prelude.

Runtime module entries are allowed only when they are the current owner namespace for absorbed built-in modules. They must not become a dumping ground for cross-module construction.

Runtime 14 classifies `animation` and `navigation` as deliberate runtime module-family root seats. `animation` owns runtime playback/evaluation and scene-hook application, while `navigation` owns the built-in fallback runtime pathfinding implementation used when the external Recast-backed plugin stack is not linked. They are runtime module entries, not unclassified root modules.

The `plugin` root namespace is allowed, but root-level flattening of native loader or ABI symbols is not allowed. Native loader exposure remains an M4 blocker until the plugin ecosystem session settles.

Graphics and RHI root entries are deferred because active RHI/WGPU work is still changing backend contracts. The deferred status is not approval to add more root exports.

Backend-specific modules such as `rhi_wgpu` are not root public modules. Future backend selection should stay behind explicit RHI/backend-owned namespaces rather than reviving concrete backend exposure at the runtime crate root.

Broad root `pub use` groups must not return. New public contracts should land in owner modules first, then be considered for a curated facade only if the reference evidence supports root exposure. The curated prelude must not re-export owner-specific helper types solely to preserve retired root paths.

## Required Follow-Up

Before editing `zircon_runtime/src/lib.rs`, run the structural audit and inspect:

- `root_surface_audit.module_decision_groups`
- `root_surface_audit.public_module_count`
- `root_surface_audit.public_use_location_count`
- `root_surface_audit.public_use_decisions`
- `root_surface_audit.root_surface_migration_debt`
- `root_surface_audit.root_surface_migration_debt_count`
- `root_surface_audit.unclassified_public_modules`
- `root_surface_audit.unclassified_public_module_count`
- `root_surface_audit.unclassified_public_use_locations`
- `root_surface_audit.unclassified_public_use_location_count`

Any new unclassified public module or public use is a review blocker. It must be classified in `runtime_root_surface.py` with a clear owner reason or moved behind an existing owner module.

The Runtime 02 M3 cut removed the crate-visible graphics re-export fan-out from `zircon_runtime/src/lib.rs`; future graphics changes must keep that count at 0. The follow-up backend cut made `rhi_wgpu` crate-private, and the builtin facade cutover removed the last broad root pub-use group. Future RHI/WGPU work must keep concrete backend exposure behind `rhi` or narrower owner namespaces. The generated-code boundary remains a separate M1/M2 track and should not be mixed into the graphics cut.
