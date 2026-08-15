---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: dynamic-runtime-animation-module-duplication
origin_plan: docs/plans/zircon_plugins/04-animation.md
fixing_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
origin_child_dir: docs/plans/zircon_plugins/04
fixing_child_dir: docs/plans/zircon_runtime/runtime/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/animation/module.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_plugins/animation/runtime/src/plugin.rs
tests:
  - linked dynamic session registers exactly one animation.runtime module and resolves the linked plugin manager
  - managed cargo +1.94.1 test -p zircon_runtime --lib dynamic_api --locked --jobs 1
---

# Runtime11: Dynamic runtime animation module duplication

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/04-animation.md`
- 来源执行切片：Plugins04 single production animation evaluator hard-cut
- 修复责任计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：动态会话构造拥有 linked plugin module 的组装与注册顺序；Plugins04 无权修改 `dynamic_api/session/construction.rs`。

## 失败现象与复现证据

`construction.rs` 先将 `LinkedRuntimePluginPlan` 解包为 `modules` 与 `linked_extensions`，随后无条件执行
`modules.modules.push(Arc::new(crate::animation::AnimationModule))`。动画插件的 runtime descriptor 同时通过
`with_module_descriptor(module_descriptor())` 提供同名 `animation.runtime` 模块。已链接动画插件因此能够同时进入
plugin module 与 Runtime fallback module 的注册路径，违背唯一 production evaluator owner。

## 最低共享层根因

Runtime11 的动态模块组装没有把 linked plugin 已声明的 canonical animation module 纳入 builtin module 的选择策略。
它把 Runtime fallback 作为无条件追加项，而非 capability routing 的明确分支。

## 架构修复验收

- 动态会话按 canonical module identity 选择唯一 `animation.runtime` descriptor；已链接动画插件时只注册插件 descriptor 与其
  `animation.evaluate` production system。
- 未链接动画插件时，Runtime 只能发布无生产求值的明确 capability/diagnostic 路由；不得重新激活旧 manager、scene hook 或
  第二套 graph/channel/bone evaluator。
- 增加覆盖 linked 和 unlinked 动态会话的聚焦回归，并通过原始 managed Runtime dynamic API gate；随后由 Plugins04 rerun
  animation package gate。

## 禁止临时方案

- 不得以模块名后缀、优先级排序、重复注册忽略或按调用方特判来隐藏两个 descriptor。
- 不得保留 linked plugin 与 Runtime fallback 的双 manager/evaluator，或通过 compatibility alias 将其中之一继续作为生产路径。
- 不得通过跳过 animation module 注册来掩盖缺失 capability；未链接场景必须显式可诊断。

## 修复结果与回传

Open state: `实现已收敛，声明门禁仍被无关 Runtime lib-test 编译错误阻断`; no pass is claimed.

### 2026-08-05 协调器验证证据

- 当前 `HEAD` `94cf43ec32367781308dd8d37f04382201109696` 已按 package identity 分流：linked
  `animation` 不再追加 Runtime fallback `AnimationModule`，unlinked 会话保留无 production evaluator 的 fallback；
  `linked_animation_plugin_registers_its_canonical_runtime_module_once` 与
  `builtin_dynamic_session_does_not_install_animation_evaluator_hook` 分别覆盖 linked/unlinked 路由。
- 初始不可变验证副本 `1106195a16954107917f5249f25abb63`（输入哈希
  `41426f552826efac4725c9b8c7f06db6a14c3ce5e087125c2ee81459e6cec1f5`，run
  `a32ef9f3bee943f4a9fe9d3a05cd6168`）在 `zircon_runtime_interface` 的 17 个模板
  `include_bytes!` 输入缺失处以 101 终止，目标测试未执行。
- 为避免 Cargo 闭包自动 overlay 旧 Performance 会话的数百个无关 attribution，建立了仅归属 17 个模板文件的
  最小验证会话；最终副本 `a16ecc2c755f42c88fad05a276ad4954` 成功物化，输入哈希
  `19b5411f3648ae0e53364719f55029491f11b34c6b5a270c724dfca196011020`。
- 最终 run `b511d5321d82480eba77516d9be61f56` 顺序请求 linked 聚焦回归、unlinked 聚焦回归、
  `zircon_runtime --lib dynamic_api` 与 `zircon_plugin_animation_runtime` package gate；它在构建
  `zircon_runtime` lib-test 时以 101 终止，报告 1066 个现有且跨域的编译错误，目标测试仍未执行。代表性阻断包括
  `RenderDeviceLimitDiagnostics` 字段漂移、`asset/project/paths.rs` 的 `u16` ASCII API、多个结构审计 source
  projection 漂移、图形测试借用/字段漂移与 ECS 测试生命周期错误。
- 因声明的 managed Cargo gate 没有执行通过，本 handoff 保持 `open`，不得生成 failure return、fixed receipt、
  Git commit 或企微完成通知。后续应先由这些下层 owner 收敛 Runtime lib-test 编译基线，再复用上述四门禁回传。
