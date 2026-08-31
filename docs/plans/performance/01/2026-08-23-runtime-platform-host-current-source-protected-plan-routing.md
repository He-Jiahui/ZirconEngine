---
title: Runtime Platform Host Current Source Protected Plan Routing
date: 2026-08-23
status: routing_only
related_report:
  - docs/plans/performance/01/2026-08-23-runtime-platform-host-current-source-performance-adoption.md
protected_targets:
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
---

# Runtime Platform Host Current Source Protected Plan Routing

本记录只提供受保护台账与其他计划的归并输入；本轮没有修改受保护文件。

## `review.md` 建议

暂不写入。Platform 52/52 已完成 current-source 静态组合复验，但 Runtime116 的 2 个 P0、Runtime45 的 5 个 P0、40+40 个资格门及全部动态证据仍未闭合。

## `pending.md` 建议

`zircon_runtime/src/platform`：静态复验完成；等待 capability truth、Window/Display identity、application lifecycle、SurfaceLease、per-window scheduler 和 Preferences durability/multi-process hard cut，以及 managed Windows/WPR/RenderDoc 证据。

## 唯一 owner 路由

| 问题 | 目标计划 |
|---|---|
| Platform Host、Window Registry、Display Topology、event loop、lifecycle、SurfaceLease、host command | Runtime116 (`99q...current-source-review.md`) |
| Preferences scope/storage/overlay/bounded I/O/durability/multi-process/product consumer | Runtime45 |
| product bootstrap/event loop/dynamic runtime/shutdown | App01 |
| RHI surface/present/device generation/HDR | Runtime09A/90 |
| stable identity、dynamic session、foreign ABI | Runtime24/43 + Runtime Interface plans |
| module/capability/service truth | Runtime42/46/50 |
| generated mobile/browser callback P0 | Tooling03 `TOOL-EXPORT-P0-005` |

不得新增第二套 window map、surface wrapper 或 event pump facade；不得以 compile feature、窗口成功创建、CloseRequested 单一路径、固定 viewport 能显示一帧或失焦 10Hz 作为 Platform Host Ready 证明。

## 晋级门

1. disabled/unsupported 不创建 backend/window/surface owner，capability 绑定 observed provider/generation/evidence。
2. native WindowId/DeviceId 不再丢弃；unknown/stale event、command、surface lease fail closed。
3. OS Destroyed、Suspend、DestroySurfaces、Exit 全部先 retire surface lease，再释放 native window，fault/soak 无 stale present。
4. WPR 证明 per-window scheduler 在 idle/occluded/background 下无 wake storm，活跃 workload 无 lateness/backlog/starvation。
5. RenderDoc 绑定同代 current-source artifact，证明 resize/recreate/present generation 与 GPU frame；CPU/功耗仍以 WPR/ETW 为准。
6. Preferences 按 Runtime45 完成 durability、hung I/O、multi-process 与产品 consumer 资格；两处外来格式 dirty 不计进展。

