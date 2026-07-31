---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: woc-mobile-browser-host-noop
origin_plan: docs/plans/woc/00-woc-engine-capability-foundation.md
fixing_plan: docs/plans/zircon_plugins/09-export-publishing.md
origin_child_dir: docs/plans/woc/00
fixing_child_dir: docs/plans/zircon_plugins/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/mobile.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/browser.rs
  - zircon_runtime/src/platform/capability/matrix/window.rs
  - zircon_app/src/entry/export_bootstrap.rs
tests:
  - cargo test -p zircon_runtime exported_mobile_host_drives_live_runtime_session --locked
  - cargo test -p zircon_runtime exported_browser_host_drives_live_runtime_session --locked
---

# Plugins 09: generated mobile and browser hosts do not execute the exported game

## 来源执行者

- 来源计划：`docs/plans/woc/00-woc-engine-capability-foundation.md`
- 来源执行切片：WOC engine capability assessment / cross-platform foundation
- 修复责任计划：`docs/plans/zircon_plugins/09-export-publishing.md`
- 交接原因：The generated host templates, packaging matrix, and platform release proof belong to the export/publishing plan rather than WOC application code.

## 失败现象与复现证据

Plugins 09 generates Android, iOS, WebGPU, and WASM project scaffolds, but the generated Rust entry currently defines:

- lifecycle, touch, keyboard, and viewport handlers that ignore every argument and return `true`;
- `zircon_export_start()` that calls a bootstrap function whose local `CoreHandle` is dropped when the function returns;
- no live `RuntimeDynamicSession`, frame loop, render surface binding, or host-owned shutdown state.

The browser template logs that `zircon_host_fetch_resource` requires a generated memory adapter and returns `0`. Browser pointer listeners only cover move, not a complete press/release/cancel lifecycle. The platform capability matrix separately records browser window events, metrics, lifecycle, IME, and pointer backends as unavailable. Android/iOS views forward nominal callbacks but never attach a render surface or drive runtime frames.

Therefore the generated applications can package files and report successful callback returns while displaying no executing WOC client.

## 最低共享层根因

Export planning materializes packaging shells but does not materialize a persistent platform runtime host. The generated native/WASM ABI exports are stubs rather than adapters to one authoritative live session and surface lifecycle.

## 架构修复验收

- Generated Android, iOS, WebGPU, and WASM hosts retain one live project runtime session from startup through teardown.
- Each host binds a real render surface/canvas, advances frames, handles resize/DPI and foreground/background transitions, and presents a nonblank project frame.
- Touch/pointer press, move, release, cancel, keyboard/text/IME, and accessibility-relevant input reach the runtime with stable IDs and correct coordinates.
- Browser asset fetch copies URI bytes through a bounded memory adapter and enforces the export allowlist; it must not return a placeholder zero result.
- Platform packaging smoke tests launch the generated artifact, load a project asset, drive an input action, render at least two frames, and shut down cleanly.
- WOC cross-platform parity remains blocked until these product tests pass on real target environments.

## 禁止临时方案

- Do not accept generated-file string tests as proof that a platform client runs.
- Do not keep callbacks that return success after dropping the runtime state.
- Do not embed the original WOC web client as a substitute for Zircon WebGPU/WASM execution.
- Do not mark mobile/browser parity complete from desktop capture output.

## 修复结果与回传

Current owner note（2026-07-22）：current-source复核仍确认`zircon_export_bootstrap`只保留局部bootstrap owner，lifecycle/touch/keyboard/viewport/fetch ABI均返回占位成功；browser虽已实例化WASM并连出函数名，仍没有authoritative live session/surface/frame ownership。PERF-MVP-548只删除生成器内部一次无意义String rescan，不改变产品能力；本failure保持open且阻止generated host进入性能验收。

Open state: `待修复`; no pass is claimed.
