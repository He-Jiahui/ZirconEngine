---
title: Plugin Native Window Hosting Current Source Review
date: 2026-08-24
scope:
  - zircon_plugins/native_window_hosting
status: static_complete_product_and_dynamic_pending
canonical_owner:
  - docs/plans/optimize/zircon_plugins/03-desktop-export-native-window-source-dist-provider-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
---

# Plugin Native Window Hosting Current Source Review

## 1. Coverage and currentness

The current Rust surface is **5/5 files**, **365 physical / 328 non-empty lines**, **13,801 bytes**, and **5 test markers**. Its workspace-relative `path + LF + decoded text + LF` SHA-256 is `1bd470077898b0b138c6dccee55e34f98594788feadcad629be3eac315cd6bf9`. The generated manifest, both Cargo manifests, first-party Editor catalog, Editor subsystem/capability/view filtering chain and Plugins03 owner report were also read. The plugin directory is clean in the shared worktree.

Commit `08094b9b9` removed the missing `authoring.zui`, duplicate core-owned surfaces and eight phantom contributions per registration. The current regression models 1,000 registrations and reports **8,000 -> 0 contributions** plus **1,000 -> 0 template resolutions**. That is a valid work-scale reduction and closes the old missing-resource/phantom-authoring branch, although the Rust test has not been executed in the unavailable managed Cargo lane.

## 2. Primary finding

The plugin now publishes one feature capability while intentionally contributing no view, drawer, template, menu, command, provider or behavior. Its dist is likewise metadata-only: no command/event/bridge/host-ready/state/unload callback and an empty serialized extension set. The first-party Editor catalog only links Navigation and Neural, so project selection cannot reach this source registration.

Meanwhile `EditorSubsystemReport` treats `editor.extension.native_window_hosting` as a known optional subsystem and enables every known subsystem when configuration is absent. `EditorCapabilitySnapshot` merges that string into available capabilities, and `builtin_view_descriptors()` uses it to admit Workbench, Prefab and Material windows. Core therefore reports and consumes a package-named capability without package selection, provider initialization or health.

This is not a hot-loop problem inside the five plugin files. The real performance/lifecycle owner is the core winit/presenter/window-host path. Tuning empty registration or caching its manifest would preserve the false-ready architecture. The next change must split host-service truth from package-feature truth.

## 3. Unreal source constraints

Unreal's `FSlateApplication` owns the platform window service. It adds a logical window to its routing list before native initialization so immediate OS messages can be resolved, maps parent/size/position/input/DPI policy into a native definition, initializes the platform window, and creates the renderer viewport through the same owner. Destruction is queued child-first, removes modal/menu state, releases renderer resources **before** destroying the native window, removes the logical mapping and only then evaluates application exit. Close requests can be vetoed by the viewport.

The transferable contract is one platform/window service owner with explicit logical/native identity, event routing, render-surface lifetime, parent/modal ordering, close veto and ordered teardown. A plugin may provide policy or optional feature integration, but it cannot truthfully claim to provide native hosting when all handles, callbacks and rendering remain core-owned and its own unload hook is empty.

## 4. Dependency-ordered plan

### M0: capability truth hard cut

Introduce separate identities such as `host.window.native.v1` for the App/platform service and `editor.feature.native_window_hosting` only if a package supplies additional behavior. Configuration may request a feature but cannot create provider capability. Default MVP windows depend on the host service actually initialized by App, not a package-named string. Add failing tests for package absent, selected-but-unlinked, provider faulted and host-service unavailable states.

### M1: decide whether the plugin exists

If native window hosting is permanently a core App/Editor service, delete the package feature, dist and catalog identity after migration. If the plugin remains, define a concrete `NativeWindowFeatureProvider` that maps logical window policy onto the host service and owns initialize/health/quiesce/unload receipts. A zero-contribution registration cannot publish Ready.

### M2: source/dist behavioral equivalence

Generate one versioned contribution/behavior bundle. Source and dist must materialize the same provider contract. The ABI needs generation-scoped callbacks, window/provider leases, explicit quiesce and unload; active windows or render callbacks make `is_stateless=true` invalid.

### M3: bounded multi-window execution

Keep OS event pumping and native handle ownership in App/platform. Editor publishes dirty window/surface generations so stable windows perform no rebuild or redraw work. Reconcile create/resize/focus/DPI/redraw/close in bounded batches, render only dirty surfaces, and retire presenter/swapchain resources after the correct GPU fence.

### M4: lifecycle and performance qualification

Exercise main, floating, modal and child windows across create/show/focus/drag/resize/DPI/monitor/close/device-loss/provider-disable/shutdown. WPR/ETW measures event-loop CPU, UI/render thread work, lock/wait, window messages, allocations, RSS and energy. RenderDoc verifies each current-source window's surface, draw count, pipeline/resource lifetime and pixel parity.

## 5. Acceptance

1. Package absent or selected-but-unlinked publishes zero package-feature capability. A successfully initialized App host publishes the separate host-service capability and a typed health generation.
2. Every logical window maps to at most one live native handle and one presenter/surface generation; create/destroy and provider enable/disable are idempotent.
3. Renderer/swapchain resources retire before native handle destruction and only after GPU ownership clears. No callback enters unloaded provider code.
4. Child/modal ordering, close veto, focus/capture, DPI/monitor migration and device-loss recovery preserve one terminal lifecycle per window.
5. A stable, occluded or minimized window causes zero layout/paint/present work except explicitly required platform pumping. Dirty work scales with changed windows, not all registered windows.
6. Record `1/4/16` window CPU p50/p95/p99, input-to-paint, resize-to-present, close latency, event counts, redraw/present counts, allocations, peak RSS/GPU memory and energy on one source/executable fingerprint.
7. Compare with Unreal only on matched window count, size, renderer/backend, refresh policy and hardware; source inspection alone cannot establish parity.

## 6. Validation status

- Static per-Rust-file review: **5/5 complete**.
- Phantom authoring work: current source models **100% removal**, but the Rust regression is unexecuted.
- Global plugin structure audit: **pass** for manifest/schema/registration/dist boundaries; it does not prove provider behavior.
- `rustfmt --check`: **fail** on export ordering and one assertion layout in clean files; no unrelated formatting edit was made.
- Cargo, App/provider integration, real OS-window lifecycle, WPR/ETW, RenderDoc and power validation: **pending**.
- This module is not eligible for protected-ledger acceptance, milestone commit or WeCom completion notification.
