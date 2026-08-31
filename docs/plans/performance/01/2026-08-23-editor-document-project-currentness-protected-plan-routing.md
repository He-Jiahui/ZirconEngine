---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-document-project-currentness-and-scene-move-m0.md
protected_ledgers:
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: plan-routing
status: proposed_protected_files_untouched
---

# Document / Project性能计划路由（2026-08-23）

本记录通知现有owner计划吸收性能合同；不直接修改受保护的`review.md`、`pending.md`或编号总计划。
Document与Project仅完成current static review和一个scene move M0，动态门未通过，不能进入已验收账本。

## 需要吸收的现有任务修正

| owner计划 | 必须吸收的合同 |
|---|---|
| Editor01 | project/scene只允许短main-affinity generation commit；锁内I/O、解析、catalog/plugin callback、等待和补偿为0，facts锁外发布。 |
| Editor03 | authoring scene以fallible exclusive transaction接收move-owned generation；stale/cancelled receipt不改变当前world/document。 |
| Editor09 | 只消费Runtime04 exact catalog delta；scene create失败不得调用project-wide reimport/rebuild。 |
| Editor10 | PERF-MVP-075/100/568/640统一为bounded recent ingress、promotable identity ticket、一次prepare、scene compact receipt和root capability；删除validate-open-validate与full result。 |
| Editor14 | 复用共享scheduler提供keyed single-flight、count/source/decoded/result bytes、age、deadline、cancel和main-affinity commit receipt；禁止project/scene私有线程。 |
| Frameworks01 | template/scene source及registry effect只有一个durable transaction owner，保留exact created ownership、rollback与restart recovery。 |
| Runtime04 | 发布immutable project/asset generation store和affected delta；迁移后删除`ProjectManager: Clone`及`current_project_snapshot`。 |
| Runtime11 | project create/open和scene prepare的filesystem/parse/write/fsync/cleanup在bounded job中执行，UI不wait；phase counter与payload resident bound纳入统一job合同。 |

## 受保护账本建议状态

- `zircon_editor/src/core/document`：保持pending。静态结构审查完成，但route gate、registry复杂度、
  full scene receipt、Cargo与产品trace未验收。
- `zircon_editor/src/core/project`：保持pending。Hub owner迁移与scene finish move是进展，但重复recent probe、
  同步create、deep snapshot、root capability、Cargo与产品trace未验收。
- M0量化只能记录为`PreparedSceneCreation::finish`完整document clone `1 -> 0`，不能记录为模块性能通过。

## 动态验收前置

只有current-source managed Windows editor可执行文件产生后，才运行WPR/xperf CPU/File I/O/waits/locks/
CSwitch/RSS/package power；完成scene generation首帧时再用RenderDoc核对draw/pixel/resource parity。
需要至少31个可比冷/热样本与p50/p95/p99/CI/effect size。没有这些证据，不提交性能里程碑commit，
也不发送企微“已完成”量化消息。

