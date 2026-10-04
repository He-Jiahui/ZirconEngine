---
handoff_kind: failure
status: open
created_at: 2026-09-28
summary_slug: export-portable-surface-target-host-lifecycle
origin_plan: docs/plans/zircon_plugins/09-export-publishing.md
fixing_plan: docs/plans/optimize/zircon_runtime/57-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-review.md
origin_child_dir: docs/plans/zircon_plugins/09
fixing_child_dir: docs/plans/optimize/zircon_runtime/57
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/runtime_api/session/viewport.rs
  - zircon_app/src/entry/runtime_entry_app/window_surface/native_target.rs
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
tests:
  - cargo +1.94.1 test -p zircon_runtime_interface --lib --locked runtime_native_surface_target_constructors_preserve_handles -- --test-threads=1
  - cargo +1.94.1 test -p zircon_app --lib --locked explicit_native_surface_failure_branches_remain_fail_closed_before_fallback -- --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked bind_viewport_surface_rejects_unsupported_surface_target_after_session_action_admission -- --test-threads=1
---

# Runtime57: portable export surface target and host lifecycle

## 来源执行者

- 来源计划：docs/plans/zircon_plugins/09-export-publishing.md
- 来源执行切片：Plugins09 WOC mobile/browser generated host live-session and real-surface acceptance
- 修复责任计划：docs/plans/optimize/zircon_runtime/57-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-review.md
- 交接原因：The shared App/runtime surface identity and lifetime contract cannot admit a qualified mobile or canvas host target.

- Origin: Plugins09 generated mobile/browser host acceptance, recorded in `docs/plans/zircon_plugins/09/failure-2026-07-17-woc-mobile-browser-host-noop.md`.
- Fixing owner: Runtime57 platform host, App-to-runtime surface identity, binding and lifetime; the Runtime191 current working tree review supplies the latest architecture evidence. The separate Runtime90 backend handoff is `docs/plans/optimize/zircon_runtime/90/failure-2026-09-28-export-portable-rhi-wgpu-surface-present.md`.
- This child record carries the open link; the plan definitions remain unchanged.

## 失败现象与复现证据

The generated Android, iOS and browser hosts cannot bind a live product surface. `ZrRuntimeNativeSurfaceTargetV1` has only NONE and WIN32 kinds and stores two unqualified `u64` handles; the App raw-window adapter accepts only Win32 plus a Windows display handle; the dynamic bind path also accepts only Win32. An Android native window, iOS view/layer or browser canvas cannot be represented with a qualified owner lifetime. A retained `ProductComposition` therefore does not make input, resize or a frame callback reach a presentable runtime surface.

At the read-only 2026-09-28 boundary, the exact source SHA-256 values were: `viewport.rs` `23de5d37bbd22a876ddbfa2fac2944cf14afb2254550f4a7f7feb67e9a4664dc`; App `native_target.rs` `b47a6f5e546665e7403d068d78248e77fb78989caddbeb4f953d4a69a775b8b2`; dynamic `surface.rs` `5327ad19929a3fb955e4f1bec55be73a4f8429ee565536ba271f8c61cd29125a`; Graphics `viewport_surface.rs` `3e1800124ed07dbe9386df45791c5fd767a43aec4314f50aa5de94116a2fed8f`. These are source observations, not dynamic acceptance.

The three frontmatter commands reach existing negative/Win32 contract tests. They do not prove portable target admission. The Plugins09 live mobile/browser filters named by its open failure are absent from current source; neither they nor a real target launch have passed.

## 最低共享层根因

The ABI target and App/dynamic bind path express only Win32 raw handles without a generation-qualified host surface lease. The portable RHI acquisition gap is separately owned by Runtime90.

## 架构修复验收

- Define a host-owned, generation-qualified surface identity for Android, iOS and browser canvas, with explicit bind, resize, suspend/context loss, unbind and destruction rules. Preserve native/JS object lifetime through each frame and reject stale generations after window or view recreation.
- Wire App target admission and dynamic runtime/Graphics routing to that identity. A resize must reach the current surface generation and subsequent frames must use the new physical extent. Every admitted bind must have one terminal release path.
- If the public `repr(C)` DTO or table layout changes, follow Runtime10's ABI version gate and update producers, consumers and tests together. Runtime90 owns native RHI/WGPU acquisition and present; Plugins09 owns generated host input, frame loop, resource delivery and launched-product acceptance.
- Add focused ABI, App and runtime tests for Android/iOS/canvas target admission, invalid owner and stale generation rejection, resize, suspend/destroy/rebind and release. Run the existing lower guards above, then the new exact tests with nonzero selected counts, followed by the original Plugins09 generated-host tests and real Android/iOS/browser product launches. The product gate needs at least two nonblank frames, resize/DPI, pause or visibility change and clean shutdown on each actual target.

## 禁止临时方案

Do not encode borrowed native or JavaScript objects as fake Win32 integers, return success without binding a surface, duplicate the surface owner in a generated template, or bypass the ABI version gate. This failure remains open; no Cargo, product run, failure return, review or closeout pass is claimed.

## 修复结果与回传

Open state: pending repair and managed acceptance; no pass is claimed.
