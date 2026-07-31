---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: runtime-animation-fallback-evaluator-divergence
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/04-animation.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/animation/manager
  - zircon_runtime/src/animation/scene_hook
  - zircon_runtime/src/animation/sequence
  - zircon_runtime/src/animation/clip_event.rs
  - zircon_plugins/animation/runtime/src/evaluation
tests:
  - powershell -NoProfile -Command "Select-String -Path 'zircon_runtime/src/animation/**/*.rs' -Pattern 'node_records\(\)|keys\.windows\(2\)|while occurrence <= end_time|weighted_poses\.first\(\)\?\.clone'"
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_animation_runtime -VerboseOutput
---

# Plugins04：Runtime Animation后备求值器分叉交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：P5 Runtime Animation 28/28逐Rust文件性能审查与PERF-MVP-439..441
- 修复责任计划：`docs/plans/zircon_plugins/04-animation.md`
- 交接原因：Plugins04已声明M1 compiled/dense/PosePool hard-cut闭环，但`zircon_runtime/src/animation`仍保留可独立注册的旧manager与scene hook，最低共享原因是production/fallback求值owner尚未真正单一化。
- 生命周期键：`runtime-animation-fallback-evaluator-divergence`

## 失败现象与复现证据

Runtime后备hook每frame收集全部scene entity，clone四类player/parameter map，重复加载asset；paused clip/sequence仍采样或应用。graph/state-machine为event与pose重复evaluate，channel逐sample全keys扫描，track/bone和blend按字符串多层线性匹配，PoseApply再次全node扫描。loop event按跨度无上限逐occurrence生成String-bearing事件。

28/28文件静态读完。直接止损已删除Blend weight临时Vec和首pose深clone，但没有改变全scene调度、compiled artifact缺失、`O(B²)`混合、主线程串行或无界事件问题，不能据此关闭交接。

## 最低共享层根因

Plugins04的新`animation.evaluate`拥有revision cache、compiled graph/state/target、PosePool和dense slot；Runtime内置模块仍复制hard-cut前算法与scene hook注册。两条可达路径没有共享active/dirty projection、compiled artifact、scratch pool、任务调度和event budget契约，导致功能演进与性能门漂移。

## 架构修复验收

- 明确唯一production evaluator owner：Runtime built-in要么直接消费Plugins04共享compiled core，要么退化为薄注册/能力路由；禁止保留第二套字符串graph/channel/bone算法。
- typed/revision projection只访问animation player/skeleton，paused且time/revision不变时sample/apply=0；seek/resume在下一generation恰好求值一次。
- graph event与pose共享一次evaluation与asset handles；channel使用validated sorted artifact+fetch hint/二分，target/bone使用dense slot，steady PosePool求值零分配。
- pure instance evaluation进入Runtime11有界worker，owner thread只做确定性transition/event/transform apply；1/100/1k instances有main-thread budget/fairness counter。
- loop event使用count/time/bytes预算和可续游标，极端跨度单frame工作有硬上限且最终有序排空；overflow/age可诊断，不静默丢关键事件。
- current-source Plugins04/Runtime package gates、pose/event/state golden、paused/seek、规模counter与产品scene trace全部通过后按同一生命周期键回传。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止仅给旧scene hook套parallel iterator；未先收敛active projection、compiled artifact和有界apply时，线程化只会放大clone与锁竞争。
- 禁止通过关闭事件、丢弃paused pose、固定低骨数或缩短测试跨度掩盖工作量；语义必须由revision和budget显式表达。
- 禁止让Runtime与Plugins04各自维护不同的target ID、graph compile、PosePool或event cursor真相源。

## 修复结果与回传

- 已完成子项：Plugins04 compiled clip sampler 已在一次性 `validate_clip_channels` 建立有限、严格递增 key-time 不变量后，使用 `partition_point` 直接定位区间；删除 steady-state 的全 key 有限性复扫和 `windows(2)` 线性扫描。聚焦回归覆盖 Step 精确 key 边界与 Linear 邻接区间，未恢复 Runtime fallback、alias 或兼容分支。
- 当前证据：exact2 snapshot `1267`；`channel_sample.rs` SHA-256 `232de31097de394ffff7fe87c56111809cc21584eff8643b59ad0f1d795399ca`。静态边界、格式与独立复审已准备，managed Rust gate 必须在 Runtime08 scene binding 与 Runtime11 operation lower compile failure 返回后使用新鲜 current-source 输入运行。
- 仍未完成：Runtime 内置 manager/scene hook 的第二套 production evaluator 尚未收敛；active/dirty projection、共享 graph/pose 求值、Runtime11 有界 worker、owner-thread apply 与 loop-event 可续预算仍是本生命周期的关闭条件。
- Open state: `partial implementation; managed validation and remaining architecture convergence pending`; no fixed return or package pass is claimed.
