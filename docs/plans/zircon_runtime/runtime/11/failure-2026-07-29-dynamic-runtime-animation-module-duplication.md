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

Open state: `待修复`; no pass is claimed.
