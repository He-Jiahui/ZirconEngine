---
related_code:
  - zircon_app/src/entry
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
reference_sources:
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
tests:
  - zircon_app/src/entry/tests
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/entry_runner/editor/tests
  - current-source Windows Cargo and product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Zircon App entry逐文件性能静态审查（2026-07-19）

## 范围与覆盖

`zircon_app/src/entry/**`当前源 **144/144** 个Rust文件、**13,231** 行、**185** 条测试已逐文件阅读；其中生产文件97个、测试文件47个，Git tracked 138个，另有其他会话引入的6个未跟踪文件也纳入当前源审查。审查覆盖Runtime/Editor/Headless入口、winit事件循环、动态runtime ABI、surface present、输入/IME/gamepad、native plugin bootstrap和editor runner启动链。

测试主要证明目录结构、ABI语义、调用顺序与profile/plugin选择。当前没有30秒空闲CPU/唤醒率、1k/10k事件风暴、host-request队列上限、单次启动project-open/registration build count或稳定帧present成本断言，因此本模块只完成静态覆盖，不能进入`review.md`。

## 关键瓶颈

- **PERF-MVP-424 / Runtime03**：reactive cadence已替代DesktopApp无条件tick，但所有未处理window event和所有device event仍可请求runtime frame；Game/Continuous/Mobile使用无上限`Poll`，Headless固定60Hz，frame pump还会重复应用control flow。相同尺寸resize也会重做runtime resize、surface rebind与fallback resize。
- **PERF-MVP-425 / Runtime10**：非空host request仍经历manager解析、全队列drain、`Vec`收集、JSON编码、ABI owned buffer、JSON解码和主线程全量应用；无count/time/backpressure/coalesce预算。reactive wake的host直达路径仍经全局`Mutex<HashMap>`查找代理。
- **PERF-MVP-426 / Runtime12**：pointer/device输入逐事件同步跨ABI；gamepad每帧无预算drain，空闲reactive模式又缺少gilrs唤醒；rumble以无界`BTreeMap<u64, Vec<_>>`积累。keyboard fallback为普通按键执行`format!("{code:?}")`。
- **PERF-MVP-427 / Editor01联动Editor12**：editor GUI/CLI先构造一次first-party registrations用于capability/session，bootstrap时再次构造；同一project先由`editor_entry_config`打开，retained host startup又打开一次。builtin bootstrap还重复store config、重建catalog和复制descriptor/report。

## 本轮直接止损

按RED→GREEN源码守卫，`file_drag_drop/{dropped,hovered}.rs`改为直接借用`Path::to_string_lossy()`的`Cow<str>`，常见有效UTF-8路径每次hover/drop少一次`String`分配。`entry_runner/bootstrap.rs`直接移动native load report拥有的两组registration vectors，删除两次深clone；对应源码守卫、`rustfmt`和`git diff --check`已通过。行为ABI不变，current-source Cargo仍待受管队列验证。

## 参考引擎结论

Bevy winit runner把focused/unfocused策略分开，并以`react_to_device_events`显式区分reactive与low-power reactive；这支持Zircon按事件类别决定是否tick，而不是“任意winit事件均请求帧”。Unreal主循环在失焦idle时跳过输入/渲染并sleep，Slate只在active timer、输入或合成cursor需要时tick/draw，模态循环还硬限制为60Hz。Zircon应采用同类状态/预算合同，不照搬其固定阈值。

## 动态验收

Runtime/Desktop/Editor/Headless分别记录30秒focused/unfocused idle的tick、redraw、wake、CPU与主线程时间；1k/10k mixed window/device/gamepad/host-request storm记录队列peak/age/drop/coalesce、单帧drain和p95；editor启动断言project open、manifest parse、first-party registration/catalog build每generation至多一次。F0/F2需补native present与fallback scope、稳定帧像素/RenderDoc对拍。2026-07-19受管`zircon_app`验证再次在Cargo启动前失败：`validate-matrix.ps1:187`的`ConvertFrom-Json`遇到非JSON首字符`s`；本机`renderdoccmd`也不可用。这些是动态验收缺口，不把静态结果冒充通过。

## 责任计划交接

- Runtime03：`runtime/03/failure-2026-07-19-app-entry-cadence-and-event-trigger-budget.md`
- Runtime10：`runtime/10/failure-2026-07-19-app-entry-host-request-and-wake-boundary.md`
- Runtime12：`runtime/12/failure-2026-07-19-app-entry-input-and-gamepad-storm-budget.md`
- Editor01：`editor/01/failure-2026-07-19-editor-startup-single-projection.md`

## 2026-07-23 current-source增量复核

入口树当前物理源码已增长为 **146** 个Rust文件、**13,796** 行、**199** 条`#[test]`；本轮完成全部 **146/146** 个current blob逐文件复读，组合指纹为`336f4f54117a11d240c56f4096aae893c7436f26f85fc39e58aa0de0daaaa0fd`，详见五份2026-07-23增量证据。33个modified和8个untracked外部文件均只读纳入，未吸收其实现。该数字只完成当前静态对账，不改变动态pending状态。

增量复核再次确认PERF-MVP-023/424/425/427并新增574；PERF-MVP-426旧风险已按current source纠正为“bounded drain/rumble与nonalloc key静态存在，gilrs wake/逐事件ABI/queue指标仍缺”。根文件还确认descriptor single-generation止损存在，但dynamic module wrapper每构造`Box::leak`两段文本，已补强004。current-source Cargo、leak/startup/owned-output/input/event storm、F0/F2产品/RenderDoc未完成，本记录状态保持`static_complete_dynamic_pending`。
