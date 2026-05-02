# Engine Remaining Milestones Update Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` or `superpowers:executing-plans` to implement this plan task-by-task. In `zirconEngine`, follow milestone-first cadence: implementation slices first, testing stage at milestone boundary.

**Goal:** Maintain one global roadmap for the remaining unfinished engine work across UI, Runtime/Editor, pluginization, SRP/RHI, GI/VG, Physics/Animation, SDF fonts, and workspace validation.

**Architecture:** This document is the status-and-routing authority only; it references existing subsystem plans and active sessions without replacing them. Each milestone keeps its implementation owner in the existing runtime/editor/plugin/render/UI subsystem plan, while this roadmap fixes ordering, completion labels, blocker ownership, and broad validation criteria.

**Tech Stack:** Rust, Cargo, `zircon_app`, `zircon_runtime`, `zircon_editor`, `zircon_runtime_interface`, independent `zircon_plugins` workspace, `.codex/plans`, `.codex/sessions`, `docs/superpowers/plans`, `docs/engine-architecture`, `docs/ui-and-layout`, `docs/assets-and-rendering`.

---

## Status Labels

| Label | Meaning |
| --- | --- |
| `完成` | Has code owner, focused validation, and archive evidence; may still be a dependency for later milestones. |
| `部分完成` | Foundation or scoped path is accepted, but productization, migration, broad validation, or dual-host closure remains. |
| `缺失` | Has only design/planning evidence or no production implementation evidence. |
| `需迁移` | Old owner paths, source-level dependencies, static built-ins, compatibility naming, or shim-like boundaries still need hard cutover. |

## Current Global Matrix

| Order | Milestone | Status | Current Evidence | Remaining Work |
| --- | --- | --- | --- | --- |
| 0 | Broad Workspace Validation Triage | 部分完成 | `zircon_editor --lib` scoped closeout passed with `876 passed; 0 failed; 1 ignored`; runtime package still blocked by graphics/plugin `ViewportRenderFrame` public-surface drift. | Clear graphics/plugin runtime blocker, rerun runtime/editor/workspace/plugin validation, and classify every failure owner before claiming green. |
| 1 | Runtime Interface / Editor Cutover | 部分完成 / 需迁移 | `zircon_runtime_interface` M0/M1 plan exists; active session `20260502-0119-runtime-interface-m2-editor-cutover` owns M2 editor cutover. | Move editor shared DTO/contract surface to `zircon_runtime_interface`, remove `zircon_editor` source-level `zircon_runtime` dependency, then validate editor tree/dependency graph. |
| 2 | Graphics / Plugin Migration | 部分完成 / 需迁移 | Plugin catalog/export/NativeDynamic diagnostics are accepted; render/plugin sessions have narrowed many VG/GI owner boundaries. | Resolve `ViewportRenderFrame`, stale GI/VG runtime imports, old owner paths, and plugin-owned execution handoff without restoring compatibility modules. |
| 3 | SRP/RHI Pass Execution | 部分完成 | RHI v1, RenderGraph resource graph, SRP pipeline/facade, and capability diagnostics have focused validation. | Add resource aliasing, real command recording, and pass executor ownership for actual SceneRenderer passes. |
| 4 | GI/VG Productization | 部分完成 / 需迁移 | Hybrid GI dynamic-light focused tests and VG/plugin boundary work exist; VG/GI heavy state is being moved to plugins. | Finish plugin-owned execution layer, Hybrid GI V1 acceptance, and Nanite-like VG cook/reference/GPU cull/raster/full-chain validation. |
| 5 | UI Productization | 部分完成 | UI M7/M10/M12/M13/M15/M16 runtime/package foundation, M18 runtime binding semantics, and editor scoped validation are accepted. | Complete M5 hot reload/conflict, M6 canvas behavior, editor M18 diagnostic projection after runtime-interface cutover, M14 localization UX, M21 action safety UX, M22 parity, and M24 emergency shell. |
| 6 | Physics / Animation Revalidation | 部分完成 | Targeted physics/animation runtime tests passed; runtime-vs-plugin ownership wording has been reconciled to runtime-owned built-ins; package-wide runtime validation was blocked by unrelated graphics test import failures. | Rerun package validation once graphics blockers clear. |
| 7 | SDF Font Real Bake | 缺失 | `UI SDF 字体真实 Bake 收束计划.md` defines the `fontsdf` direction and renderer-local scope. | Implement real glyph bake, metrics-based quads, atlas/capture tests, and docs; keep RGB MSDF/shaping/fallback chain out of this V1. |
| 8 | Final Workspace Green | 缺失 | No accepted full workspace green evidence after the recent UI/runtime/plugin changes. | Run final runtime, editor, root workspace, and `zircon_plugins` workspace validation, then archive exact pass/fail evidence and blocker owners. |

## Completed Baseline To Preserve

- UI foundation accepted as baseline: schema migration, component contract/private-boundary validation, descriptor registry authority, invalidation/cache foundation, M15 runtime/package resource refs, and M16 package/artifact manifest surface.
- Editor scoped baseline accepted: UI-owned editor failures were fixed and `cargo test -p zircon_editor --lib` passed in the closeout target.
- Plugin/export baseline accepted: catalog, SourceTemplate/LibraryEmbed/NativeDynamic manifest and diagnostics flows exist; NativeDynamic remains manifest/export/load plus M0 behavior boundary, not full behavior migration.
- Render foundation accepted: RHI v1, RenderGraph resource graph, SRP asset/feature/pass compile path, and `RenderFramework` facade are in place.
- Physics/Animation targeted baseline accepted: runtime scene tick/contact/sequence slices have targeted integration tests, but not package-wide acceptance.

## Explicit Non-Completion List

- Do not mark workspace broad green until root workspace and plugin workspace validation pass or all failures are categorized.
- Do not mark graphics/plugin complete while `ViewportRenderFrame`, stale GI/VG runtime imports, or plugin-owned execution handoff blockers remain.
- Do not mark M15 complete until editor dependency view, resolver/file existence checks, watcher/hot reload, runtime loader backend, and graphics/RHI consumption land.
- Do not mark M16 complete until cross-process persistent cache store and runtime-loader/resource UX are accepted.
- Do not mark NativeDynamic full behavior migration complete until ABI v2 behavior, unload/state, serialized command/event, denied capability, and plugin cutover milestones are accepted.
- Do not mark UI M14/M19/M20/M21 complete without localized text refs, transition/animation diagnostics, layout debug packet, and host action policy evidence.
- Do not mark Virtual Geometry Nanite-like complete without cook chain, CPU reference, VisBuffer64, GPU cull/raster, automatic LOD, and dual-host evidence.
- Do not mark SDF font real bake complete until the renderer uses real glyph outlines and capture tests prove non-placeholder SDF output.

## Dependency-Ordered Execution

### Milestone A: Broad Validation Re-baseline

**Implementation slices**

- [ ] Refresh `.codex/sessions` and `.codex/plans` before running validation.
- [ ] Check target drive free space and active `cargo` / `rustc` writers before selecting target dirs.
- [ ] Run scoped runtime/editor validation first; do not start full workspace validation while scoped blockers are unresolved.
- [ ] Classify failures into UI-owned, graphics/plugin-owned, runtime-interface-owned, physics/animation-owned, Cargo/disk/environment, or unrelated active-session blocker.
- [ ] Update this roadmap and the owning subsystem plan with exact commands, results, and blocker owner.

**Testing stage**

- `cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-remaining-runtime --message-format short --color never`
- `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-remaining-editor --message-format short --color never`

**Exit evidence**

- Runtime and editor status is either green or blocked with exact failing tests and owner labels.
- No workspace green claim is made from scoped package tests alone.

### Milestone B: Runtime Interface / Editor Cutover Tracking

**Implementation slices**

- [ ] Treat `docs/superpowers/plans/2026-05-01-runtime-interface-cdylib-loader.md` Milestone 2 as the implementation authority.
- [ ] Keep this roadmap read-only with respect to `zircon_runtime_interface`, `zircon_editor`, and `zircon_app::entry::runtime_library` while active session `20260502-0119-runtime-interface-m2-editor-cutover` owns the cutover.
- [ ] After handoff, record whether `zircon_editor` no longer compiles `zircon_runtime` as a source dependency.

**Testing stage**

- `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`
- `cargo test -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`
- `cargo tree -p zircon_editor --locked`

**Exit evidence**

- Editor dependency graph shows interface-only runtime boundary, or remaining source-level imports are listed by owner.

### Milestone C: Graphics / Plugin Migration Closure

**Implementation slices**

- [ ] Use the GI/VG plugin migration and render-plugin plans as source of truth.
- [ ] Fix the lowest graphics/plugin boundary blocker first: public-surface drift, stale owner-path imports, or plugin execution handoff.
- [ ] Preserve hard-cutover policy: no old-path re-exports, compatibility modules, or shim restores.
- [ ] Re-run runtime package validation after the blocker is addressed.

**Testing stage**

- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-graphics-plugin --message-format short --color never`
- `cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-graphics-plugin --message-format short --color never`
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-graphics-plugin --message-format short --color never`

**Exit evidence**

- Runtime lib tests compile past graphics modules.
- GI/VG plugin crates compile through their current runtime boundaries.

### Milestone D: SRP/RHI Pass Execution

**Implementation slices**

- [ ] Keep `zircon_runtime::core::framework::render` as the public facade and avoid direct `wgpu` leakage above renderer/backend boundaries.
- [ ] Add RenderGraph resource aliasing and real command-recording hooks without changing GI/VG feature ownership.
- [ ] Move real SceneRenderer passes behind executor ownership incrementally; leave monolithic path only as an internal migration entry until replacement is complete.

**Testing stage**

- `cargo test -p zircon_runtime --lib rhi::tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-srp-rhi --message-format short --color never`
- `cargo test -p zircon_runtime --lib render_graph::tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-srp-rhi --message-format short --color never`
- `cargo test -p zircon_runtime --lib graphics::tests::pipeline_compile --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-srp-rhi --message-format short --color never`
- `cargo test -p zircon_runtime --lib render_framework_bridge --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-srp-rhi --message-format short --color never`

**Exit evidence**

- Graph execution and facade tests prove pass registration, resource declarations, and viewport submission still agree.

### Milestone E: GI/VG Productization

**Implementation slices**

- [ ] Complete plugin-owned Hybrid GI and Virtual Geometry prepare/feedback/runtime host execution.
- [ ] Finish Hybrid GI V1 acceptance: surface-cache convergence, Deferred/Forward+ parity, capability downgrade, and end-to-end evidence.
- [ ] Advance VG Nanite-like chain in order: cook data, CPU reference, VisBuffer64/GPU minimal, automatic LOD/cull, dual-host parity, UE comparison.

**Testing stage**

- `cargo test -p zircon_runtime --lib hybrid_gi_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-gi-vg --message-format short --color never`
- `cargo test -p zircon_runtime --lib virtual_geometry_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-gi-vg --message-format short --color never`
- `cargo test -p zircon_runtime --lib node_and_cluster_cull_pass_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-gi-vg --message-format short --color never`
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-gi-vg --message-format short --color never`

**Exit evidence**

- Runtime and plugin tests agree on feature ownership, stats/readbacks, and opt-in behavior.

### Milestone F: UI Productization

**Implementation slices**

- [x] Root-class authoring UX closed at focused M10 editor-contract scope; keep broad workspace green separate until unrelated blockers clear.
- [x] Implement M18 runtime-owned binding schema and restricted expression model; public runtime integration evidence passes while filtered lib tests remain blocked by unrelated UI DTO identity drift.
- [ ] Implement M5 hot reload/conflict, M6 canvas resize/interact, and M24 emergency shell recovery.
- [ ] Implement M14 localization and M21 action safety as separate runtime-owned validation surfaces.
- [ ] Add M22 runtime/editor dual-host parity fixtures for layout and event semantics.
- [ ] Keep M15/M16 future work limited to editor dependency view, resource UX/loading, resolver, watcher, runtime loader, cross-process cache store, and broad validation.

**Testing stage**

- `cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-ui-productization --message-format short --color never`
- `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-ui-productization --message-format short --color never`
- `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-ui-productization --message-format short --color never`

**Exit evidence**

- Editor/runtime UI regressions pass, and each UI milestone records its own docs/archive evidence before status is raised.

### Milestone G: Physics / Animation Revalidation

**Implementation slices**

- [x] Reconcile docs that describe physics/animation as runtime built-ins versus pluginized runtime packages.
- [ ] Keep the accepted targeted runtime tests as baseline, but do not mark package-wide acceptance until graphics blockers are gone.
- [ ] Rerun runtime package validation after graphics/plugin migration stabilizes.

**Testing stage**

- `cargo test -p zircon_runtime --test physics_manager_runtime_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-physics-animation --message-format short --color never`
- `cargo test -p zircon_runtime --test runtime_physics_animation_tick_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-physics-animation --message-format short --color never`
- `cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-physics-animation --message-format short --color never`

**Exit evidence**

- Targeted tests remain green and package-wide blocker ownership is no longer unrelated graphics import drift.

### Milestone H: SDF Font Real Bake

**Implementation slices**

- [ ] Follow `UI SDF 字体真实 Bake 收束计划.md`; keep production renderer changes local to `zircon_runtime/src/graphics/scene/scene_renderer/ui/**`.
- [ ] Add `fontsdf`-based glyph bake, whitespace/missing-glyph handling, real metrics, and non-placeholder atlas bytes.
- [ ] Keep RGB MSDF, shaping, bidi, and full fallback chains out of this V1.

**Testing stage**

- `cargo test -p zircon_runtime --lib sdf_atlas --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-sdf-font --message-format short --color never`
- `cargo test -p zircon_runtime --lib sdf_draw_plan --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-sdf-font --message-format short --color never`
- `cargo test -p zircon_runtime --lib sdf_font_bake --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-sdf-font --message-format short --color never`
- `cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-sdf-font -- --test-threads=1 --nocapture`

**Exit evidence**

- Capture and unit tests prove SDF text uses real glyph outlines, not the old placeholder footprint.

### Milestone I: Final Workspace Green

**Implementation slices**

- [ ] Run validation only after active sessions finish or explicitly hand off their touched modules.
- [ ] Record exact target dirs, disk state, commands, pass/fail counts, and blocker ownership.
- [ ] Update subsystem plans only with evidence; do not mark milestones complete from unrelated scoped tests.

**Testing stage**

- `cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-final-runtime --message-format short --color never`
- `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-final-editor --message-format short --color never`
- `cargo test --workspace --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-final-workspace --message-format short --color never`
- `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --jobs 1 --target-dir E:\cargo-targets\zircon-engine-final-plugins --message-format short --color never`

**Exit evidence**

- Workspace status is either green with command evidence or blocked with a categorized owner matrix.

## Reference Sources

- `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md`
- `.codex/plans/UI 后续产品化与验证归档计划.md`
- `.codex/plans/GI_VG 插件化激进迁移计划.md`
- `.codex/plans/M5 Nanite-Like Virtual Geometry 全链收束计划.md`
- `.codex/plans/zircon_plugins 全量插件化收敛规划.md`
- `.codex/plans/UI SDF 字体真实 Bake 收束计划.md`
- `.codex/plans/Physics + Full Animation Support 新计划.md`
- `.codex/plans/Runtime_Editor 插件化剩余收敛计划.md`
- `.codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md`
- `.codex/plans/Zircon SRPRHI 渲染管线补全计划.md`
- `docs/superpowers/plans/2026-05-01-runtime-interface-cdylib-loader.md`
- `.codex/sessions/20260502-0119-runtime-interface-m2-editor-cutover.md`
- `.codex/sessions/20260501-2047-ui-productization-full-milestones.md`

## Update Rules

- This roadmap must be updated when any listed milestone changes status, but subsystem implementation details remain in their existing plans.
- Active session notes must not be rewritten as completed unless their owner archives or updates them with completion evidence.
- Do not record local machine secrets, auth files, private keys, or user-specific API values in this or any follow-up plan.
- Each status upgrade must include code owner, focused validation, docs/archive evidence, and remaining gaps.
