---
related_code:
  - zircon_app/src/runtime_presenter.rs
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
  - zircon_app/src/entry/runtime_entry_app/surface_present
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/render/16-compute-neural.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/UnrealClient.cpp
  - dev/bevy/crates/bevy_render/src/view/window/mod.rs
  - dev/bevy/crates/bevy_render/src/view/window/screenshot.rs
tests:
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards
  - zircon_app/src/entry/runtime_entry_app/surface_present/redraw.rs unit tests
  - zircon_app/src/runtime_presenter.rs unit tests
  - current-source managed Windows Cargo pending
  - forced-fallback WPR/Tracy/RenderDoc matrix pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App runtime surface present 当前源码性能复审（2026-08-14）

## 范围与证据边界

本轮完整复读 `surface_present/**` 6 个生产文件、`runtime_presenter.rs`、
`frame_capture.rs` 以及 `runtime_entry_surface_present_guards/**` 6 个测试文件，共 **14/14
个文件、1,623 行**。同时沿调用图核对 App `RuntimeSession`、Runtime dynamic session、
`RenderFramework` 同步/异步 capture owner；这些调用图文件不计入 14 文件覆盖数。

当前相关 Rust 文件已有其他 Session 的未提交修改，本轮仅做只读结构复审，不吸收或覆盖其
实现。统一 Windows validator 在生成 Cargo 命令前被外部 unmanaged artifacts 阻断，故本记录
只证明当前源码静态风险，不把历史测试或旧 RenderDoc capture 冒充动态验收。

## 当前调用链与结构性瓶颈

Native surface 成功绑定后，App 直接调用 `present_viewport`，这条主路径没有在 App 侧搬运
RGBA。问题集中在显式强制 capture 或 surface API 不可用的 Softbuffer 回退：

1. 每次 redraw 调用同步 `RuntimeSession::capture_frame`。Runtime dynamic session 会重新
   `current_extract`、`current_ui_extract` 并执行 `submit_extract_with_ui`；framework capture 随后
   `finish_submission`，持 operation/state 锁等待 readback completion，必要时再同步
   `capture_latest_frame`，最后发布 owned RGBA。
2. App 在事件循环线程逐像素把 RGBA 转写为 Softbuffer XRGB surface。完整帧已删除冗余预清零，
   但这只消除一次额外 memset，没有消除同步 readback、整帧转换和主线程提交等待。
3. 首帧 PNG 开启时，`complete_presented_frame` 会再次调用 `capture_frame`。在 Softbuffer 回退的
   同一 redraw 内，这会把 extract、UI 投影、render submission、同步 capture 再执行一次，而不是
   复用刚刚显示的 `RuntimeFrame`。
4. PNG encoder、buffer flush、`sync_all` 与原子替换都在同一事件循环调用栈执行。它只属于显式
   证据捕获，不是稳定帧成本，但必须单列启动/退出延迟，不能混入普通 present 基线。
5. 当前结构测试反而要求 fallback 源码包含 `capture_frame()` 并锁定 native-present 后的调用顺序；
   没有测试每 generation capture 次数、同步 wait、readback bytes、队列上限或首帧重复提交，因此
   这些 source-shape GREEN 不能作为性能验收。

静态 payload 下限如下，尚未计入 GPU readback staging、capture owner/inspection clone、锁等待和
PNG 编码：1080p RGBA 为 **8,294,400 bytes/帧**，4K RGBA 为 **33,177,600 bytes/帧
(31.64 MiB)**。4K/60Hz 仅一次 RGBA 全帧读取加一次 Softbuffer 全帧写入即约
**3.71 GiB/s**；首帧重复 capture 会额外触发一次完整渲染/读回链，但其 GPU pass、copy 和 stall
必须由当前源码实测确定。

## 参考引擎结论

- Unreal `FViewport` 普通绘制在 `UnrealClient.cpp:1868-1882` 完成 viewport draw、显式
  `ProcessScreenShots` 后排队结束/呈现帧；CPU 读回不是普通呈现媒介。其 hit-proxy 路径在
  `1928-2022` 仅在 cache 失效时生成并读取 surface，且源码明确把 stereo 下额外更新判为性能损失。
  这支持“产品 present、显式 capture、缓存型诊断读回”三种行为分离，而不是每帧用 capture
  模拟 present。
- Bevy `view/window/mod.rs:77-101` 持有并直接 `present` swapchain `SurfaceTexture`；
  `view/window/screenshot.rs:55-65` 把 screenshot 定义为显式 component request，并声明异步完成。
  它与 Zircon 已有 GPU product + bounded readback 架构方向一致。

参考源码给出的是所有权与调度边界，不提供可直接照搬的阈值。队列容量、frame age 与功耗预算
必须由 Zircon 的 F2/F4 产品矩阵实测确定。

## 统一优化计划

本问题继续使用既有 **PERF-MVP-023**，不新增重复 failure：Render16 保持唯一
`GpuReadbackQueue` owner；Runtime10 负责把“请求 capture、非阻塞轮询 ready generation、释放
shared owned frame”的版本化 ABI 投影到 App；Render17 负责 CPU/GPU 指标与 capture 证据。

App fallback 只发起有界异步 capture 请求并轮询 latest-ready mailbox；没有新帧时保留上一帧或
按 runtime demand 等待 wake，不忙轮询。每个 ready generation 最多发生一次 GPU readback、一次
ABI owner 发布和一次 Softbuffer 转换，stale generation 可计数丢弃。首帧 PNG 在 fallback 下复用
同一 ready frame；native surface 下仍走独立显式 screenshot 请求。PNG encode/sync 可移到有界
worker，但 `exit_after_presented_frames` 必须等待持久化结果，不能以提前退出伪造成功。

测试应从源码字符串断言升级为行为与 counter：禁止 V1 ABI 原地扩形，禁止 App/Render17 新建
私有 readback ring，禁止把同步 wait 挪到另一主线程函数，禁止删掉 fallback/截图或降分辨率伪造
收益。

## 动态验收矩阵

- forced fallback/native surface，1080p/4K，30/60/120Hz，stable 300 帧；首帧 PNG on/off，
  resize burst、surface unavailable、device loss 与退出等待。
- 记录 `extract/ui/submit/capture requests`、ready/drop/age、`finish_submission`、blocking poll/wait、
  GPU readback bytes、ABI owner bytes、Softbuffer copy passes/bytes、live owners/peak RSS、main-thread
  p50/p95/p99、CPU、GPU span 与功耗。
- fallback 稳态要求每 ready generation：render submission 不因同一 App redraw 重复，capture/readback
  不超过 1，Softbuffer copy 不超过 1；首帧 PNG 不增加第二次 fallback render/capture。队列 entry、
  bytes 与 age 有硬上限，停止消费 60 秒后 RSS 不继续增长。
- native surface 稳态 App RGBA bytes、Softbuffer copy、capture/readback 均为 0；显式 screenshot 仍能
  生成像素等价 PNG。WPR/Tracy 核对事件循环等待与 CPU 栈，RenderDoc 核对同 generation 的 render/
  copy/readback/submit，不能用旧高级 volumetric capture 代替当前 F2 产品帧。

在 managed Cargo、真实 forced-fallback trace、当前 F2 PNG/RDC 和独立审查齐全前，
`zircon_app/src/entry/**` 与 `runtime_presenter.rs` 继续保留在 `pending.md`，不进入 `review.md`。
