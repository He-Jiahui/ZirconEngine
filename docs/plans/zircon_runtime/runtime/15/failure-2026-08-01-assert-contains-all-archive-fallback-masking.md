---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: assert-contains-all-archive-fallback-masking
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/support.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention
tests:
  - assert_contains_all rejects anchors absent from the supplied source
  - Runtime15 historical guards name and read one explicit canonical owner
---

# Runtime15 assert_contains_all archive fallback masking

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime15 structure-convention assertion helper audit
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：结构守卫 assertion helper 与其调用点属于 Runtime15 测试基础设施，Performance01 只记录其可导致 false green 的审阅结论。

## 失败现象与复现证据

`support.rs::assert_contains_all` 根据自由文本 `label` 选择额外 archive/current-owner inventory，并在调用方传入的 `source` 缺少锚点时从额外内容补齐。八文件迁移后的只读扫描显示 Runtime15 parent、runtime index、engine review 与 engine structure 的 live-root 读取仍分别散布在 504、484、621、616 个结构守卫文件中，因此错误 source 可以被 helper 静默掩盖。`runtime_15_output_archive_source` 实际还会拼接 active `runtime/15/*.md`，包括 open failure，而不是只读取 `_archive`；新增 failure 本身即可改变 fallback 结果。

## 最低共享层根因

assertion helper 同时承担“检查已传入 source”和“猜测另一个证据 owner”两种职责，source identity 没有进入类型或调用合同；`runtime_15_output_archive_source` 的名称还与其实际读取的 active child directory 不一致。

## 架构修复验收

- `assert_contains_all` 只检查调用方显式传入的 source，或者由 typed owner API 返回带固定身份的 source；不得按 label 猜 owner。
- 分批把历史守卫硬切到明确 canonical archive，把 current-state 守卫保留在 live/index/current-owner 文件，并为每批保留 failure/fixed 与真实执行证据。
- 新增 helper regression：传入错误 source 时必须失败，即使其他 archive 含有同名锚点。

## 禁止临时方案

不得一次性删除 fallback 后以大面积 ignored/allow-failure 收场，不得把 archive 内容复制回 live 文档，也不得把 label 匹配扩成更多隐式 owner。

## 修复结果与回传

Open state: `五组高置信调用点已开始迁移；其余调用点需要按 historical/current owner 分批归因后删除 label-based fallback，当前不得宣称 helper false-green 风险已关闭`。
