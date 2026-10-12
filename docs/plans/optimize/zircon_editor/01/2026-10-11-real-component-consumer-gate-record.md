---
related_code:
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/asset_imports.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-10-09-real-dynamic-component-consumer-gate-plan.md
  - docs/plans/astra/features/editor/2026-10-09-editor01-component-consumer-validation-list.md
doc_type: actual-source-adoption-record
title: Editor01 real component consumer coverage gate
date: 2026-10-11
implementation_status: actual_source_adopted
validation_status: native_pending
product_status: acceptance_pending
---

# Editor01 真实组件消费者覆盖门接入记录

日期：2026-10-11。本文记录现有组件资产强覆盖门的一次真实消费者接入。源码已实际采用并经独立 source review；没有运行 Cargo、native tests、性能矩阵或产品验收，因此完整门禁仍可能失败。

## 保留的强门合同

变更位于既有 [`asset_imports.rs`](../../../../../zircon_editor/src/tests/ui/boundary/zui_asset_governance/asset_imports.rs)，不新增 Rust 测试或验证 arm。原 9 个测试、静态 import walker、205 个 production Component、53 个静态资产 roots、3 个 Workbench design contracts，以及 `production_zui_component_assets_are_reachable_from_ui_root_widget_imports` 对完整资产遍历与 `offenders.is_empty()` 的要求均保留。此前审查库存为 258 个 production documents；205 个组件仍全部进入原 loop。

保留的三个 design-contract locators 分别对应 Level Editor window、command palette 与 preferences。强门没有通过新增静态根、retirement allowlist、资产类型改写或只测新 helper 来消除遗漏。

## 五条真实消费者 source witness

既有 strong-gate 测试持有 `crate::tests::support::env_lock().lock()` 返回的实际 `MutexGuard`，在该环境锁下调用五个生产 owner 的现有驱动：

| Owner | 实际驱动 |
|---|---|
| Workbench ten-core pane | `BuiltinWorkbenchWindowTemplateSurfaceBridge::exercise_core_module_sources_for_test()` |
| Particle pane | `BuiltinWorkbenchWindowTemplateSurfaceBridge::exercise_particle_library_sources_for_test()` |
| Tools window | `UiHostWindow::exercise_module_tools_sources_for_test()` |
| UiAsset preview | `UiHostWindow::exercise_builtin_ui_asset_preview_consumers_for_test()` |
| Generated bottom pane | `UiHostWindow::exercise_generated_bottom_sources_for_test()` |

每个驱动返回其真实 owner 完成 paint/input/retirement 检查后收集的 `UiV2SourceFileReceipt` 闭包。门禁使用没有 persistent store 的新 `UiV2PrototypeStoreFileCache` 重载消费者实际返回的文件，并要求当前 canonical physical path、asset ID、完整五字段 ordered-source equality 与原始 source bytes 一致；重复物理文件的身份和内容也必须一致。重载依赖只有在真实消费者也返回对应 receipt 时才获得 coverage，额外 `loaded` dependencies 本身没有覆盖信用。

这五个 witness 已接入原有可达性集合；完整 205 Component `offenders.is_empty()` loop 仍是唯一整体强门。本次没有运行该 gate，因此不登记任何 96 项 coverage credit，也不声称 offender 清单为空。

## 实际采用与审查证据

- [canonical apply receipt](../../../../../.codex/outbox/editor-real-component-coverage-gate-actual-v1/actual-canonical-apply.json)：SHA256 `5616986c95aaecb45a87b4504dc4ef231cc9af913681829bf220dcec3d7d366e`，状态为 canonical source adopted、native pending。
- [独立 review](../../../../../.codex/outbox/editor-real-component-coverage-gate-current-source-review-v1/final-v1/review.json)：SHA256 `8ef0bbba09af0f90b4ae113519e8ebd56c911fd0004b1ca603667956a7500f17`，0 blocking findings。
- [candidate patch](../../../../../.codex/outbox/editor-real-component-coverage-gate-proposal-v2/candidate.patch)：SHA256 `25a19014829e811deef200fd2cb7d9ff6354ccf9ff07725966d0daa645eeab54`。
- final binding SHA256 `2185e421564fba9cf84c02c8f78005c14b1d3445ef379ccddece620a72e081c5`。
- 采用后的 [`asset_imports.rs`](../../../../../zircon_editor/src/tests/ui/boundary/zui_asset_governance/asset_imports.rs)：raw SHA256 `f346860533e62f92f4692dbc43b1a7e4e2b0f67d8318093128b126e690b16100`，LF SHA256 `78f377c7272c64aa46c4a160630869deaefd1d9d62d3abc440f52d2aa1fa5523`。

采用记录包含 33 项 pre-adoption guard；目标文件之外的 32 项 read-only inputs 在采用核对中保持一致。canonical Rustfmt、scoped whitespace 和完整 patch reverse-check 均为 exit 0。恢复核对中首次 helper 的 post-check 因 optional `afterLfBytes` 未提供而退出；Root 随后只读确认目标文件为 26,178 LF bytes 且 LF hash 与完整 accepted postimage 相同，没有重新 apply 或重写 source。首次 control pre-snapshot 未持久化，因此 receipt 保留了这个历史观察限制；本文不声称已证明未保存的 pre-control 快照与当前 controls 的历史元数据相等。

## 尚未关闭的产品门

`additional`、`main`、`extension` 三个旧 aggregate 仍在 production inventory。其真实消费者迁移、旧导航与 paint/hit 祖先及 BlendSpace screenshot consumer 的精确接管，以及 aggregate 实体的产品兼容责任与退役决定仍 pending。因此，即使未来五个 witness 执行通过，完整 gate 仍需依靠全部 205 个组件的原断言判定。

后续 V11 仍保留 15 arms / 14 producers，只对实际采用的 source gate 绑定本次 hash；prepared manifest SHA256 `a3b8742b5959a445533517d8431e609c5eb354d5303467b5efddbadbe4006d1f` 记录 479 inputs，#2/#15 筛选下限分别为 22 / 191。按交接记录该 successor 是 prepared-not-launched，新增 Rust tests 与 arms 均为 0；授权提交后继续异步验证。此处没有原生执行、性能测量、Jenkins 提交、产品或 milestone acceptance 声明。
