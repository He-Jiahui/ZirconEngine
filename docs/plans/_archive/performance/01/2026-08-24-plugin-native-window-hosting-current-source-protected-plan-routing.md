---
title: Plugin Native Window Hosting Current Source Protected Plan Routing
date: 2026-08-24
status: routing_only
source_report:
  - docs/plans/performance/01/2026-08-24-plugin-native-window-hosting-current-source-review.md
---

# Plugin Native Window Hosting Current Source Protected Plan Routing

| Existing owner | Required adoption |
|---|---|
| Plugins03 desktop/native-window integration | Mark phantom authoring/missing ZUI closed; retain capability truth, product reachability, core-vs-plugin owner decision and source/dist behavior as open P0 work. |
| App/platform window host | Own `host.window.native.v1`, OS event loop, native handle identity, initialization health and ordered shutdown. |
| Editor13 layout/window lifecycle | Own logical window, parent/modal/focus/DPI/monitor/close policy and provider enable/disable transaction. |
| Editor retained host / Runtime09 UI | Own dirty surface generations, bounded window reconciliation, retained paint/hit identity and stable-window zero-work gates. |
| RHI/presenter owner | Own per-window surface/swapchain generation, GPU-fence retirement, device-loss recovery and RenderDoc acceptance. |
| Plugins01 / Plugins04 | If the plugin remains, own behavior ABI, provider lease, quiesce/unload and source/dist equivalence. Metadata-only dist cannot publish Ready. |
| First-party Editor catalog | Return typed missing-provider faults and bind feature capability only to the admitted provider generation. |

Protected `docs/plans/performance/review.md` and `docs/plans/performance/pending.md` remain untouched. Their owner may add one concise `native_window_hosting` pending entry after adoption. Acceptance requires the owner hard cut plus current-source OS-window, WPR, RenderDoc and power evidence.
