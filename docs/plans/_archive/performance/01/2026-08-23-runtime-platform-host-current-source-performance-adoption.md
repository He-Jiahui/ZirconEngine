---
title: Runtime Platform Host Current Source Performance Adoption
date: 2026-08-23
scope:
  - zircon_runtime/src/platform
status: static_complete_dynamic_pending
source_fingerprint: 887ffbf10a7a8a2fa674c8c4dee1737291fceab3f4fb73f0cc760831395d2e7d
canonical_owners:
  - docs/plans/optimize/zircon_runtime/99q-runtime-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericApplication.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericWindow.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericWindowDefinition.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/Windows/WindowsApplication.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/Windows/WindowsApplication.h
  - dev/bevy/crates/bevy_winit/src/winit_windows.rs
  - dev/bevy/crates/bevy_winit/src/winit_monitors.rs
  - dev/godot/servers/display/display_server.h
  - dev/godot/platform/windows/display_server_windows.cpp
---

# Runtime Platform Host Current Source Performance Adoption

## 1. 当前覆盖

`zircon_runtime/src/platform/**` 当前共有 **52/52 个 Rust 文件、8,190 physical lines / 7,542 non-empty lines、282,537 bytes、94 个 test marker**。按“相对路径 + NUL + 原始 bytes + NUL”的有序 SHA-256 为 `887ffbf10a7a8a2fa674c8c4dee1737291fceab3f4fb73f0cc760831395d2e7d`。

Runtime116 已在同一 current working tree 对 Window/Platform production、App host、dynamic ABI/surface、focused tests 与五引擎参考做 E3 current-source 审查；Runtime45 覆盖 Preferences contract、bounded I/O、overlay、atomic backend 与产品 consumer。当前只有 `preferences/atomic_file.rs` 和 `tests/preferences.rs` 两处 foreign dirty，差异为 rustfmt import/断行，不改变语义，本轮保留且不接管。

因此 52/52 采用组合证据完成静态复验：Window Host 结论归 Runtime116，Preferences 结论归 Runtime45。本报告只定义 performance 执行顺序和动态门，不复制两套 P0/P1 账本。

## 2. MVP 结论

Zircon 已有可运行的单窗口底座：winit `ApplicationHandler`、`WindowDescriptor`、Win32 raw-handle surface、默认 viewport bind/unbind、CPU fallback presenter、IME/cursor/gamepad host request，以及 reactive/continuous/low-power/fixed cadence。失焦 Game cadence 已降到约 10Hz，continuous pump 也会清除已合并 request bit，这些局部功耗修正应保留。

但 Platform Host 仍有两个本地 P0，必须先于性能微调：

1. `PlatformConfig.enabled` 不参与 capability admission。matrix 只依据任意 `PlatformTarget` 和 compile feature 就把 window、monitor、event loop、lifecycle、metrics 报为 `Supported`，即使 backend 未安装、未初始化、未观测或明确 disabled。
2. Graphics 通过 `create_surface_unsafe` 保有 native handle，而 App 的 `WindowEvent::Destroyed` 只分发事件，不先 unbind/drop surface、presenter 和 window；`suspended`、`destroy_surfaces`、`memory_warning`、`exiting` 也没有汇入同一 transaction。CloseRequested 的显式路径不能覆盖 OS Destroyed 和移动 surface lifecycle。

当前 `window_event` 仍丢弃 winit `WindowId`，`device_event` 丢弃 `DeviceId`；所有动态 ABI surface 请求仍只接受 default viewport。没有 Window Registry、Display Topology generation、SurfaceLease 或 per-window event loop scheduler，就无法判断“扫描/触发频率过高”属于哪个 owner，也无法安全地并行或降频。

## 3. 性能问题的正确排序

| 顺序 | 当前问题 | 性能/功耗影响 | Owner |
|---|---|---|---|
| P0 | capability 把 compiled/selected 当 Ready | disabled/headless/fallback 仍可能创建无用 owner、线程、窗口或轮询 | Runtime116 M116.1 |
| P0 | raw surface lifetime 未随 Destroyed/Suspend 收口 | stale present、设备/表面错误、重复 bind/recreate 与不可控恢复开销 | Runtime116 M116.5 |
| P1 | 全部 event 丢失 WindowId 并路由 viewport 1 | 无法做 per-window invalidation、frame demand、input batching 和公平调度 | Runtime116 M116.2/4 |
| P1 Partial | cadence 有 coalescing/低功耗模式，但无 wake source/deadline clock/lateness/backlog/starvation | 空闲功耗已有改善，复杂 workload 仍可能主线程 wake storm 或饥饿 | Runtime116 M116.4 |
| P0/P1 | Preferences 单 global active lane、hung backend、虚假 Durable、全局 root、无 CAS | 一个 I/O 可阻塞全部 key 和 shutdown；多进程丢更新 | Runtime45 |

先修 identity/lifecycle/capability truth，不是回避性能：没有真实 owner/generation，WPR 看到的 wake、present、resize、I/O 和 thread 无法归因；没有 surface lease，任何“减少 rebinding”都可能把 use-after-destroy 风险藏起来。

## 4. Unreal 源码依据

Unreal ApplicationCore 通过 `FGenericApplication`/`FGenericWindow`/`FGenericWindowDefinition` 分离 application、window definition、native window 与 message handler；Windows backend 的 `FWindowsApplication` 创建/初始化真实 `FWindowsWindow`，按 HWND/window 对象处理 native message、display metrics 和 lifecycle。其目标不是接口数量，而是每个事件有明确 window owner，application/window/display 状态可以分别观察和销毁。

Unreal 的 message pump 与 window lifecycle 还保留周期统计和明确清理路径；这说明 Zircon 应把 event-loop wake、window command、surface availability 和 application state 纳入一个可观察 scheduler/transaction，而不是把所有 winit callback 折叠到固定 viewport。Bevy 的 `WinitWindows` native/entity 双向映射与 monitor owner、Godot `DisplayServer` 的 per-window/display API 作为轻量旁证；架构上限仍采用 Unreal 的 ApplicationCore owner/lifecycle 分层。

## 5. 计划采纳

### M116.0：Truth Freeze 与 Unsafe RED

建立 disabled/compiled/installed/observed/ready 差异、OS Destroyed 后 present、suspend 时 surface 存活、resize 重复 bind 的 RED。冻结 WindowId/DisplayId/SurfaceLease/HostCommand/lifecycle ABI。

### M116.1-3：Host、Window Registry 与 Display Topology

Platform backend instance 声明线程亲和、health、quiesce 与 terminal result；capability 只由 observed owner 发布。建立 stable WindowId/native 双向映射、generation、role/relationship 与 per-window command receipt；DisplayTopology 用 immutable generation/diff 表示 hotplug、DPI、HDR、mode 与 placement。

### M116.4：Event Loop Scheduler

统一 frame demand、window command、timer、input、background work 与 lifecycle wake source。保留当前 cadence coalescing/失焦降频，补充 wake identity、clock domain、deadline、lateness、backlog、starvation 和 per-window fairness；避免“每有事件就整帧 redraw”或无来源的持续轮询。

### M116.5：Surface Lease Hard Cut

`SurfaceLease { window_id, window_generation, output_generation, surface_generation }` 贯穿 App、dynamic ABI、Graphics。Destroyed/Suspended/DestroySurfaces 必须先 stop submit、quiesce/fence、unbind/drop lease，再释放 native window；resize 使用 prepare/fence/publish/retire，不反复无代次 bind。

### M116.6-7：Product Cutover 与资格

App、Editor、headless、export host 共享真实 Platform Host contract。多窗口、多显示器、DPI/hotplug、系统 destroy、suspend/resume、surface loss、device restart、soak 全部通过后，才比较 frame pacing、event latency、CPU/RSS 与功耗。

## 6. 动态检测计划

1. WPR/ETW：CPU sampling、DPC/ISR、context switch、thread lifetime、input/window message、timer resolution/wake、file I/O、RSS、energy；按 window/surface/lifecycle generation 输出 marker。
2. Cadence：focused/unfocused/occluded/minimized/headless，reactive/continuous/low-power/fixed，窗口 `1/4/32`；记录 wake/frame demand/coalesced/replaced、lateness p95/max、starvation、CPU 与 package power。
3. Lifecycle：create/resize/DPI/move/monitor hotplug/close/OS destroy/suspend/resume 循环与 fault injection；记录 bind/recreate/retire count、stale rejection、surface recovery time 和 leaked owner count。
4. RenderDoc：只在 current-source product executable 上检查 surface generation、present、resize 后 render target/pipeline state 和非空帧；它不能证明 event-loop CPU、I/O 或功耗。
5. Preferences 另按 Runtime45 做 hung I/O、flush durability、多进程 CAS、cold read、quota/RSS；不能把两处 rustfmt dirty 当作功能进展。

当前没有受管 Windows current-source executable，故 Cargo、真实窗口、WPR/ETW、RenderDoc 均为 0。静态复验可以完成，性能资格继续 pending。

## 7. 本轮结果

- Platform 52/52 Rust 文件由 Runtime116 + Runtime45 current working tree 组合覆盖。
- 当前 source fingerprint 已记录；2 个 foreign dirty 仅格式变化并被保留。
- 生产代码、测试、Cargo、ABI 改动为 0；2 个 Platform P0、5 个 Preferences P0 保持原 owner open。

