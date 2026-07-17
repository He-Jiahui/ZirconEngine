---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: module-descriptor-regeneration
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/02
plan_link_mode: child_record_only
related_code:
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/plugins/builder.rs
tests:
  - descriptor generation count during bootstrap-with-report
  - bootstrap report and activated module descriptor equivalence
---

# Runtime02：bootstrap report 重复生成 module descriptors

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F0 启动路径逐文件静态审查
- 修复责任计划：`docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md`
- 交接原因：descriptor snapshot 的所有权属于 Runtime02 模块启动契约，性能 Session 不应在 app/bin 层建立旁路缓存。

## 失败现象与复现证据

`bootstrap_with_report` 为报告生成一批 module descriptors，随后 `entry.bootstrap()` 在注册阶段再次生成；`PluginGroupBuilder::try_finish` 的排序路径也读取 descriptors。一次 MVP bootstrap 对同一模块至少存在多次 descriptor 构造/复制，descriptor 构造若包含选择、字符串或动态服务查询，会放大冷启动成本。

## 最低共享层根因

根因是启动报告和实际注册没有共享一次冻结后的 descriptor snapshot，责任属于 Runtime02 的入口/模块所有权，不应在 editor 或某个 bin 中缓存旁路。

## 架构修复验收

- 先加每模块 descriptor 生成次数测试，再让报告、排序与注册消费同一冻结结果。
- 报告中的顺序、依赖和 capability 必须与最终激活模块完全一致。
- 当前源码 cold/warm bootstrap trace 对比 descriptor 次数和耗时；没有数据不得声称启动收益。

## 禁止临时方案

- 不得在 editor/bin 单独缓存 descriptors。
- 不得删除 bootstrap report 或降低一致性检查来换取时间。

## 修复结果与回传

Open state: `待 Runtime02 以单次 descriptor snapshot 修复并回传聚焦测试与启动 trace`。
