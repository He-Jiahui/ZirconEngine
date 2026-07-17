---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: export-host-high-frequency-input-dispatch
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/09-export-publishing.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/browser.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/mobile.rs
  - zircon_runtime/src/input/input_manager.rs
tests:
  - browser 125/500/1000 Hz pointer event coalescing benchmark
  - Android multi-pointer move dispatch-count test
  - exported host button-edge and raw-delta parity test
---

# Plugins09：export host 高频输入逐事件同步转发

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：export platform host files 逐文件静态审查
- 修复责任计划：`docs/plans/zircon_plugins/09-export-publishing.md`
- 交接原因：browser、Android 与 iOS 模板必须共同消费 Runtime12 输入合并契约，不能在单个平台局部 throttle。

## 失败现象与复现证据

生成的 WebGPU/WASM host 对每个 browser `pointermove` 事件立即调用
`zircon_export_handle_touch`；没有 requestAnimationFrame 级 latest-position 合并、raw-delta 累加或队列预算。
高轮询率鼠标可在一个渲染帧内产生多次 JS→WASM ABI 调用。

Android 生成 host 对每个 `MotionEvent.ACTION_MOVE` 遍历 `event.pointerCount` 并逐 pointer 同步 JNI dispatch；
一次多指 move 会放大为 N 次 ABI 调用。iOS 也对 `touchesMoved` 集合逐 touch 立即转发。button/touch begin/end
属于不可丢边沿，move/metrics 则适合帧内合并，但当前模板没有区分策略。

## 最低共享层根因

Export host templates 各自直接绑定平台事件到 ABI，没有复用 Runtime12 的统一输入采样/合并契约，也没有暴露
coalesced count、queue age 或 dropped/latest-value 指标。
Runtime12 共同负责定义 frame coalescing、raw delta 和边沿事件的跨平台输入语义。

## 架构修复验收

- browser/mobile host 对 pointer/touch move 使用帧级 latest-position + raw-delta 累加；begin/end/cancel、按键边沿严格保序不丢。
- viewport metrics/resize 在一帧内合并，生命周期事件不合并越过状态边界。
- 125/500/1000 Hz pointer fixture 下，单 pointer 每 frame ABI move dispatch 有明确上限，多指按 active pointer 数线性。
- JS/WASM、JNI 与 Swift host 输出和 desktop Runtime12 输入 snapshot 在按钮状态、坐标、delta、touch id 上 parity。
- 记录 input events received/coalesced/dispatched、queue age 与 main-thread/ABI wall time。

## 禁止临时方案

- 不得粗暴 throttle 所有输入而丢失 press/release/touch begin/end/cancel。
- 不得只在 runtime manager 末端丢事件；跨语言 ABI 调用应在 host 边界先避免。
- 不得让三个平台各自发明不同的 move 合并语义；Runtime12 定义公共契约，Plugins09 负责模板落地。

## 修复结果与回传

Open state: `待 Plugins09/Runtime12 实现 export-host frame input coalescing 与跨平台 parity 压测`。
