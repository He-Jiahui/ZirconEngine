# Full Milestone Export Hosts And Component Drawer Templates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `continuous-milestone-execution` with `zircon-dev`, `zr-architecture-first-engineering`, and `code-module-docs-maintenance`. Execute milestones in order from runtime contracts upward; do not run build/unit-test loops after every implementation slice. Each milestone has its own named testing stage.

**Goal:** Make Android, iOS, WebGPU, and WASM export plans emit real host scaffold artifacts while allowing editor plugins to contribute structured Component Drawer UI templates that the Inspector can project or safely fall back from.

**Architecture:** `zircon_runtime` owns platform-neutral export contracts, target policy, generated artifact manifests, and resource/host scaffold file planning. `zircon_app` remains the process/runtime host crate referenced by generated scaffolds, and `zircon_editor` owns authoring-time Build Export and Inspector projection of plugin-provided drawer templates. Plugin packages remain declaration sources only in this milestone; do not edit active plugin lanes unless a fixture is absolutely required.

**Tech Stack:** Rust workspace (`zircon_app`, `zircon_runtime`, `zircon_editor`, `zircon_runtime_interface`), Cargo, Slint host projection, TOML-backed UI documents, existing editor operation and component drawer registries.

---

## Architecture Note

- Roadmap anchor: `.codex/plans/全系统重构方案.md` and `.codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md`; follow the current fixed root package shape from the skills: `zircon_app`, `zircon_runtime`, `zircon_editor`, and `zircon_runtime_interface`.
- Owner roles: export artifact contract belongs to `zircon_runtime::plugin::export_build_plan`; generated app hosts reference `zircon_app::EntryRunner`; authoring state and Inspector drawer projection belong to `zircon_editor`.
- Required abstractions: platform host scaffold descriptor, generated artifact purpose/path contract, drawer template descriptor metadata, Inspector payload fields for template identity and binding action routes, and fallback diagnostics for missing/unavailable plugin drawers.
- Mainstream precedent: mobile/browser hosts must be generated as target-specific launcher scaffolds rather than pretending desktop `main.rs` is portable; editor custom inspectors should be declarative descriptors with validated bindings and safe unloaded-plugin fallback.
- Depth test: adding another host target or drawer template should require adding a target scaffold/template descriptor, not changing editor runtime control flow or hard-coding plugin IDs.

## Current Baseline

- `ExportTargetPlatform::{Android,Ios,WebGpu,Wasm}` and `ExportPlatformPolicy` already classify mobile/browser host kind, resource strategy, and native dynamic restrictions in `zircon_runtime/src/plugin/export_profile.rs`.
- `ExportBuildPlan::from_project_manifest(...)` already rejects NativeDynamic packaging for mobile/browser targets and drives generated files through `generated_files_for_profile(...)`.
- `generated_files_for_profile(...)` emits desktop `Cargo.toml`, `src/main.rs`, `src/zircon_plugins.rs`, and `assets/zircon-project.toml` for SourceTemplate exports.
- `ComponentDrawerDescriptor` currently records `component_type`, `ui_document`, `controller`, and operation bindings; runtime registration validates that bindings resolve to real editor operations.
- Inspector snapshot/projection currently marks drawer availability from runtime component descriptors and emits generic editable property rows; it does not carry plugin drawer UI template identity or consume plugin drawer templates.
- The worktree has concurrent edits in Build Export, pane projection, docs, and plugin lanes. New work must preserve those edits and keep changes narrow.

## File Map

- Modify `zircon_runtime/src/plugin/export_profile.rs`: add target host identifiers and helpers if needed without changing existing policy semantics.
- Modify `zircon_runtime/src/plugin/export_build_plan/generated_files.rs`: dispatch generated file emission by platform host kind.
- Add `zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs`: build target-specific generated scaffold files for Android, iOS, WebGPU, and WASM.
- Modify `zircon_runtime/src/plugin/export_build_plan/main_template.rs`: keep desktop entry generation isolated from target-specific platform scaffolds.
- Modify `zircon_runtime/src/plugin/export_build_plan/mod.rs`: wire the new platform host files module.
- Modify `zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs`: add focused contract tests for mobile/browser generated files, resource strategy, and no native dynamic files.
- Modify `zircon_editor/src/core/editor_extension.rs`: extend Component Drawer descriptors with structured template metadata while preserving validation rules.
- Modify `zircon_editor/src/ui/host/editor_extension_registration.rs`: validate any new drawer template operation binding references against available operations.
- Modify `zircon_editor/src/ui/host/editor_event_runtime_access.rs`: expose enabled drawer/template descriptors to Inspector snapshot builders without leaking disabled capability registrations.
- Modify `zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot.rs`: add drawer template metadata to plugin component snapshots.
- Modify `zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs`: resolve Component Drawer descriptors for selected dynamic components and attach drawer template metadata.
- Modify `zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs`: carry drawer template metadata into host view data.
- Modify `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs`: carry drawer template metadata in Inspector payloads.
- Modify `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/inspector.rs`: project snapshot drawer metadata into payloads.
- Modify `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs`: project drawer metadata into view data.
- Modify `zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs`: render or annotate plugin drawer template nodes while keeping generic fallback rows.
- Modify `zircon_editor/src/tests/editor_event/runtime.rs`: add Component Drawer template descriptor registration and capability fallback tests.
- Modify `zircon_editor/src/tests/host/pane_presentation.rs`: assert Inspector payload/projection carries drawer template identity.
- Update `docs/engine-architecture/runtime-editor-pluginized-export.md`: record export host scaffold and Component Drawer template contracts.
- Update `docs/editor-and-tooling/editor-command-workflow.md`: record Inspector custom drawer projection and fallback behavior.
- Update `tests/acceptance/editor-component-drawer-bindings.md`: add acceptance inventory and command evidence for template-backed drawers.

## M1 Export Host Contract

- Goal: `ExportBuildPlan` must expose deterministic, target-specific generated artifact contracts for Android, iOS, WebGPU, and WASM.
- In-scope behaviors: mobile/browser SourceTemplate plans emit scaffold files; desktop plans keep the existing Cargo/main/plugin/assets contract; mobile/browser never emit native dynamic plugin load manifests; generated file purposes make host/runtime/resource responsibility explicit.
- Dependencies: current `ExportTargetPlatform` policy and plugin packaging diagnostics must remain correct.
- Implementation slices:
  - Add runtime test coverage in `zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs` for Android, iOS, WebGPU, and WASM generated file paths and purposes.
  - Add `platform_host_files.rs` with pure functions that return `ExportGeneratedFile` entries for mobile/browser host scaffolds.
  - Update `generated_files_for_profile(...)` to call desktop generation for desktop targets and target-specific scaffold generation for mobile/browser targets when SourceTemplate is enabled.
  - Keep `main_template.rs` desktop-focused; do not duplicate desktop dynamic plugin bootstrap in mobile/browser scaffolds.
  - Update export architecture docs with the new artifact contract.
- Testing stage `M1 Export Host Contract Testing`:
  - Run `rustfmt --edition 2021 --check zircon_runtime/src/plugin/export_build_plan/*.rs zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs`.
  - Run `cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --jobs 1 --message-format short --color never`.
  - Run `cargo test -p zircon_runtime source_template_emits_mobile_and_browser_host_scaffolds --locked --jobs 1 --message-format short --color never`.
  - Debug/correction loop: if generated paths differ by platform, fix `platform_host_files.rs` before changing tests.
  - Exit evidence: tests pass and docs list all target scaffold paths.

## M2 Platform Host Scaffolds And Build Export Projection

- Goal: Build Export reports and UI projections must show mobile/browser scaffold files as real export artifacts, not desktop-only generated counts.
- In-scope behaviors: generated file list includes platform scaffold names; Build Export target rows surface host kind/resource strategy diagnostics; export materialization writes all scaffold files under the selected output root.
- Dependencies: M1 artifact contract.
- Implementation slices:
  - Reuse existing `ExportGeneratedFile` materialization; avoid adding special-case editor write paths.
  - Update Build Export projection structs only if existing view data cannot show host kind/resource strategy clearly.
  - Add or adjust editor tests that inspect generated file counts/status for Android/WebGPU profiles.
  - Keep native dynamic preparation path desktop-only through existing `should_probe_exported_native_manifest(...)` and `platform_policy.supports_native_dynamic` rules.
  - Update docs with how export output directories differ by desktop/mobile/browser.
- Testing stage `M2 Platform Build Export Testing`:
  - Run `rustfmt --edition 2021 --check` on touched runtime/editor export files.
  - Run focused editor export tests matching `build_export`.
  - Run `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`.
  - Debug/correction loop: if editor compile fails in unrelated dirty files, record exact failure and isolate whether touched APIs caused it before widening edits.
  - Exit evidence: Build Export tests pass or unrelated concurrent failures are documented with touched-file isolation.

## M3 Component Drawer Template Protocol

- Goal: Component Drawer descriptors must describe a structured template contract that plugins can register and editor runtime can validate.
- In-scope behaviors: descriptors carry template/document identity, controller, operation bindings, optional display metadata, and data binding root; runtime rejects invalid operation references; disabled capability registrations remain hidden.
- Dependencies: existing editor operation registry and component drawer binding validation.
- Implementation slices:
  - Add tests in `zircon_editor/src/tests/editor_event/runtime.rs` for successful template descriptor lookup and rejection when a drawer references a missing operation binding.
  - Extend `ComponentDrawerDescriptor` minimally with optional template metadata that can be serialized and cloned without breaking existing constructor callers.
  - Add accessor methods for template identity/data source needed by snapshot builders.
  - Reuse `EditorUiTemplateDescriptor` lookup for drawer `ui_document` when possible; do not introduce plugin-specific runtime object sharing.
  - Update acceptance notes for template-backed drawer validation.
- Testing stage `M3 Component Drawer Protocol Testing`:
  - Run `rustfmt --edition 2021 --check zircon_editor/src/core/editor_extension.rs zircon_editor/src/ui/host/editor_extension_registration.rs zircon_editor/src/tests/editor_event/runtime.rs`.
  - Run focused editor event tests matching `component_drawer`.
  - Debug/correction loop: if a test passes without exercising the new metadata, strengthen assertions before editing production code further.
  - Exit evidence: descriptor protocol tests prove lookup, validation, and capability gating.

## M4 Inspector Projection And Fallback

- Goal: Inspector snapshots and Slint host projection must consume enabled Component Drawer template descriptors for plugin components and safely fall back to generic protected rows when templates are unavailable.
- In-scope behaviors: dynamic component snapshots carry drawer template metadata; Inspector payload/view data preserves metadata; projected host nodes mark custom drawer identity/action route; missing/unloaded drawer still disables editing with warning diagnostics.
- Dependencies: M3 descriptor protocol and current Inspector pane payload/projection pipeline.
- Implementation slices:
  - Add snapshot fields to `InspectorPluginComponentSnapshot` and downstream payload/view structs for `drawer_ui_document`, `drawer_controller`, and template/binding action IDs.
  - Resolve enabled `ComponentDrawerDescriptor` for each selected dynamic component in `editor_state_snapshot_build.rs` using runtime extension access rules.
  - Update pane payload builders and projections to carry drawer metadata without changing core scene ownership.
  - Update Slint host conversion to add custom-drawer marker/header/action nodes before generic property rows, preserving fallback diagnostics when unavailable.
  - Add pane presentation tests for template metadata round-trip and missing-drawer fallback.
- Testing stage `M4 Inspector Projection Testing`:
  - Run `rustfmt --edition 2021 --check` on touched editor snapshot/projection files.
  - Run focused tests matching `inspector` and `pane_payload` where available.
  - Run `cargo test -p zircon_editor --lib pane_payload_builders_emit_stable_body_metadata_for_first_wave_views --locked --jobs 1 --message-format short --color never`.
  - Debug/correction loop: diagnose lower snapshot/payload struct mismatches before changing Slint projection code.
  - Exit evidence: tests prove metadata survives snapshot to host contract and fallback remains protected.

## M5 Milestone Validation And Documentation

- Goal: Finish the Full Milestone with complete docs, formatting, diff hygiene, focused Cargo validation, and a residual gap report.
- In-scope behaviors: docs headers are current; plan sources and test commands are recorded; no malformed generated files; remaining platform host limitations are explicit.
- Dependencies: M1-M4 complete.
- Implementation slices:
  - Update `docs/engine-architecture/runtime-editor-pluginized-export.md` frontmatter and content for platform host artifacts and drawer template protocol.
  - Update `docs/editor-and-tooling/editor-command-workflow.md` frontmatter and content for Inspector template projection and fallback.
  - Update `tests/acceptance/editor-component-drawer-bindings.md` with final command results.
  - Update `.codex/sessions/20260503-2112-export-hosts-component-drawer-design.md` with implemented modules, tests, and residual warnings.
  - Run `git diff --check` after formatting.
- Testing stage `Full Milestone Validation`:
  - Run `rustfmt --edition 2021 --check` on all touched Rust files.
  - Run `git diff --check`.
  - Run focused runtime export tests added in M1.
  - Run focused editor component drawer and pane projection tests added in M3/M4.
  - Run scoped `cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never` and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` if disk/time constraints allow.
  - Record any failures that predate or are isolated from this milestone instead of silently claiming green.

## Expected Residual Gaps To Report

- Android/iOS scaffolds will be source-package host scaffolds, not signed store-ready Gradle/Xcode builds unless the implementation explicitly adds those packaging toolchains.
- WebGPU/WASM scaffolds will define browser host entry assets and fetch/resource bundle contract, not a fully optimized CDN/deployment pipeline.
- Component Drawer templates will be declarative host projection metadata and validated operation bindings, not arbitrary native plugin UI code execution.
- Runtime host ABI still needs future deepening for full mobile lifecycle, touch/keyboard/canvas event parity, suspend/resume, and browser fetch streaming if not completed by the milestone implementation.
