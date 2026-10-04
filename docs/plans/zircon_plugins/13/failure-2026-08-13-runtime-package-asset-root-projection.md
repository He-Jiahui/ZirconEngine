---
handoff_kind: failure
status: open
created_at: 2026-08-13
summary_slug: runtime-package-asset-root-projection
origin_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
fixing_plan: docs/plans/zircon_plugins/13-standalone-plugin-build.md
origin_child_dir: docs/plans/zircon_runtime/runtime/09
fixing_child_dir: docs/plans/zircon_plugins/13
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/dynamic_api/session/linked_plugins.rs
  - zircon_runtime/src/plugin/runtime_plugin/registration_report.rs
  - zircon_runtime/src/asset/project/manager/package_assets.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter woc_project_ui_surface_runtime_round_trip
---

# Plugins 13: runtime plugin package asset roots are absent from session startup

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 来源执行切片：project-authored retained UI runtime bridge
- 修复责任计划：`docs/plans/zircon_plugins/13-standalone-plugin-build.md`
- 交接原因：Plugin distribution and runtime startup own the physical package root. Runtime09 owns consumption of an already registered project/package asset registry and must not infer package paths from component IDs or `.zui` import strings.

## 失败现象与复现证据

`ProjectManager::register_package_asset_roots(...)` and its registry scan already make `package://<package>/.../*.zui` artifacts visible to the runtime UI prototype store. `RuntimeDynamicSession` receives only `RuntimePluginRegistrationReport`, whose package manifest declares logical `asset_roots`, but the report has no resolved physical package root. Consequently session startup cannot register a linked package's assets before `ProjectManager::scan_and_import()`, and a project root importing a linked plugin component cannot resolve that component document.

## 最低共享层根因

The standalone/native plugin discovery and registration projection drops the resolved package directory between manifest discovery and `RuntimePluginRegistrationReport`. This is lower than the Runtime09 UI bridge: the bridge has no authoritative source from which a filesystem path can be derived safely.

## 架构修复验收

- A selected linked plugin registration carries a canonical package root together with its manifest asset roots.
- Runtime project startup registers each selected package root before project asset scanning, so `package://` `.zui` documents enter the same `ProjectManager` registry as project assets.
- A project `.zui` import of a linked plugin component builds one retained runtime surface and passes render plus accessibility extraction without a path fallback.
- The original Runtime09 project UI regression remains runnable through the coordinator-managed validation path.
- Closeout requires adding and actually executing a focused linked-plugin component regression (originally listed as `project_runtime_ui_loads_linked_plugin_component_asset`): the filter currently matches no test and must not be treated as a passed acceptance command. Validate the new test together with the existing Runtime09 regression using coordinator-assigned Windows targets and `--locked`.

## 禁止临时方案

- Do not derive a package filesystem path from a package ID, component ID, manifest string, current directory, or environment variable.
- Do not copy plugin UI assets into the project tree, add a second UI asset registry, or special-case a plugin component in Runtime09.
- Do not weaken package URI or import validation to hide an absent package root.

## 修复结果与回传

Open state: `待修复`; Runtime09 continues independent UI lifecycle, input, extraction, and declared-root work.

## 2026-09-24 current-source and acceptance receipt

- `RuntimePluginRegistrationReport` still exposes the logical `package_manifest` but no canonical physical package root; its plugin/native constructors and `LinkedRuntimePluginPlan::prepare` therefore cannot hand a selected linked package directory to the runtime project path. `ProjectManager::register_package_asset_roots` exists, but the current session construction has not registered selected packages before the project UI surface load. This is a source implementation gap, not a validation-only return.
- The original `tests` field named a non-existent `project_runtime_ui_loads_linked_plugin_component_asset` filter. The real `woc_project_ui_surface_runtime_round_trip` test exists in `dynamic_api/session/tests/runtime_ui_surface.rs`; the former name is retained above as a required new regression, not a runnable or passing test. The original command intent is preserved here; no dynamic pass is claimed.
- `registration_report/plugin.rs`, `registration_report/native.rs`, `dynamic_api/session/construction.rs`, and `dynamic_api/session/runtime_ui.rs` have pre-existing changes outside this Session's scope. The runtime source fix awaits audited ownership and an exact source snapshot; none of those changes is claimed or overwritten here. Managed Cargo is also not submitted on the dirty external `E:/Git/zr_vm` checkout. Keep this failure open until source, both executable tests, upward UI acceptance, and independent review pass.
