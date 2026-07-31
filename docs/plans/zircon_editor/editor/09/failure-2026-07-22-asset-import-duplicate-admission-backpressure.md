---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: asset-import-duplicate-admission-backpressure
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/09
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/asset/import_flow
  - zircon_editor/src/core/jobs
tests:
  - same URI and same UUID duplicate storm single-flight
  - queue entry byte age backpressure
  - cancellation panic shutdown and path migration
---

# Editor09：资产导入重复准入与背压缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-555 import-flow/job-admission 性能审查
- 修复责任计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 交接原因：generation identity、single-flight ticket 与 retained-flight budget 属于 Editor09 import admission；Performance01 只负责审计放大，Editor14/Runtime04/Runtime11 保持消费者和既有 owner。
- 回链：`PERF-MVP-555`，并向 Editor14/Runtime04/Runtime11 消费者保留验证依赖；不转移 import owner。

## 失败现象与复现证据

ImportFlow按URI分配mutex group只把重复任务串行化，并不合并同URI或同UUID/source generation；watch、digest mismatch与manual请求风暴可以把等价任务无限提交到EditorJobSystem。submit为保持path migration/importing原子性同时持有state与index mutex，RAII结束再依次获取两锁，重复请求还各自生成label/progress URI String。

源码合同以 10,000 次同 generation 请求稳定复现旧准入模型的 job 放大，并分别锁定 entry、估算驻留 byte
与 oldest-active-age 缺少硬上限的失败面。百万级产品 trace 仍是最终性能验收，不以较小单元合同替代。

## 最低共享层根因

Editor09 的准入层缺少 generation identity、共享完成对象和 retained-flight budget；Plan14 只能看到已经
放大的独立 job，Runtime04 `AssetManager` 也无法在编辑器队列之前合并 observer。锁问题来自 ImportFlow
同时把索引权威解析和本地准入状态塞进一个临界区，而不是 Runtime 资产管线本身。

## 架构修复验收

Editor09/14在job准入前按`(uuid, source/import generation)`建立single-flight共享ticket，合并reason/observer并限制entries+bytes+oldest age；Runtime04 AssetManager仍是唯一实际import owner。reservation token应缩短双锁区且保持同UUID跨URI迁移正确。要求1M重复请求实际import≤1/generation、队列内存硬有界、backend/job submit不在双锁区、取消/panic/shutdown不泄漏importing状态。禁止只扩大队列或另建UI import scheduler。

具体 acceptance：

- generation 必须包含 UUID、规范化 URI 和 Runtime source digest；路径迁移产生新 generation，但同 UUID
  generation 继续串行。
- 等价 observer 必须共享一个真实 job id、完成结果与原因集合；失败/取消/panic 必须允许重新提交。
- entry、估算驻留 byte、oldest-active-age 必须是 typed 硬预算，成功缓存也必须受同一预算回收。
- index 解析、flow reservation、`begin_import`、job submit 与 backend 执行不得形成 state/index 双锁区。
- current-source 受管 Cargo、独立 review、百万级产品 trace、fixed return 和 managed commit 全部完成后才能关闭。

## 禁止临时方案

- 禁止扩大 Plan14 queue 或把阈值写死在 UI 调用方。
- 禁止新建 editor worker pool/import scheduler、复制 Runtime registry/digest 或写 `.zmeta`。
- 禁止用按 URI 串行冒充 single-flight，或在 failure/cancel 后永久缓存失败结果。
- 禁止把共享工作树偶然 GREEN、静态源码扫描或 10,000 次单元 storm 写成百万级产品验收。

## 修复结果与回传

Open：源码已实现精确 generation-keyed shared flight、revision 重验、UUID phase token、cleanup-before-result、
typed 三预算、失败逐出和共享取消；专项静态合同 7/7 GREEN，Rust 测试源码覆盖 10,000 次 storm、
admission-pending 快速失败、TOCTOU retry、hot-key TTL、动态结果 byte 回收、panic/shutdown 和路径迁移。
两位独立复审已关闭全部代码 finding，最终 Critical/Important/Minor=`0/0/0`。受管 Cargo、百万级产品
trace、F1/F4 import 产品证据、fixed return 与 managed commit 尚未完成，因此本记录保持 `status: open`。

2026-07-23 workspace edition 复核：exact rustfmt 从 crate root 递归解析时证明 `state.rs` 的 oldest-age
准入分支使用了 edition 2024 才允许的 let-chain，而 workspace `edition = "2021"`，因此此前源码候选会在
进入业务测试前解析失败。现已用语义等价的嵌套 `if let` + age predicate 硬切，并在专项静态合同加入
全 `import_flow/**/*.rs` let-chain guard；先观测该 guard RED，修复后专项门由 7/7 更新为 8/8 GREEN，
exact rustfmt/diff-check 通过。该修正不替代受管 Cargo、百万级 trace、current-source review 或 fixed return，
Failure 继续保持 open。

## 产出记录与时间

- 2026-07-22：登记旧实现的逐请求 job 放大、双锁临界区与无 entry/byte/age 背压问题；明确最低 owner
  为 Editor09 import admission，Runtime04 `AssetManager` 与 Plan14 `EditorJobSystem` 继续作为唯一既有 owner。
- 2026-07-22：完成 source-level 架构修复与 7/7 静态 GREEN；保留 open 状态等待独立复审、受管 Rust
  门、百万级产品 trace、failure fixed return 和受管提交，不提前写入关闭结论。
- 2026-07-22：独立初审暴露 admission ABA、UUID begin/clear 交界、result 早于 cleanup、registry
  generation TOCTOU、hot completed TTL 与动态结果 byte 未计入等问题；exact15 business candidate 已用
  flight-owned admission、
  revisioned generation、UUID phase token、cleanup-before-notify 及 completion-time/dynamic-byte 回收全部修复。
  双重复审的代码 finding 均为 `0/0/0`；Cargo/百万级产品证据/fixed return/commit 未完成，继续保持 open。
- 2026-07-30 current production caller复核：retained host的`import_model_requested`仍同步调用
  `import_model_into_project -> AssetManager::import_asset`，完全绕过本ImportFlow。因此现有single-flight与三预算
  对F4模型按钮不生效；模型、skeleton、每个clip和默认材质分别触发Runtime full import，A个animation至少
  A+3次全项目scan/resource prepare，并在UI callback阻塞。failure新增产品接入门：按钮只提交一个compound
  ticket，stage/glTF derive/Runtime transaction走Runtime11/04唯一owner；`product_ticket=1`、`scan/parse<=1`、
  `UI filesystem/import wall=0`。证据：`docs/plans/performance/01/2026-07-30-editor-retained-host-assets-current-review.md`；
  在F4 trace和managed Cargo前不得fixed return。
