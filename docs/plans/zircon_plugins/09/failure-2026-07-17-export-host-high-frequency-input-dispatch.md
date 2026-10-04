---
handoff_kind: failure
status: open
created_at: 2026-07-17
updated_at: 2026-09-21
summary_slug: export-host-high-frequency-input-dispatch
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/09-export-publishing.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/09
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/browser.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/mobile.rs
  - zircon_runtime/src/core/framework/input/input_manager.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/event_buffer/frame.rs
  - tools/tests/test_plugins_09_export_host_input_coalescing.py
tests:
  - browser 125/500/1000 Hz pointer event coalescing benchmark
  - Android multi-pointer move dispatch-count test
  - exported host button-edge and raw-delta parity test
  - generated host frame scheduler source contract
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

Current owner note（2026-07-18）：中立 `InputManager` trait 与 frame snapshot/queue status surface 位于
`core/framework/input/input_manager.rs`；`DefaultInputManager` 的生产 `submit_event(...)` 实现在
`input/runtime/default_input_manager.rs`，其 `FrameEventBuffer` 已在 `input/runtime/event_buffer/frame.rs`
对相邻 `CursorMoved` 做 latest-position 合并、对相邻 `MouseMotion` 做 raw-delta 累加。该 Runtime 内部合并
不能消除 browser/JNI/Swift host 在 ABI 前的逐事件跨语言调用，因此本 failure 仍保持 `open`，修复责任仍归
Plugins09/Runtime12，不恢复已删除的 flat input-manager owner。

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

Current owner note（2026-07-22）：本轮重新逐文件核对40个`export_build_plan`生产文件，browser仍为每个`pointermove`直接JS→WASM，`resize`直接读取layout并跨ABI；Android/iOS仍逐active pointer/touch同步调用。由于公共edge/move/raw-delta/metrics语义尚未由Runtime12落地，本轮没有做单平台局部throttle，PERF-MVP-052与本failure保持open。

Open state: Plugins09 host-side frame coalescing is implemented on the current source, but
source-bound managed Cargo, real Android/iOS execution, live runtime ABI parity and p95 evidence
remain pending; this record must not return as `fixed-*` yet.

### 2026-09-21 current-source host-frame implementation

- 根因复核：browser 仍由 `pointermove`/`resize` 直接跨 ABI；Android `ACTION_MOVE` 按
  `pointerCount` 逐项同步派发；iOS `touchesMoved` 逐 touch 同步派发。Runtime12 的
  `FrameEventBuffer` 已在 ABI 之后合并 `CursorMoved`/`MouseMotion`，不能消除上述跨语言调用。
- 实现：browser 以 pointer-id `Map` + 单次 `requestAnimationFrame` 发布 latest position，累积
  `movementX/Y`，并在 pointer/key/lifecycle 边沿前排空且取消待执行帧；resize 同帧只发布一次。
  Android 以 `postOnAnimation` 和稳定插入序 map 每 active pointer 每帧发布一次；down/up/cancel/key
  严格先排空 move。iOS 使用 weak-target `CADisplayLink`，避免 display-link retain cycle，并以
  pointer identity 合并 move/viewport。三端均发布 received/coalesced/dispatched、raw delta、最大
  queue age 与 ABI wall-time 计数。
- TDD：新增守卫首次在旧源码得到 `4/4` 失败。修复后扩展为 `5/5` GREEN；其中 Node 直接执行从
  Rust 模板拼出的 JS，在 60 Hz 模拟帧下分别注入 125/500/1000 Hz 单 pointer 事件，三组均只产生
  60 次 move ABI 派发，并验证 move→pointer-up、move→key、raw-delta 累加及 100 次 resize→1 次
  viewport 派发。命令：
  `python -X utf8 tools/tests/test_plugins_09_export_host_input_coalescing.py`。
- 本地结构证据：两份 Rust 模板 `rustfmt --edition 2024 --check` 通过；精确
  `git diff --check` 通过（仅 checkout 的 LF→CRLF 提示）。这些不是受管 Cargo 或真实设备证据。
- 下层阻塞：生成 runtime library 的 `zircon_export_handle_touch` 当前仍属于同计划 open failure
  `woc-mobile-browser-host-noop` 的 live-session/runtime-input 接线范围。只有该下层 owner 完成后，才能
  在 browser/Android/iOS 真宿主上验收按钮状态、坐标、raw delta、退出与两帧非空 present parity。
  此外仍需 source-bound managed Cargo 运行现有 platform export tests，以及原文要求的设备 p95/ABI
  wall-time；在这些证据齐全前保持 `open`。

### 2026-09-21 independent-review repair

- 冻结 snapshot `3725` 的首轮独立审查为 `Critical=0 / Important=3 / Moderate=4`，因此该快照及其
  票据不得复用为修复证据。审查确认 browser 在 canvas 外可能丢 release，iOS display-link selector
  与生命周期边界不正确，移动端 identity/order、Android 已投递 callback、main-thread wall 指标及
  行为覆盖也不完整。
- Browser 现在在 down 时显式 pointer capture，由 window 接收 up/cancel，并在 lost capture 与
  pagehide 上生成有序 cancel；pagehide 先排空 move，再 cancel 所有 active pointer，最后发布
  suspended。Node 回归真实执行生成 JS，并覆盖 canvas 外 release 与 cancel-before-suspend。
- Android 改为唯一稳定 `Runnable`，同步 edge/key/lifecycle drain 会通过 `removeCallbacks` 撤销已投递
  callback；新 move 才能再投递下一次 view-frame callback。iOS 使用正确的
  `tick(_ displayLink: CADisplayLink)` selector，在 resign-active 时排空并按稳定单调 touch id 取消，
  did-become-active 后重新发布 lifecycle/viewport；pending move 由显式顺序表排放，不再使用
  `touch.hash`、无序 `Set`/`Dictionary.keys.first` 作为 ABI identity/order。
- 三端均新增 `inputMainThreadWallTime`，与 `inputAbiWallTime` 分离。当前本地合同为 `6/6` GREEN，
  `rustfmt --edition 2024 --check` 与精确 `git diff --check` 通过；移动端双指/频率参考模型验证每帧每
  active pointer 最多一次派发。该模型和源码合同不是 Kotlin/Swift 编译或真机证据，后两项仍明确
  留在验收门禁中。修正后独立复审、受管 current-source 票据、Cargo、live ABI、真机与 p95 仍待完成。

### 2026-09-21 second independent-review repair

- Snapshot `3727` 的独立复审结果为 `Critical=0 / Important=3 / Moderate=0`，另有一项不在冻结
  snapshot 内的直接 canonical-test 阻断。绑定该 snapshot 的 partial ticket
  `2b17d7de41ea4c5eb825ba24ff7a46eb` 虽以 `6/6` 和 rustfmt 通过，但因审查未清零且后续源码已变化，
  只能保留为旧快照证据，不能复用为当前修复验收。
- Browser 生成的 canvas 现直接声明 `touch-action: none`，避免 UA 的 pan/zoom direct manipulation
  在 pointer capture 之外取消触控流。Android 新增 `onPause` 对偶边界：同步撤销已投递 frame
  callback、排空 pending move/viewport、按 pointer id 稳定排序取消 active touches，最后发布
  `ZIRCON_LIFECYCLE_SUSPENDED`；`onResume` 才重新发布 resumed 并采集 fresh viewport。iOS 两个
  initializer 都通过统一配置设置 `isMultipleTouchEnabled = true`，实际允许同一 touch sequence 的
  多 active touch 进入既有单调 id/order 合同。
- TDD 先只增加三项合同，旧模板得到恰好 `3` 个失败；生产修复后命令
  `python -B -X utf8 tools/tests/test_plugins_09_export_host_input_coalescing.py -v` 为 `7/7` GREEN。
  两份模板 `rustfmt --edition 2024 --check` 与精确 `git diff --check` 通过。当前机器仍没有
  Kotlin/Gradle、Swift/xcodebuild 工具，因此这些是 browser 动态 + mobile 源码合同证据，不是移动端
  编译或真机验收。
- 直接消费者 `zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform.rs` 仍要求已删除的
  `UInt64(touch.hash)`。其当前 SHA-256 为
  `8b764ee46921faa8009831062cbfa3a97e15172b5ffe590c91dec2e66c8bec7d`，与协调器中 archived
  `astra-test-owner-repair-20260905` attribution
  `6f71dfc45b66695797cfe6e625a00f13fe53b5c211d63567943c921c33878217` 不同；文件含未归属的 foreign
  diff 且无 live lease，本 lifecycle 不吸收该文件。其 owner 需要在保留现有改动的前提下，把 stale
  assertion 改为单调 touch-id/multitouch/lifecycle 新合同并回传当前哈希；在此之前 managed platform
  export Cargo gate 是确定阻断，而不是尚未执行即可假定通过。

### 2026-09-21 third independent-review repair

- Snapshot `3729` 的独立复审把上一轮三项 Important 清零，结果为
  `Critical=0 / Important=0 / Moderate=1`。剩余 Moderate 是 Android 冷启动 `onResume` 在首次 layout
  前读取 `decorView.width/height`，可能先发布 `0x0` 或旧 viewport。绑定该源码的 partial ticket
  `60630f6be7ec4512ad3b1a12d4715398` 因后续修复而成为旧快照票据，不作为当前证据复用。
- Android 恢复路径现在只在 `frameView.isLaidOut` 时尝试采样，且公共 queue 入口拒绝非正 width/height；
  冷启动由既有 layout listener 首次发布有效尺寸，已 layout 的 pause/resume 则立即排队当前尺寸。合同先
  要求 layout/positive-size guard，在旧源码得到恰好 `1` 个失败，修复后完整 Python/Node 批次重新
  `7/7` GREEN；rustfmt 与精确 diff-check 继续通过。

### 2026-09-21 fourth independent-review repair

- Snapshot `3731` 的受管 partial ticket `84a4d08b52714d2d816788a44cbe8bff` 真实执行
  Python/Node `7/7` 与 rustfmt 并通过，但随后独立复审仍为
  `Critical=0 / Important=0 / Moderate=1`，所以该票据只保留为旧源码证据，不能复用为当前验收。
- 剩余 Moderate 是 Android `View.isLaidOut` 只表示 attach 后至少经历过一次 layout；Activity 暂停
  期间若发生窗口、inset 或配置尺寸变化，恢复时旧的正尺寸仍可能满足该条件，而新 hierarchy layout
  已由 `isLayoutRequested` 标记为待处理。旧实现会在 layout listener 发布新值前，把旧 viewport 排入
  frame callback；正尺寸 guard 无法识别这种 stale snapshot。
- TDD 先把恢复合同收紧为 `!frameView.isLaidOut || frameView.isLayoutRequested` 时不得立即采样，旧模板
  得到精确 `1` 个失败。生产入口采用同一 guard 后完整 Python/Node 批次重新 `7/7` GREEN；待布局时
  由既有 layout listener 发布 fresh viewport，只有已布局且无待处理 layout 时才立即采样。两份模板
  rustfmt 与精确 diff-check 同时通过。该证据仍是 browser 动态 + mobile 源码合同，不替代 Kotlin、
  Swift、真机、live ABI 或 p95 验收。
- Current-source partial ticket `acf2a1ea0f044e4888aa3bdbf75a202a`（job
  `6607213fcd1b4ed6b1a111d8bb76a1d9`）已在 source manifest
  `0de7c121bae9e0cd61e7a2c5bf09c543241f558719c18f428f6fbd3c549169d1` 上真实执行并以 exit `0`
  通过：Python/Node `7/7`、rustfmt 及 marker
  `PLUGINS09_EXPORT_HOST_INPUT_PENDING_LAYOUT_CURRENT_SOURCE_PASS` 均匹配。Snapshot `3733` 的第五轮
  独立复审同时为 `Critical=0 / Important=0 / Moderate=0`。直接 canonical Rust test 的 foreign
  stale assertion、managed Cargo、Kotlin/Swift、live ABI、真实设备与 p95 仍未完成，因此本 failure
  保持 `open` 且不得 return/closeout。

### 2026-09-25 current-source rolling reconciliation (Plugins09 owner)

- Session `failure-roll-01a084c8-plugins09-export-host-input-r3` owns this refresh. Snapshot `3813` seals the six related runtime/template paths plus the canonical contract script:
  - `zircon_runtime/src/plugin/export_build_plan/platform_host_files/browser.rs` — `1e827517b73bf2cb5c5b5b5e24f0ff20e53a0e49f8014e7e6acd9912960d928e`
  - `zircon_runtime/src/plugin/export_build_plan/platform_host_files/mobile.rs` — `0bab09fa5d9fdc34bf6b1a6a5cf009062d658148556267a36d06e151efa7c986`
  - `zircon_runtime/src/core/framework/input/input_manager.rs` — `de9a04ff9d7d7fbd7da4436baacfde8d928df9f89510919e1f00a95ae53ee5db`
  - `zircon_runtime/src/input/runtime/default_input_manager.rs` — `35ee1027f0ec92304963cfb8ed94c97db7ca834a9ba360357f7f50ddeea6ed3e`
  - `zircon_runtime/src/input/runtime/event_buffer/frame.rs` — `760bd44254c30aeb6e4bad787452b1b9215c5889a00b8095f7d58092d6cb4cd6`
  - `tools/tests/test_plugins_09_export_host_input_coalescing.py` — `88651aab2a2b59155ea8c21a1f4290cba16db9e91823d1503c9a2cf340ba557e`
- The browser template, mobile template, and `default_input_manager.rs` are already dirty in the shared checkout; the canonical contract script is untracked. These are preserved as current/foreign provenance and were not rewritten. `git diff --check` reports no whitespace errors (only normal LF→CRLF notices). `rustfmt +1.94.1 --edition 2024 --config skip_children=true --check` passes for both host templates plus `default_input_manager.rs` and `event_buffer/frame.rs`; the complete five-Rust-path check remains non-passing only because the pre-existing `input_manager.rs` trait signature indentation differs from rustfmt output.
- The exact current contract was executed: `python -B -X utf8 tools/tests/test_plugins_09_export_host_input_coalescing.py -v` ran all seven tests (`7/7`, including generated browser 125/500/1000 Hz dispatch, Android pause/layout guards, iOS display-link lifecycle, shared telemetry, and mobile multi-pointer reference scheduling). Source inspection confirms pointer capture/lost-capture/pagehide cancellation, one view-frame callback for Android, paused display-link scheduling for iOS, and separate ABI/main-thread wall counters. This is browser dynamic plus source-contract evidence, not Kotlin/Swift compilation or device/live-ABI evidence.
- Managed Windows Cargo, the stale canonical Rust consumer assertion owner, WOC live ABI/runtime-input handoff, Kotlin/Gradle and Swift/xcodebuild, real-device p95, and closeout remain pending; the failure stays `open`/`resolving_failure`.

### 2026-09-25 independent static review receipt

- Reviewer `review_editor03_gizmo_private` checked snapshot `3813`, all six Rust/template paths plus the canonical script hash, and the documented dirty/untracked provenance. Result: Critical/Important/Moderate = `0/0/0`.
- The review confirms the seven-test current contract and the browser pointer-capture/pagehide, Android callback/pause/layout, iOS display-link/lifecycle, and split ABI/main-thread telemetry semantics. It does not infer Cargo, mobile toolchain, live ABI, device or p95 acceptance.
