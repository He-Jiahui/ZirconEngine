---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: woc-runtime-host-client-server-extensibility
origin_plan: docs/plans/woc/00-woc-engine-capability-foundation.md
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
origin_child_dir: docs/plans/woc/00
fixing_child_dir: docs/plans/zircon_runtime/runtime/10
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/entry_runner/headless.rs
  - zircon_app/src/entry/runtime_entry_app
  - zircon_runtime/src/dynamic_api/session/profile.rs
  - zircon_runtime/src/dynamic_api/session/linked_session.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
tests:
  - cargo test -p zircon_app linked_project_runtime_accepts_custom_gameplay_plugin --locked
  - cargo test -p zircon_app server_runtime_host_ticks_until_shutdown_without_window --locked
  - cargo test -p zircon_runtime headless_profile_selects_server_runtime_target --locked
---

# Runtime 10: normal runtime host cannot run a shared native client/server gameplay package

## 来源执行者

- 来源计划：`docs/plans/woc/00-woc-engine-capability-foundation.md`
- 来源执行切片：WOC engine capability assessment / MVP foundation
- 修复责任计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 交接原因：The lowest shared cause is the Runtime 10 dynamic-session/ABI host boundary and its Zircon App consumer, before WOC simulation or network protocol code.

## 失败现象与复现证据

WOC uses one deterministic simulation unchanged in offline, authoritative server, and headless RL modes. Zircon can register native runtime plugins and `create_linked_runtime_session()` accepts registration reports, but the normal public desktop runner loads the default runtime library and exposes no custom linked-registration entry point for a project gameplay package.

The server path is not executable as an application host:

- `RuntimeDynamicSessionProfile::Headless::target_mode()` returns `ClientRuntime`;
- `EntryRunner::run_headless()` bootstraps a core and immediately returns;
- the headless runtime-entry configuration uses a no-window winit loop, but there is no public authoritative server runner with shutdown control or server target selection;
- export bootstrap returns a `CoreHandle`, not a live project session/frame loop.

The dynamic client session also always installs `RuntimeCameraController`, which mutates the active scene camera on right/middle drag and wheel input. A game package cannot configure or disable that development orbit behavior through the project contract.

## 最低共享层根因

Zircon has plugin registration, dynamic ABI, and a window host as separate pieces, but no product-level runtime host contract that composes a project root, arbitrary linked gameplay registrations, client/server target mode, a live tick loop, input/UI policy, and controlled shutdown.

## 架构修复验收

- Provide a public runtime host entry that runs the existing desktop event loop with caller-supplied runtime plugin and feature registration reports plus a project root.
- Provide a real `ServerRuntime` host that runs fixed updates without a window or renderer until explicit shutdown, supports deterministic/manual stepping for tests, and does not return immediately after bootstrap.
- Ensure client, offline, server, and headless test hosts can load the same native gameplay/simulation package without WOC-specific engine branches.
- Expose an explicit camera/input policy; project gameplay can disable the implicit orbit camera and own camera controls while raw input remains available.
- Preserve the stable ABI boundary and host-request flow; do not create a second app/runtime truth.
- Add executable client and server fixtures proving a custom linked plugin registers systems/resources, ticks the loaded project, and shuts down cleanly.

## 禁止临时方案

- Do not add WOC-specific code to `dynamic_api`, `EntryRunner`, or the builtin plugin catalog.
- Do not run the authoritative server as a hidden graphical client profile.
- Do not call one bootstrap tick or a filtered registration test a live server host.
- Do not duplicate the simulation between client and server packages.

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
