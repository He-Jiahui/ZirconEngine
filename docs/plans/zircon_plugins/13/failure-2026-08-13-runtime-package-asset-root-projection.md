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
  - cargo test -p zircon_runtime --locked woc_project_ui_surface_runtime_round_trip
  - cargo test -p zircon_runtime --locked project_runtime_ui_loads_linked_plugin_component_asset
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

## 禁止临时方案

- Do not derive a package filesystem path from a package ID, component ID, manifest string, current directory, or environment variable.
- Do not copy plugin UI assets into the project tree, add a second UI asset registry, or special-case a plugin component in Runtime09.
- Do not weaken package URI or import validation to hide an absent package root.

## 修复结果与回传

Open state: `待修复`; Runtime09 continues independent UI lifecycle, input, extraction, and declared-root work.
