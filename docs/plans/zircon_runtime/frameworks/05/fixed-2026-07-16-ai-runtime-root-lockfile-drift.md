---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: ai-runtime-root-lockfile-drift
origin_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
fixing_plan: docs/plans/zircon_plugins/06-ai.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/05
fixing_child_dir: docs/plans/zircon_plugins/06
plan_link_mode: child_record_only
related_code:
  - zircon_plugins/ai/runtime/Cargo.toml
  - Cargo.lock
tests:
  - cargo test -p zircon_runtime --lib --locked ui::tests::text_pipeline::measure_cache
  - managed current-source root job e0c50e57b3774641897b9a839be00f71
resolved_at: 2026-07-16
---


# Plugins06: AI runtime 根锁文件与清单漂移阻断 Text locked gate

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 来源执行者：`frameworks05-m3-text-hardcut-20260715`
- 来源执行切片：M3 shared Text service 当前源码 default/UI focused gate
- 修复责任计划：`docs/plans/zircon_plugins/06-ai.md`
- 交接原因：Text focused gate 未进入 `zircon_runtime` 编译，失败发生在 AI runtime 清单和根锁文件之间；Frameworks05 不拥有 AI package 或共享 `Cargo.lock` 更新。

## 失败现象与复现证据

2026-07-15，受管 job `b6ba6fe766a248e2976e808b11f6edd2` 首先因
`zircon_plugins/ai/runtime/Cargo.toml` 引用未声明的 workspace `serde_json` 而在工作区
清单解析阶段退出 101。AI owner 随后将依赖收敛为显式 `serde_json = "1.0.149"`，但根
`Cargo.lock` 中 `zircon_plugin_ai_runtime` 的依赖列表仍没有 `serde_json`。

新的受管 locked job `c92cc389bf1848a19945626f162adbf4` 因此以
`cannot update the lock file ... because --locked was passed` 退出 101；两次任务都没有编译
或执行 Text 测试。

## 最低共享层根因

AI runtime 新增了根工作区可见的直接依赖，但只更新了 package 清单，没有同步根工作区
锁文件。Cargo 在解析根 workspace 时必须统一解析所有成员，因此即使 focused 命令只选择
`zircon_runtime`，`--locked` 也会在编译前拒绝这组不一致输入。

## 架构修复验收

- AI owner 以自己的受管流程同步根 `Cargo.lock`，使 `zircon_plugin_ai_runtime` 清单与锁文件一致。
- 根工作区 `cargo test -p zircon_runtime --lib --locked ui::tests::text_pipeline::measure_cache` 必须能进入当前源码编译；后续 Text 测试结果仍归 Frameworks05 处理。

## 禁止临时方案

- 不得由 Frameworks05 手工编辑或生成 AI owner 的锁文件差异。
- 不得移除 `--locked`、删除 AI 依赖或排除 workspace member 来绕过门禁。
- 不得把清单解析失败记作 Text 测试通过或失败。

## 修复结果与回传

- 根因：AI runtime 曾短暂增加 serde_json 直接依赖，却未同步根 Cargo.lock，令任意根工作区 --locked gate 在编译前拒绝清单。
- 架构修复：AI 集成节点改用 zircon_runtime 已有动态组件/属性契约，不再增加第三方依赖；清单仅扩展既有 runtime feature，使根锁文件依赖行保持稳定。
- 验证：受管 job 579134d0d21c413296f213302875a5d0 完成 AI runtime 64/64 tests 与 doctests；来源 owner 的当前共享源码受管根 job e0c50e57b3774641897b9a839be00f71 退出 0，证明原 --locked 向上门禁恢复。
- 回传：已消除 AI 清单与根锁文件漂移，Frameworks05 的 Text locked gate 可在当前共享源码进入编译并通过。
