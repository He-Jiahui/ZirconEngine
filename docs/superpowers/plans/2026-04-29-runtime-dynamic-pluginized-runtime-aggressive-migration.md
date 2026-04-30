# Runtime Dynamic And Pluginized Runtime Aggressive Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or superpowers:subagent-driven-development to execute this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. In this repository, execute from the existing `main` checkout; do not create worktrees or feature branches.

**Goal:** Finish the Runtime Dynamic real-library closure and aggressively converge pluginized Runtime functionality extraction so optional runtime implementations live in `zircon_plugins`, not in `zircon_runtime` core paths.

**Architecture:** `zircon_runtime` keeps only lifecycle, asset, scene, base render, input, plugin loader, export/profile contracts, neutral render DTOs, and generic plugin registries. Runtime feature implementations move to `zircon_plugins/<plugin>/runtime` and enter the engine only through `RuntimePluginRegistrationReport`, `RuntimeExtensionRegistry`, render feature descriptors, SourceTemplate/LibraryEmbed linked registration, or NativeDynamic manifests/diagnostics. No compat, shim, alias, re-export, or legacy activation path is allowed to survive this milestone.

**Tech Stack:** Rust workspace, PowerShell helper scripts, Cargo, `zircon_runtime`, `zircon_app`, `zircon_editor`, independent `zircon_plugins` workspace, NativeDynamic ABI v1/v2 manifest handshake, `plugin.toml`, SourceTemplate/LibraryEmbed/NativeDynamic export profiles.

**Testing Policy For This Milestone:** Do not run tests, Cargo checks, rustfmt, helper-script reruns, or validation scripts during implementation slices. Run all listed commands only in Task 12, the milestone testing stage.

---

## File Structure And Ownership Map

**Native Dynamic real-library closure**
- Modify: `zircon_plugins/Cargo.toml` only when workspace membership or canonical native fixture naming changes.
- Modify: `zircon_plugins/native_dynamic_fixture/plugin.toml` for package metadata, module target modes, crate names, capabilities, and packaging declarations.
- Modify: `zircon_plugins/native_dynamic_fixture/native/Cargo.toml` for `cdylib` crate metadata.
- Modify: `zircon_plugins/native_dynamic_fixture/native/src/lib.rs` for ABI v1 descriptor plus runtime/editor entry symbols.
- Modify: `zircon_runtime/src/plugin/native_plugin_loader/*` for discovery, target-specific library selection, ABI probing, entry report conversion, load-manifest containment, duplicate package diagnostics, and report projection.
- Modify: `zircon_runtime/src/plugin/export_build_plan/*` for profile strategy resolution, native package dedupe, generated load-manifest rows, source package lookup, and native materialization reports.
- Modify: `zircon_editor/src/ui/host/editor_manager_plugins_export/*` and `zircon_editor/src/ui/host/native_dynamic_export_preparation/*` only if native-aware editor export build path needs report or staging synchronization.

**Pluginized Runtime extraction**
- Modify: `zircon_runtime/src/plugin/*` for descriptor, project selection, extension registry, report, and catalog contracts.
- Modify: `zircon_runtime/src/builtin/runtime_modules.rs` only to remove optional implementation activation, not to add new optional branches.
- Modify: `zircon_runtime/src/graphics/**` only for neutral render DTOs, base renderer seams, generic feature registry, and removal of legacy VG/GI/particle activation paths.
- Modify: `zircon_plugins/{physics,animation,sound,net,navigation,particles,texture,virtual_geometry,hybrid_gi}/runtime/**` as the owning implementation locations.
- Do not recreate `zircon_runtime/src/physics`, `zircon_runtime/src/animation`, or old optional extension roots.

**Documentation and coordination**
- Modify: `.codex/plans/zircon_plugins 全量插件化收敛规划.md`.
- Modify: `.codex/plans/Runtime_Editor 插件化剩余收敛计划.md`.
- Modify: `docs/engine-architecture/runtime-editor-pluginized-export.md`.
- Modify: relevant `docs/assets-and-rendering/*` files when VG/GI/particle render ownership changes.
- Maintain active session state in `.codex/sessions/20260429-0605-no-test-milestone-continuation.md` until the milestone closes.

---

## Task 1: Freeze Hard-Cut Rules And Active Ownership

**Files:**
- Modify: `.codex/sessions/20260429-0605-no-test-milestone-continuation.md`
- Read before editing source: `.codex/plans/zircon_plugins 全量插件化收敛规划.md`
- Read before editing source: `.codex/plans/Runtime_Editor 插件化剩余收敛计划.md`
- Read before editing source: `.codex/sessions/*.md`

- [ ] Record in the session note that this milestone owns Runtime Dynamic closure and pluginized Runtime migration convergence.
- [ ] Record the no-test rule: no tests, checks, helper reruns, Cargo commands, rustfmt, or validation scripts until Task 12.
- [ ] Record current active owners to avoid blind edits: render/plugin cutover owns broad `zircon_runtime::graphics`, Runtime UI showcase owns `zircon_runtime::ui` and editor template/projection files, editor chrome sessions own Slint/workbench chrome files.
- [ ] Record that aggressive migration means direct deletion or replacement of old paths, not dual-path compatibility.

**Completion gate:** Session note states milestone scope, forbidden validation before Task 12, and active ownership warnings.

---

## Task 2: Stabilize Coordination Helper For The Future Testing Stage

**Files:**
- Modify: `.opencode/skills/zircon-project-skills/cross-session-coordination/scripts/Get-RecentCoordinationContext.ps1`
- Modify: `.codex/skills/zircon-project-skills/cross-session-coordination/scripts/Get-RecentCoordinationContext.ps1`

- [ ] Keep the existing `Set-StrictMode -Version Latest` behavior.
- [ ] Wrap the assignment of `Get-RecentPlanFiles` in `@(...)` before checking `.Count`.
- [ ] Wrap the assignment of `Get-RecentSessionFiles` in `@(...)` before checking `.Count`.
- [ ] Do not run either helper script in this task.
- [ ] Add the exact helper rerun commands to the Task 12 testing checklist.

**Completion gate:** Both helper copies use array-wrapped call sites for recent plans and active sessions.

---

## Task 3: Canonicalize NativeDynamic Real Library Fixture

**Files:**
- Modify: `zircon_plugins/Cargo.toml`
- Modify: `zircon_plugins/native_dynamic_fixture/plugin.toml`
- Modify: `zircon_plugins/native_dynamic_fixture/native/Cargo.toml`
- Modify: `zircon_plugins/native_dynamic_fixture/native/src/lib.rs`
- Modify: `docs/engine-architecture/runtime-editor-pluginized-export.md`

- [ ] Treat `native_dynamic_fixture` as the canonical real-library fixture package name.
- [ ] Remove stale `native_dynamic_sample` references from docs, plans, tests, and package metadata when they appear in touched files.
- [ ] Ensure `zircon_plugins/Cargo.toml` includes `native_dynamic_fixture/native` as an independent plugin workspace member and does not add `zircon_plugins` to the root workspace.
- [ ] Ensure `plugin.toml` declares the package id `native_dynamic_fixture`, runtime module crate name `zircon_plugin_native_dynamic_fixture_native`, editor module crate name `zircon_plugin_native_dynamic_fixture_native`, and appropriate client/editor target modes.
- [ ] Ensure the native crate is `crate-type = ["cdylib"]` and avoids dependencies on `zircon_editor` or optional runtime implementation crates.
- [ ] Ensure `native/src/lib.rs` exports these ABI v1 compatibility symbols:
  - `zircon_native_plugin_descriptor_v1`
  - `zircon_native_dynamic_fixture_runtime_entry_v1`
  - `zircon_native_dynamic_fixture_editor_entry_v1`
- [ ] Ensure `native/src/lib.rs` also exports the current ABI v2 manifest/diagnostic symbols without treating them as real behavior migration:
  - `zircon_native_plugin_descriptor_v2`
  - `zircon_native_dynamic_fixture_runtime_entry_v2`
  - `zircon_native_dynamic_fixture_editor_entry_v2`
- [ ] Ensure descriptor and entry reports return owned manifest/diagnostic data compatible with `NativePluginAbiV1` / `NativePluginEntryReportV1` and `NativePluginAbiV2` / `NativePluginEntryReportV2`.

**Completion gate:** There is one canonical real native fixture package and no touched docs or metadata refer to `native_dynamic_sample`.

---

## Task 4: Close Native Loader Target-Specific Runtime/Editor Paths

**Files:**
- Modify: `zircon_runtime/src/plugin/native_plugin_loader/discover_load_manifest.rs`
- Modify: `zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs`
- Modify: `zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs`
- Modify: `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs`
- Modify: `zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_manifest.rs`
- Modify: `zircon_runtime/src/plugin/native_plugin_loader/platform_library_name.rs`

- [ ] Keep directory discovery and generated load-manifest discovery as separate entry points.
- [ ] Ensure `discover_from_load_manifest(export_root)` rejects `path` or `manifest` entries escaping `export_root` before probing libraries.
- [ ] Ensure duplicate package ids keep the first valid candidate and report diagnostics for later duplicates before library probing.
- [ ] Ensure runtime-only loading selects runtime module crate names, invokes runtime entry symbols only, and strips editor module declarations before startup reports are exposed.
- [ ] Ensure editor-only loading selects editor module crate names, invokes editor entry symbols only, and strips runtime module declarations before editor registration reports are exposed.
- [ ] Ensure full diagnostic loading groups target module kinds by dynamic library path so combined runtime/editor cdylibs load once and split cdylibs load independently.
- [ ] Ensure missing library, ABI mismatch, missing symbol, invalid manifest TOML, null entry pointer, and entry diagnostics remain non-fatal diagnostics.
- [ ] Do not add special cases for `native_dynamic_fixture`; all behavior must be package-manifest and module-kind driven.

**Completion gate:** Runtime/editor/all native loading are target-specific projections over the same manifest/ABI contract, with no plugin-id special cases.

---

## Task 5: Close NativeDynamic Export Materialization And Load-Manifest Generation

**Files:**
- Modify: `zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs`
- Modify: `zircon_runtime/src/plugin/export_build_plan/materialize.rs`
- Modify: `zircon_runtime/src/plugin/export_build_plan/native_plugin_load_manifest_template.rs`
- Modify: `zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs`
- Modify: `zircon_runtime/src/plugin/export_build_plan/generated_files.rs`

- [ ] Keep per-selection `packaging` authoritative; export profile strategies enable carriers but do not coerce selections.
- [ ] Deduplicate `NativeDynamic` copied packages by plugin id in first-seen order.
- [ ] Deduplicate generated native load-manifest rows by copied package id and sanitized output directory.
- [ ] Reject sanitized output directory collisions with diagnostics before load-manifest rows are generated.
- [ ] Copy only distributable native artifacts from `native/`: `.dll`, `.so`, `.dylib`, and platform symbol sidecars.
- [ ] Exclude runtime/editor Rust source crates and native crate source files from copied runtime distributions.
- [ ] Record diagnostics when a selected native package has no `native/` directory or no compiled artifact under `native/`.
- [ ] Prevent direct source package lookup for ids containing path components; such packages must be discovered by scanning `plugin.toml` under the configured plugin root.
- [ ] Ensure pure `NativeDynamic` exports can produce copied packages and `plugins/native_plugins.toml` without requiring a generated SourceTemplate Cargo project.

**Completion gate:** NativeDynamic export output is a copied package set plus load manifest, not a source leak or profile coercion path.

---

## Task 6: Close Editor Native-Aware Export Build Reporting

**Files:**
- Modify only if needed after static inspection: `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/*`
- Modify only if needed after static inspection: `zircon_editor/src/ui/host/native_dynamic_export_preparation/*`
- Modify: `docs/engine-architecture/runtime-editor-pluginized-export.md`

- [ ] Keep editor source edits out of active workbench/Slint chrome areas.
- [ ] Ensure native-aware build stages native package metadata/resources into `.native-dynamic-staging/<sanitized-plugin-id>` before materialization.
- [ ] Ensure native-aware build reuses existing compiled native artifacts when present and records build diagnostics when it invokes native Cargo.
- [ ] Ensure `EditorExportBuildReport` carries generated files, copied packages, source-template Cargo invocation, native Cargo invocations, stdout/stderr/status, and diagnostics.
- [ ] Ensure exported loader validation uses runtime-only load-manifest API for runtime export target modes and editor-only load-manifest API for editor target modes.
- [ ] Keep `EditorManager` entry points structural; detailed report, staging, artifact filtering, cargo invocation, cleanup, and diagnostics behavior must stay in folder-backed child modules.

**Completion gate:** Native-aware editor export reports all generated/copied/native build/load-manifest diagnostics without pulling editor UI chrome into this milestone.

**Static audit / Task 12 note 2026-04-30:** Editor export/reporting inspection found the report and staging surfaces already carry the required generated/copied/native Cargo diagnostics, and exported load-manifest validation routes through the target-mode-specific loader. `native_project_selection(...)` now aggregates target modes across all package modules, matching builtin selection/status reporting so editor-only native packages do not complete into empty default target modes. Focused Task 12 regression coverage now includes `cargo test -p zircon_editor --lib native_selection_aggregates_runtime_and_editor_module_target_modes --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture` and `cargo test -p zircon_editor --lib native_aware_completion_aggregates_native_module_target_modes --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture`.

---

## Task 7: Hard-Cut Runtime Legacy Activation Paths

**Files:**
- Modify: `zircon_runtime/Cargo.toml`
- Modify: `zircon_runtime/src/builtin/runtime_modules.rs`
- Modify: `zircon_runtime/src/plugin/runtime_plugin/*`
- Modify: `zircon_runtime/src/plugin/extension_registry/*`
- Modify: `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/*`
- Modify: `zircon_runtime/src/graphics/pipeline/*`
- Modify: `zircon_runtime/src/graphics/runtime/render_framework/*`

- [ ] Remove or keep removed root optional implementation paths such as `zircon_runtime/src/physics` and `zircon_runtime/src/animation`.
- [ ] Remove runtime Cargo feature switches that compile optional physics, animation, sound, net, navigation, particles, texture, VG, or GI implementations back into `zircon_runtime`.
- [ ] Keep `GraphicsBase`, assets, scene, input, base render, plugin loader, and export/profile contracts in runtime core.
- [ ] Ensure required project plugins are satisfied only by target-matching linked `RuntimePluginRegistrationReport` values or valid native diagnostics where applicable.
- [ ] Ensure optional missing plugins produce diagnostics and do not instantiate old runtime implementation modules.
- [ ] Ensure `BuiltinRenderFeature::VirtualGeometry`, `BuiltinRenderFeature::GlobalIllumination`, and `BuiltinRenderFeature::Particle` do not create passes, executor ids, or runtime state without plugin-contributed descriptors.
- [ ] Ensure flagship quality profiles open capability gates only; they must not reopen legacy built-in render feature switches.

**Completion gate:** Runtime core cannot activate optional migrated implementations through Cargo features, builtin module branches, or legacy render enum identity.

---

## Task 8: Finish Non-Render Runtime Plugin Package Ownership

**Files:**
- Modify: `zircon_plugins/physics/runtime/**`
- Modify: `zircon_plugins/animation/runtime/**`
- Modify: `zircon_plugins/sound/runtime/**`
- Modify: `zircon_plugins/net/runtime/**`
- Modify: `zircon_plugins/navigation/runtime/**`
- Modify: `zircon_plugins/particles/runtime/**`
- Modify: `zircon_plugins/texture/runtime/**`
- Modify: `zircon_plugins/*/plugin.toml`
- Modify: `zircon_runtime/src/plugin/package_manifest/*`

- [ ] Confirm every non-render runtime plugin has a non-empty `RuntimePlugin` implementation.
- [ ] Confirm every non-render runtime plugin registers at least one concrete `ModuleDescriptor` when it needs startup graph visibility.
- [ ] Confirm `navigation`, `particles`, and `texture` remain manager-backed activation points, not descriptor-only shells.
- [ ] Confirm each plugin descriptor target modes match its `plugin.toml` target modes.
- [ ] Confirm plugin package manifests declare runtime crate names and capabilities needed by SourceTemplate and LibraryEmbed.
- [ ] Remove any touched descriptor-only fallback that exists only to keep tests green without real activation.

**Completion gate:** Non-render optional runtime features are owned by plugin runtime crates with real registration surfaces and no core runtime implementation fallback.

---

## Task 9: Aggressively Extract VG/GI Heavy Runtime State To Plugin Ownership

**Files:**
- Modify: `zircon_plugins/virtual_geometry/runtime/**`
- Modify: `zircon_plugins/hybrid_gi/runtime/**`
- Modify: `zircon_runtime/src/graphics/runtime/virtual_geometry/**`
- Modify: `zircon_runtime/src/graphics/runtime/hybrid_gi/**`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer/advanced_plugin_outputs/**`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/**`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/virtual_geometry/**`
- Modify: `zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/**`
- Modify: `docs/assets-and-rendering/render-framework-architecture.md`

- [ ] Coordinate with active render/plugin owner before editing overlapping graphics files.
- [ ] Keep neutral DTOs, readback report DTOs, base renderer seams, and generic render feature registry in `zircon_runtime`.
- [ ] Move plugin-owned VG/GI runtime state, prepare scheduling, feedback accumulation, GPU resource packages, and implementation-specific tests toward `zircon_plugins/virtual_geometry/runtime` and `zircon_plugins/hybrid_gi/runtime`.
- [ ] Replace direct field access across renderer core with owner methods or explicit store-parts packages before moving ownership.
- [ ] Keep `SceneRendererAdvancedPluginOutputs` and `SceneRendererAdvancedPluginResources` as temporary migration seams only while extracting state; do not let them become permanent generic dumping grounds.
- [ ] Remove legacy seed-backed `compat` naming and old compatibility source vocabulary from touched code and docs.
- [ ] Ensure base render without linked VG/GI descriptors stays lightweight and does not allocate heavy VG/GI state.

**Completion gate:** VG/GI heavy implementation state is either moved into plugin runtime crates or isolated behind explicit migration seams with documented remaining move targets; runtime graphics keeps only neutral contracts and base renderer paths.

---

## Task 10: Clean Hard-Cut Residue In Docs And Plans

**Files:**
- Modify: `.codex/plans/zircon_plugins 全量插件化收敛规划.md`
- Modify: `.codex/plans/Runtime_Editor 插件化剩余收敛计划.md`
- Modify: `docs/engine-architecture/runtime-editor-pluginized-export.md`
- Modify: `docs/engine-architecture/index.md` only if document links change.
- Modify: `docs/assets-and-rendering/render-framework-architecture.md` when graphics ownership changes.

- [ ] Replace stale static/builtin/legacy/facade wording in touched pluginization docs with descriptor/catalog/export/owner/accessor terminology.
- [ ] Document NativeDynamic as manifest/diagnostics/export package backend, not as the activation path for linked Rust runtime implementations.
- [ ] Document SourceTemplate and LibraryEmbed as the activation paths for linked Rust plugin crates.
- [ ] Document that `zircon_plugins` remains an independent workspace and is not added to root workspace members.
- [ ] Document remaining VG/GI migration state explicitly without claiming runtime graphics is fully extracted until Task 9 is complete.
- [ ] Keep machine-readable headers current in modified docs.

**Completion gate:** Plans and docs match the actual hard-cut architecture and do not describe old optional runtime implementation paths as valid.

---

## Task 11: Source-Level Residue Sweep Without Running Validation

**Files:**
- Inspect: `zircon_runtime/src/**`
- Inspect: `zircon_plugins/**`
- Inspect: `zircon_app/src/**`
- Inspect: `zircon_editor/src/ui/host/**`
- Modify only files whose old-path residue is unambiguously in this milestone scope.

- [ ] Search for stale identifiers and terms in source text using file/content search tools rather than build commands.
- [ ] Remove old optional runtime activation branches when they are found in runtime core.
- [ ] Remove touched `compat`, `shim`, `bridge`, `legacy`, or alias modules that only preserve old paths.
- [ ] Remove stale native sample naming from touched source/tests/docs if it conflicts with canonical `native_dynamic_fixture`.
- [ ] Record unresolved residue that belongs to an active owner instead of editing across active ownership blindly.

**Completion gate:** Obvious source-level hard-cut residue in milestone-owned files is removed or recorded as an active-owner handoff.

**Static audit note 2026-04-30:** Source searches across milestone-owned plugin/export paths found no live `native_dynamic_sample`, migration-only `compat`/`shim`/`facade`/`legacy` terms, or old optional plugin Cargo feature switches. The only source hit was `editor.host.capability_bridge`, which is a current editor capability id, not a migration bridge module.

---

## Task 12: Milestone Testing Stage

**Files:**
- Read: `.github/workflows/ci.yml`
- Run from repository root after Tasks 1-11 are complete.
- Use a shared target directory according to `.opencode/skills/zircon-dev/validation/SKILL.md` and the cargo target disk policy.

- [x] Run coordination helper validation:

```powershell
.\.opencode\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot "E:\Git\ZirconEngine" -LookbackHours 4
.\.codex\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot "E:\Git\ZirconEngine" -LookbackHours 4
```

- [x] Run touched documentation/script whitespace validation:

```powershell
git diff --check -- .codex .opencode docs zircon_runtime zircon_plugins zircon_editor zircon_app
```

- [x] Run native loader/export focused runtime integration validation:

```powershell
cargo test -p zircon_runtime --test native_plugin_loader_contract --locked --jobs 1 -- --nocapture
cargo test -p zircon_runtime --test export_build_plan_contract --locked --jobs 1 -- --nocapture
```

- [x] Run minimal runtime no-default validation:

```powershell
cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1
```

- [x] Run runtime plugin registration/export contract validation:

```powershell
cargo test -p zircon_runtime --lib export_build_plan --no-default-features --features core-min --locked --jobs 1 -- --nocapture
cargo test -p zircon_runtime --lib runtime_modules --locked --jobs 1 -- --nocapture
cargo test -p zircon_runtime --lib plugin_render_feature --locked --jobs 1 -- --nocapture
```

- [x] Run native fixture workspace validation:

```powershell
cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1
```

- [x] Run plugin runtime crate validation:

```powershell
cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_physics_runtime -p zircon_plugin_animation_runtime -p zircon_plugin_sound_runtime -p zircon_plugin_net_runtime -p zircon_plugin_navigation_runtime -p zircon_plugin_particles_runtime -p zircon_plugin_texture_runtime -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --locked --jobs 1 -- --nocapture
```

- [x] Run editor plugin/report validation only after runtime/native checks are green:

```powershell
cargo test -p zircon_editor --lib editor_runtime_ --locked --jobs 1 -- --nocapture
cargo test -p zircon_editor --lib native_aware_completion_aggregates_native_module_target_modes --locked --jobs 1 -- --nocapture
cargo test -p zircon_editor --lib native_selection_aggregates_runtime_and_editor_module_target_modes --locked --jobs 1 -- --nocapture
cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_editor_support -p zircon_plugin_physics_editor -p zircon_plugin_animation_editor -p zircon_plugin_sound_editor -p zircon_plugin_net_editor -p zircon_plugin_navigation_editor -p zircon_plugin_particles_editor -p zircon_plugin_texture_editor -p zircon_plugin_virtual_geometry_editor -p zircon_plugin_hybrid_gi_editor -p zircon_plugin_runtime_diagnostics_editor -p zircon_plugin_ui_asset_authoring_editor -p zircon_plugin_native_window_hosting_editor --locked --jobs 1
```

- [x] Run final validator only after focused checks pass or after failures have been documented and fixed:

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir "D:\cargo-targets\zircon-runtime-dynamic-milestone"
```

**Completion gate:** All commands above either pass with fresh output or have failures root-caused, assigned to the lowest owning layer, fixed, and rerun upward.

**Task 12 evidence 2026-04-30 (Windows, shared target `D:\cargo-targets\zircon-runtime-dynamic-milestone`):**
- `.\.opencode\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot "E:\Git\ZirconEngine" -LookbackHours 4` passed and reported active render/plugin, UI cutover, and Runtime Dynamic sessions.
- `.\.codex\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot "E:\Git\ZirconEngine" -LookbackHours 4` passed and reported the same active sessions.
- `git diff --check -- .codex .opencode docs zircon_runtime zircon_plugins zircon_editor zircon_app` exited `0`; output still included LF-to-CRLF warnings for dirty active-owner files.
- `cargo test -p zircon_runtime --test native_plugin_loader_contract --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture` passed: 1 test.
- `cargo test -p zircon_runtime --test export_build_plan_contract --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture` passed: 8 tests.
- `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone"` passed.
- `cargo test -p zircon_runtime --lib export_build_plan --no-default-features --features core-min --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture` passed: 11 tests.
- `cargo test -p zircon_runtime --lib runtime_modules --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture` passed: 5 tests.
- `cargo test -p zircon_runtime --lib plugin_render_feature --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture` passed: 7 tests.
- `cargo check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone"` passed.
- `cargo test --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_physics_runtime -p zircon_plugin_animation_runtime -p zircon_plugin_sound_runtime -p zircon_plugin_net_runtime -p zircon_plugin_navigation_runtime -p zircon_plugin_particles_runtime -p zircon_plugin_texture_runtime -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture` passed: 76 runtime plugin tests plus doctests.
- `cargo test -p zircon_editor --lib editor_runtime_ --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture` passed: 12 tests.
- `cargo test -p zircon_editor --lib native_aware_completion_aggregates_native_module_target_modes --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture` passed: 1 test.
- `cargo test -p zircon_editor --lib native_selection_aggregates_runtime_and_editor_module_target_modes --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone" -- --nocapture` passed: 1 test.
- `cargo check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_editor_support -p zircon_plugin_physics_editor -p zircon_plugin_animation_editor -p zircon_plugin_sound_editor -p zircon_plugin_net_editor -p zircon_plugin_navigation_editor -p zircon_plugin_particles_editor -p zircon_plugin_texture_editor -p zircon_plugin_virtual_geometry_editor -p zircon_plugin_hybrid_gi_editor -p zircon_plugin_runtime_diagnostics_editor -p zircon_plugin_ui_asset_authoring_editor -p zircon_plugin_native_window_hosting_editor --locked --jobs 1 --target-dir "D:\cargo-targets\zircon-runtime-dynamic-milestone"` passed.
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir "D:\cargo-targets\zircon-runtime-dynamic-milestone"` passed `cargo build --workspace --locked --target-dir D:\cargo-targets\zircon-runtime-dynamic-milestone` and `cargo test --workspace --locked --target-dir D:\cargo-targets\zircon-runtime-dynamic-milestone`.

---

## Task 13: Close Session And Milestone Documentation

**Files:**
- Modify: `.codex/sessions/20260429-0605-no-test-milestone-continuation.md`
- Move to archive when complete: `.codex/sessions/archive/20260429-0605-no-test-milestone-continuation.md`
- Modify: `.codex/plans/zircon_plugins 全量插件化收敛规划.md`
- Modify: `.codex/plans/Runtime_Editor 插件化剩余收敛计划.md`
- Modify: `docs/engine-architecture/runtime-editor-pluginized-export.md`

- [x] Record exact files changed for the milestone.
- [x] Record the Task 12 commands and outcomes.
- [x] Record remaining active-owner handoffs, especially any VG/GI graphics extraction that remains outside this session.
- [x] Archive the active session note with `status: completed` only after Task 12 evidence is recorded.
- [x] Do not commit unless the user explicitly requests a commit.

**Completion gate:** Active session root no longer contains this completed note, and docs/plans reflect the final milestone state.

**Task 13 evidence 2026-04-30:** Session closeout was written to `.codex/sessions/archive/20260429-0605-no-test-milestone-continuation.md`. Remaining handoffs are Native ABI v2 real behavior migration and active render/plugin VG/GI heavy-state extraction; both stay outside this Runtime Dynamic validation closeout.
