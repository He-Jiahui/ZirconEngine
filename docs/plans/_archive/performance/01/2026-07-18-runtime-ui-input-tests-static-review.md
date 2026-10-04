---
related_code:
  - zircon_runtime/src/ui/tests/runtime_input_manager.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager
  - zircon_runtime/src/ui/tests/runtime_ui_support
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-entry-events-and-input-routing.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
tests:
  - 10 tracked Rust files and 16 test definitions statically reviewed
  - no sleeps, ignored tests, filesystem access, or worker spawning found
  - current-source Cargo input batch pending behind the shared CPU FIFO
  - event-burst counters and product input trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI input tests逐文件性能静态审查（2026-07-18）

## 范围与覆盖

本批逐文件完整阅读`tests/runtime_input_manager.rs`、`tests/runtime_input_manager/**`与`tests/runtime_ui_support/**`共10/10个tracked Rust文件、1,504行、16个测试。累计UI tracked source从467/783增至477/783。

现有测试确定性覆盖capture优先级、popup top-only、preview tunnel短路、focus path、default action、double click timer、primary/secondary touch、多指针capture隔离、window batch与runtime fixture装载。扫描仅命中1处clone、2处collect与6处`rebuild_dirty`，没有sleep、ignored test、filesystem或worker；但没有125/500/1,000 Hz事件风暴、队列年龄、route visits、diagnostic bytes或rebuild次数预算。

## PERF-MVP-314：batch测试逐事件重建，缺少几何屏障与合并契约

`RuntimeUiManager::{dispatch_input_batch,dispatch_platform_input_batch,dispatch_window_input_pump_batch}`都通过`dispatch_manager_batch`循环调用单事件入口，而单事件入口在每次dispatch后执行`rebuild_dirty_surface`；ABI runtime batch又逐项调用window pump单事件入口。因此测试名虽为batch，实际不能证明同一帧高频move/analog/render-only dirty被合并，1,000个事件可观察为最多1,000次rebuild。

不能直接把整批改为末尾一次rebuild：resize/scale/fixture变化之后的pointer hit-test可能依赖新几何。EditorUI01与Runtime12应定义typed event barrier：geometry/window lifecycle先提交layout barrier；可合并的move/analog/hover采用frame内latest/delta；press/release/cancel、text/IME和popup边沿保持顺序；render-only dirty在下一个barrier或帧尾统一rebuild。测试helper必须调用同一产品batch authority，不维护独立语义。

验收要求：1/100/1,000事件批记录normalized events、route visits、full diagnostics bytes、layout/render/hit rebuild、coalesced/dropped与queue age；纯move/analog burst每帧rebuild有常数上限，resize后首个pointer使用新几何，press/release/cancel与multi-pointer capture顺序不丢失，错误仍报告原始batch index。Windows Cargo、F4产品input trace和idle/burst p95完成前保持pending。

## 责任计划

路由共享artifact、diagnostics opt-in与pointer lifecycle继续回链PERF-MVP-293/297；本批把batch barrier/coalescing验收补给EditorUI01和Runtime12，不另建平行输入authority。UE Slate application把平台事件、窗口状态和widget routing集中在单一application authority中，Zircon同样需要产品adapter与测试helper共用同一分段批处理，而不是在测试层逐事件隐式重建。
