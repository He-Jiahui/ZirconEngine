---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-core-export-currentness-and-product-chain-revalidation.md
protected_ledgers:
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_plugins/09-export-publishing.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: plan-routing
status: proposed_protected_files_untouched
---

# Editor core Export性能计划路由（2026-08-23）

本记录只通知owner计划吸收合同，不修改受保护`review.md`、`pending.md`和编号计划。
`core/export/**`已完成9/9 current static reconcile，动态验收未通过。

## Owner合同

| owner | 必须吸收 |
|---|---|
| Editor15 | 唯一headless export graph/run generation供UI/commandlet/CI共享；nested plan、`.core.json`、第二inventory和同stage重复prepare/execute归零。 |
| Editor14 / Runtime11 | enumerate/hash/copy/process/log/persist统一bounded job authority；bytes/age/deadline/cancel/terminal receipt显式，Drop I/O和private threads为0。 |
| Runtime04 | Cook/Pack只发布content-addressed manifest和generation；不允许Editor从source/output目录重新推断artifact membership。 |
| Plugins09 | native plugin staging返回一次package manifest，Stage消费receipt；不按stage重复扫描/复制package tree。 |
| EditorUI08 | UI只投影run/stage/output generation；stable filesystem query、完整log materialization和wide row rebuild均为0。 |

## 账本建议

- `zircon_editor/src/core/export`保持pending；9/9静态完成和10/10 source contract不等于F4产品验收。
- 保留并发实现的16 KiB line、512 tail、16 buffered-event、process-tree cleanup；这些不能抵消当前
  graph constructor 3、`.core.json` 2、core executor 2、fingerprint cancel check 0、Drop persist 1。
- 不登记`VecDeque`等局部容器微调为里程碑；先完成唯一graph/manifest/receipt hard cut。

## 动态前置

在approved D/E/F target产生current-source可执行文件后，执行至少31个cold/warm/1% changed F4样本，采集
WPR CPU、wait/lock、CSwitch、File/process I/O、allocator/RSS和package power，并报告p50/p95/p99、CI、
effect size与环境。随后启动exported client/server做产品与RenderDoc parity。通过前不提交性能里程碑，
不发送企微“完成”量化消息。
