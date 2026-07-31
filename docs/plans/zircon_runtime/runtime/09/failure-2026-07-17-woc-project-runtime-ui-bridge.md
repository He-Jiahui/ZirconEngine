---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: woc-project-runtime-ui-bridge
origin_plan: docs/plans/woc/00-woc-engine-capability-foundation.md
fixing_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
origin_child_dir: docs/plans/woc/00
fixing_child_dir: docs/plans/zircon_runtime/runtime/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/hud.rs
  - zircon_runtime/src/dynamic_api/session/menu.rs
  - zircon_runtime/src/dynamic_api/session/preview.rs
  - zircon_runtime/src/ui/surface
  - zircon_runtime/src/plugin/extension_registry/apply_to_world.rs
tests:
  - cargo test -p zircon_runtime woc_project_ui_surface_runtime_round_trip --locked
  - cargo test -p zircon_runtime woc_project_ui_input_render_accessibility_share_surface --locked
---

# Runtime 09: project-authored retained UI is not connected to the game runtime

## 来源执行者

- 来源计划：`docs/plans/woc/00-woc-engine-capability-foundation.md`
- 来源执行切片：WOC engine capability assessment / MVP foundation
- 修复责任计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 交接原因：The generic retained UI implementation is Runtime 09-owned; the failure is its missing project-runtime lifecycle and extraction bridge, not a WOC widget defect.

## 失败现象与复现证据

Zircon has a substantial retained UI foundation: `UiSurface`, templates, layout, hit testing, focus, text editing, navigation, drag/drop, accessibility extraction, component state, and GPU render extraction.

The dynamic game session does not own one. `RuntimeDynamicSession::current_ui_extract()` only checks two hardcoded paths:

1. `runtime_session_menu_extract`, which recognizes `gameplay.menu_state` and contains Vampire-specific default copy;
2. `runtime_session_hud_extract`, which recognizes `gameplay.hud_text` and `vampire.hud_text`.

Input events are submitted to the input manager but are not routed through a project `UiSurface`. Accessibility capture always returns `dynamic_preview_accessibility_snapshot()` with the diagnostic `runtime UI surface accessibility extraction unavailable in dynamic preview`. Runtime extension application installs ECS components/resources/events/systems, but not a project UI surface or UI extract provider.

WOC requires interactive action bars, unit frames, inventory, character sheet, talents, quest log, map, chat, mail, bank, auction house, guild/social views, modal dialogs, mobile controls, and admin-like dense tables. A text HUD and one hardcoded menu cannot express or exercise that surface.

## 最低共享层根因

The retained UI model is implemented as a library subsystem but lacks an authoritative project-runtime owner that loads UI assets, retains component state, routes window/touch/IME/gamepad input, updates bindings, emits `UiRenderExtract`, and exposes the same surface to accessibility.

## 架构修复验收

- A project can declare and load one or more `.zui`/UI package roots into persistent runtime-owned `UiSurface` instances without engine source special cases.
- Runtime input is routed to the UI surface with defined consumption/capture semantics before gameplay input, including mouse, touch, keyboard, IME, gamepad navigation, and resize/DPI changes.
- The rendered UI extract and accessibility snapshot come from the same live surface and reflect focus, bounds, values, and actions.
- Plugin-contributed UI components and project bindings are available in the runtime surface, not only in editor hosts.
- The bridge supports WOC-scale repeated lists, popups, text fields, drag/drop, tooltips, and multi-window state without rebuilding the whole tree each frame.
- Add a product test that loads a project UI with an action bar, inventory grid, chat input, popup, and touch control, drives input, and validates render plus accessibility output.

## 禁止临时方案

- Do not add additional WOC-specific component IDs beside the Vampire menu/HUD checks.
- Do not implement the WOC HUD as ad hoc `UiRenderCommand` assembly inside `dynamic_api`.
- Do not bypass retained input, accessibility, or binding ownership with test-only extracts.
- Do not weaken WOC interaction or visual acceptance to static screenshots.

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
