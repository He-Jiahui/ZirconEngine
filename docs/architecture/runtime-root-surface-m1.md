---
related_code:
  - zircon_runtime/src/lib.rs
  - docs/architecture/runtime-reference-engine-evidence.md
  - docs/architecture/runtime-architecture-review-m0.md
  - docs/architecture/runtime-interface-convergence.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface_markdown.py
implementation_files:
  - docs/architecture/runtime-root-surface-m1.md
  - docs/architecture/runtime-architecture-review-m0.md
  - docs/architecture/runtime-interface-convergence.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface_markdown.py
plan_sources:
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - docs/architecture/runtime-reference-engine-evidence.md
tests:
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface_markdown.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - git diff --check -- .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py docs/architecture/runtime-root-surface-m1.md docs/architecture/runtime-architecture-review-m0.md docs/architecture/runtime-interface-convergence.md .codex/sessions/20260604-1232-runtime-architecture-review.md
doc_type: milestone-detail
---

# Runtime Root Surface M1 Gate

## Purpose

This document turns the M1 root-surface target into an auditable gate. The goal is not to remove every `pub mod` from `zircon_runtime/src/lib.rs`; the goal is to keep only stable facades and deliberate runtime module entry points at the root, while making migration debt explicit.

The reference-engine evidence is in `docs/architecture/runtime-reference-engine-evidence.md`:

- Bevy supports separating app composition, plugin contracts, schedules, and ECS query/cache owners.
- Fyrox supports keeping runtime engine ownership separate from editor state.
- Unreal supports explicit Runtime, Editor, Programs, plugin, and module boundaries.

## Current Gate Output

The structural audit now reports a `root_surface_audit.m1_gate_status` value. The current source contract requires the following target status; managed validation remains pending:

`classified-and-clear`

M1 gate status `classified-and-clear`

Current source contract expectations (managed validation pending):

- `public_module_count = 24`
- `public_use_location_count = 2`
- `module_decision_count = 24`
- `public_use_decision_count = 2`
- `unclassified_public_module_count = 0`
- `unclassified_public_use_location_count = 0`
- `root_surface_migration_debt_count = 0`
- `crate_visible_graphics_reexport_count = 0`

The current public module classification is:

- `stable-facade`: `core`, `dynamic_api`, `engine_module`, `prelude`
- `namespace-entry`: `builtin`, `plugin`
- `runtime-module-entry`: `animation`, `asset`, `diagnostic_log`, `foundation`, `input`, `navigation`, `operation`, `platform`, `scene`, `script`, `text`, `ui`
- `graphics-rhi-deferred`: `graphics`, `render_graph`, `rhi`
- `target-host-entry`: `browser`

The current root `pub use` classification is:

- `crate::core::resource`: stable facade for the runtime-internal resource spine.
- `zircon_runtime_reflection_macros`: stable developer-facing macro facade.

The contract requires an empty migration debt list and no grammar, cfg, profile, schema or mirror risk before classified-and-clear is possible.

Markdown rendering for this gate is split out to `runtime_root_surface_markdown.py`. `runtime_root_surface.py` remains the declaration/cfg/profile audit and risk owner, while the Markdown owner only formats the current `root_surface_audit` facts. The split preserved a dated audit output; the replacement source contract expects: 24 raw public module declarations, 22 distinct namespaces (native target-client has 21 public modules), 2 public `pub use` locations, 24 per-declaration module decisions, 2 public-use decisions, zero unclassified root entries, zero migration debt, zero crate-visible graphics re-exports, M1 gate `classified-and-clear`, and `risks = []`.

`rhi_wgpu` backend root public exposure is removed. The backend now lives in the separate `zr_rhi_wgpu` crate behind runtime graphics/RHI imports; direct external `zircon_runtime::rhi_wgpu` paths must not return.

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


## Current module-family source inventory

The current `module_family_boundary` inventory is `expected_family_count = 4`:
`animation = 29`, `navigation = 24`, `diagnostic_log = 39`, `engine_module = 9`.
The Runtime 14 plan and its fixed output record retain the dated 23/17/33/8
inventory independently. They remain historical evidence, not current scan output.
The navigation capacity owner is `runtime/world_scan/tests/capacity_tests.rs`.
The current navigation operation integration also mounts `operation/history_retention.rs`
under registration for pre-publication history admission, while `operation/tests/handler.rs`
is the handler's `cfg(test)` snapshot/retention contract owner. Neither moves Editor
history storage into Runtime or changes the crate-root fallback judgement.
The existing seven guard, eight animation JSON boundary, and five Cargo recipe
contracts remain required by their original owners. This inventory update is
`source_applied`; `managed_tests_pending`.


## Current root declaration contract

current root raw public declarations 24/24; distinct namespaces 22/22; native target-client public modules 21/21.
The source contract is 28 external declarations, 27 path attributes and 25 cfg
attributes, with two exact public re-export sites and a strict 127 physical-line
ceiling. Browser is a classified target-host-entry, reachable only with target-web
on wasm32 without native-host. Native asset/plugin aliases and their portable
alternatives are mutually exclusive. The retained runtime root public modules
21/21 snapshot refers to the native target-client profile; it is not the raw
all-cfg declaration count. The former 45-line guard actually failed on this
layout. Current guard/audit validation is source_applied / managed_tests_pending.
