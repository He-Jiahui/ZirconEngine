---
related_code:
  - zircon_runtime/src/platform
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
reference_sources:
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
  - dev/bevy/crates/bevy_winit/src/state.rs
tests:
  - zircon_runtime/src/platform/tests
  - current-source Windows zircon_runtime platform tests pending
  - F0/F2 focused/unfocused idle event-loop trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime platform逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/platform`当前源 **39/39** 个Rust文件、**4,267** 行已逐文件阅读：root 6、`capability` 10、`tests` 23。生产面是target/feature词汇、纯值capability matrix、module descriptor与diagnostic formatting；测试面覆盖desktop/mobile/browser/headless、feature manifest、status token与cross-product。

## 性能结论

未发现独立F0/F2热路径：生产代码没有锁、线程、文件I/O、channel、sleep、poll loop或逐帧队列。`PlatformCapabilityMatrix::report`只做固定字段条件判断，`diagnostic_lines`分配28条String，但当前调用位于启动报告、显式diagnostics snapshot或测试，不是runtime frame/input loop。matrix内部多次调用`window_backend`是常数规模纯值工作，不值得以额外cache/状态复杂化。

`EventLoopPolicy`在此目录仅是policy token；真实`ControlFlow::{Poll,Wait}`与`request_redraw`位于`zircon_app/src/entry/runtime_entry_app`，已由F0/F2 entry审查负责。因此本切片不把policy声明误判成事件循环性能，也不做无收益代码修改。

## 参考引擎对照

Bevy `WinitSettings`区分focused/unfocused `UpdateMode`：游戏focused可continuous，unfocused切reactive-low-power；desktop app focused/unfocused都可reactive，并在state中映射`Wait`/`WaitUntil`/`Poll`。Zircon现有`Game/DesktopApp/Mobile/Continuous/Headless`词汇具备相同分层入口，动态验收必须落到app entry，证明EditorHost idle使用Wait、Client按目标帧策略、失焦不无条件busy-poll。

## 动态验收

当前只完成静态阅读。待受管Cargo运行platform tests与feature matrix，并在Windows client/editor各采样focused/unfocused 30秒：记录event-loop wake、redraw request、frame tick、CPU、timer resolution和input-to-frame p95；DesktopApp idle应由事件/定时器唤醒，Client continuous策略必须有明确帧率/呈现节奏，Headless不得创建window backend。完成前保持`pending.md`，不进入`review.md`。
