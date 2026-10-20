---
related_code:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/framework/platform/mod.rs
  - zircon_runtime/src/core/framework/platform/runtime_target_mode.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface_markdown.py
implementation_files:
  - zircon_runtime/src/prelude.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface_markdown.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/index.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime/src/tests/runtime_absorption/root_surface.rs zircon_runtime/src/tests/runtime_absorption/mod.rs
  - root_surface_guard_static_passed static checks passed 2026-06-13
  - graphics_alias_block_removed_static_passed_cargo_pending static checks passed 2026-06-17
  - rhi_wgpu_root_backend_private_static_passed_cargo_pending static checks passed 2026-06-17
  - builtin_root_facade_removed_static_passed_cargo_pending static checks passed 2026-06-17
  - runtime_root_surface_markdown_split_static_passed_cargo_deferred_tests_deferred static checks passed 2026-06-21
  - builtin_helper_types_removed_from_prelude_static_pending added 2026-06-17
  - root_surface_interface_convergence_mirror_uses_current_audit_counts added 2026-06-14; Cargo pending active compile lanes
  - runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts added 2026-06-14; Cargo pending active compile lanes
  - rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_surface.rs passed 4/4 on 2026-06-13
  - git diff --check -- Runtime 02 root-surface scoped files: passed 2026-06-13 with LF-to-CRLF warnings only
doc_type: module-detail
---

# Runtime Root Surface

## Current Public Root Surface

`zircon_runtime/src/lib.rs` currently exposes 24 public module declarations and 22 distinct public namespaces. This is a namespace surface, not a flattened type surface. The public modules are `core`, `diagnostic_log`, `dynamic_api`, `engine_module`, `prelude`, `animation`, `asset`, `scene`, `text`, `ui`, `graphics`, `render_graph`, `rhi`, `builtin`, `foundation`, `input`, `navigation`, `operation`, `platform`, `plugin`, `script`, and target-only `browser`. The cross-domain `runtime_diagnostics` collector remains a feature-independent private support module so optional `graphics` and `dynamic_api` builds do not depend on one another.

The crate root has two public `pub use` sites:

- `crate::core::resource`
- `zircon_runtime_reflection_macros::{zircon_host_function, zircon_host_module, ZirconScriptType}`

The Runtime 02 builtin facade cutover removed root-level `RuntimeModuleLoadReport`, `RuntimeModuleLoadDiagnostic`, `RuntimePluginId`, and `RuntimeTargetMode` exports. These helper types are also excluded from `zircon_runtime::prelude` so the prelude cannot become a compatibility facade for retired root paths. Module assembly helpers, ids, and load reports use `zircon_runtime::builtin`; the later Frameworks05 hard cut gives `RuntimeTargetMode` its direct neutral owner at `zircon_runtime::core::framework::platform`. Neither crate-root forwarding nor a builtin target-mode alias is retained. The former dedicated required-missing DTO was deleted in the Frameworks 02 diagnostic hard cut; required-provider absence is represented only by structured plugin availability entries.

Subsystems such as `graphics`, `render_graph`, `rhi`, `ui`, `input`, `scene`, `asset`, and `plugin` are exposed as namespaces. They must not add flattened root `pub use` surfaces without updating Runtime 02 and its guard.

The concrete WGPU backend is owned by the separate `zr_rhi_wgpu` crate behind controlled runtime graphics/RHI imports. Neither internal `crate::rhi_wgpu` nor external `zircon_runtime::rhi_wgpu` root aliases exist.

Runtime 14 classifies `animation`, `navigation`, `operation`, and `text` as intentional crate-root runtime module-family seats. `animation` owns playback/evaluation and scene-hook application. `navigation` owns the built-in fallback runtime pathfinding implementation while advanced Recast/editor/baking behavior remains plugin-owned. `operation` owns the runtime operation registry used by module families and navigation plugins; `text` owns shared runtime typography contracts consumed by UI, graphics, editor, and product integrations.

## Core Spine

`zircon_runtime/src/core/mod.rs` is limited to the decided spine: `runtime`, `framework`, `manager`, `math`, and `resource`. The former root fragments `config_store`, `event_bus`, `frame_clock`, `job_scheduler`, `lifecycle`, `modules`, `state`, and `tasks` remain retired as root modules. Curated core facade exports may still point into `core::runtime` or `core::framework`, but the old root module names must not return.

## M3 Alias Cutover

Runtime 02 M3 removed the crate-private graphics alias block from `zircon_runtime/src/lib.rs`. The root no longer contains `pub(crate) use graphics::...`, `#[allow(unused_imports)]`, or root-level aliases for `RendererFeatureReferenceListKind`, `GraphicsError`, `SceneRenderer`, `WgpuRenderFramework`, `ViewportFrame`, `HybridGiRuntimeProvider`, `VirtualGeometryRuntimeProvider`, or `SolariRuntimeProvider`.

The current status anchor is `graphics_alias_block_removed_static_passed_cargo_pending`. Graphics and render callers must use `crate::graphics::...` or a narrower owner namespace directly. The root-surface audit now reports crate-visible graphics alias debt 0/0. This closes the M3 `lib.rs` graphics alias removal without editing active render production modules; package-level core/root/generated/export/app/editor/plugin validation remains pending under the Runtime 02 Cargo gate.

## Guard

`zircon_runtime/src/tests/contracts/root_surface/public_surface.rs` binds this document to Runtime 02 and the runtime index. It checks the crate root public module list, the two allowed public re-export sites, the absence of flattened subsystem `pub use` surfaces, the absence of private graphics alias debt, the removed M3.2 type-alias symbols, the removed builtin root facade, the absence of builtin helper type leakage through `prelude`, and the core spine root modules.

`root_surface_interface_convergence_mirror_uses_current_audit_counts` also binds the interface convergence review to the current root-surface audit facts: 24 raw public module declarations, 22 distinct namespaces, native target-client 21 public modules, 2 public `pub use` locations, 0 crate-visible graphics re-export symbols, concrete backend imports remain behind owner namespaces, builtin facade cutover complete, and M1 gate status `classified-and-clear`. It rejects the stale 17-module / 20-module / 3-public-use / migration-debt / 75-symbol / 80-symbol mirrors after the graphics alias, backend root-public, and builtin facade cutovers.

`runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts` binds this root-surface document to the wider Runtime 02 `core_spine_root_generated_boundary`: core root entries 6/6, core public modules 5/5, retired core root entries 0, runtime root public modules 21/21 (native target-client profile), public `pub use` sites 2/2, crate-visible graphics alias debt 0/0, root-surface M1 gate `classified-and-clear`, generated export templates 10/10, generated behavior 6/6, generated allowed adapters 6/6, generated migration debt 0/0, generated-code M1 gate `classified-and-clear`, root_entries guard tests 13, root_surface guard tests 6/6, generated-code guard tests 7/7, `guard_test_anchor_count = 26`, `missing_guard_test_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`.

`runtime_root_surface.py` remains the root public-surface audit and risk owner, while `runtime_root_surface_markdown.py` owns `render_runtime_root_surface_markdown`. The 2026-06-21 split separated audit ownership from Markdown rendering; the current source contract expects 24 raw public module declarations, 22 distinct public namespaces, 2 public `pub use` sites, 24 per-declaration module decisions, 2 public-use decisions, 0 unclassified root entries, 0 migration debt, 0 crate-visible graphics re-exports, M1 gate `classified-and-clear`, and `risks = []`.

`runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation` keeps the broader Runtime 02 validation lane visible after these static guards: core/root/generated/export_build_plan/app/editor/plugin checks and default lib-test reruns must still have evidence before Runtime 02 can be promoted.

## Declaration and cfg contract

The original 45 physical-line guard is retained as a failed historical layout
contract, not a passing result. The current root has 127 physical lines
(115 nonempty, 3270 bytes in this LF checkout). Its 28 external declarations
and 27 explicit path attributes occupy 55 separate declaration/attribute lines under the repository standard rustfmt layout.
The replacement keeps the physical metric and a strict 127-line ceiling.
It separately rejects all business implementations and unknown syntax.

Current source: 24 public module declarations, 22 distinct public namespaces,
28 external module declarations, 27 path attributes and 25 cfg attributes.
The two extra public declarations are mutually exclusive asset/plugin aliases.
Browser is a target-host-entry: target-web AND wasm32 AND NOT native-host.
The native target-client profile exposes 21 public namespaces; target-web on
wasm32 exposes 17. core-min exposes 4, asset-types 5, and portable graphics 14.
These are source contract expectations, not claims of compiled profile tests.

current root raw public declarations 24/24; distinct namespaces 22/22; native target-client public modules 21/21

The following identity table is shared by the Rust guard and Python audit.
Order, visibility, names, paths, cfg predicates and attachment are exact.
private means no visibility modifier, - means implicit module path,
and true means no cfg attribute. Only external semicolon modules are allowed.
The remaining permitted items are the existing exact self alias, dev-link
import and its allow attribute, native-host/wasm compile_error, test/windows
recursion cfg_attr, and the two curated public re-export sites. No function,
implementation, inline module, data declaration, include, or unknown macro,
attribute or import is permitted. All feature closures come from Cargo.toml.

<!-- root-declaration-identities:start -->
| visibility | name | path | cfg |
| --- | --- | --- | --- |
| pub(crate) | memory_validation | memory_validation.rs | all(test,feature="native-host",feature="memory-validation") |
| pub | core | core/mod.rs | true |
| pub | diagnostic_log | diagnostic_log/mod.rs | feature="diagnostic-log" |
| pub | dynamic_api | dynamic_api/mod.rs | feature="dynamic-api" |
| pub | engine_module | engine_module/mod.rs | true |
| pub | prelude | prelude.rs | true |
| private | runtime_diagnostics | runtime_diagnostics/mod.rs | true |
| pub | animation | animation/mod.rs | feature="animation" |
| pub | asset | asset/mod.rs | feature="native-host" |
| pub | asset | asset/portable.rs | all(feature="asset-types",not(feature="native-host")) |
| pub | scene | scene/mod.rs | any(feature="native-host",feature="graphics",feature="animation",feature="ui") |
| pub | text | text/mod.rs | feature="text" |
| pub | ui | ui/mod.rs | feature="ui" |
| pub | graphics | graphics/mod.rs | feature="graphics" |
| pub | render_graph | render_graph/mod.rs | feature="graphics" |
| pub | rhi | rhi.rs | feature="graphics" |
| pub | builtin | builtin/mod.rs | true |
| pub | foundation | foundation/mod.rs | any(feature="native-host",feature="graphics",feature="animation",feature="ui") |
| pub | input | input/mod.rs | any(feature="native-host",feature="graphics",feature="animation",feature="ui") |
| pub | navigation | navigation/mod.rs | feature="navigation" |
| pub | operation | operation/mod.rs | any(feature="native-host",feature="graphics",feature="animation",feature="ui") |
| pub | platform | platform/mod.rs | feature="native-host" |
| pub | plugin | plugin/mod.rs | feature="native-host" |
| pub | plugin | plugin/portable.rs | all(feature="graphics",not(feature="native-host")) |
| pub | script | script/mod.rs | feature="script" |
| pub(crate) | tests | tests/mod.rs | all(test,feature="native-host") |
| pub | browser | - | all(feature="target-web",target_arch="wasm32",not(feature="native-host")) |
| private | browser_input_validation | browser/input.rs | all(test,feature="ui",not(target_arch="wasm32")) |
<!-- root-declaration-identities:end -->

Validation is source_applied / managed_tests_pending. Grammar, schema, cfg,
profile or mirror violations prevent the audit from reporting classified-and-clear.
