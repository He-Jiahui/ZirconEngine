---
related_code:
  - zircon_runtime/src/platform
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
reference_sources:
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
tests:
  - zircon_runtime/src/platform/tests
  - current-source Windows Cargo and focused/unfocused/headless product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime platform逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/platform/**`当前源 **39/39** 个Rust文件、**4,602** 行、**64** 条测试已逐文件阅读：root config/feature/module/service/target 6个、capability matrix/backend/report 10个、feature/cross-target/headless/diagnostic/structure测试23个。`mod.rs`与`module.rs`已有其他会话改动，本轮全程只读保留。

## 性能结论

- 生产层没有窗口创建、event loop pump、输入队列、线程、锁或I/O实现；它只把Copy feature flags与target/mode映射为Copy `PlatformCapabilityReport`，以及按显式请求格式化28条diagnostic String。
- 仓内非测试caller只通过`PlatformConfig::capability_report`和`PlatformManager`取得报告；没有每帧或每事件调用证据。`diagnostic_lines`会分配Vec/String且matrix子查询重复计算window backend，但当前属于低频control-plane，不能冒充产品瓶颈。
- `EventLoopPolicy`只声明Game/DesktopApp/Mobile/Continuous/Headless默认选择，不实际执行cadence。App入口已确认的idle/request-redraw/Continuous Poll问题继续归PERF-MVP-005/424；pointer/gamepad高频问题继续归003/006/426，不在本目录重复建根因。
- 64条测试对8 targets×3 modes、headless synthetic input、feature propagation和diagnostic schema覆盖较完整，但没有产品CPU/wakeup/latency证据。

本轮没有为了“有修改”而改动控制面格式化；简单但无production收益的缓存会引入另一份capability truth，不符合generation owner原则。

## 参考约束与动态验收

Bevy `WinitSettings`把focused/unfocused update mode和Reactive/low-power wait明确放在真实winit owner，并说明device/user/window event的唤醒差异。Zircon当前platform matrix只到枚举，动态验收必须在App/winit owner验证该枚举是否真正映射为相同cadence语义。

需运行current-source platform/package gates，并在Game/DesktopApp/Editor/Headless下记录focused/unfocused 30秒的tick/redraw/wake/CPU、device/window/user event storm与input latency；验收继续复用PERF-MVP-424指标。Cargo和产品trace完成前留在`pending.md`，不得进入`review.md`。
