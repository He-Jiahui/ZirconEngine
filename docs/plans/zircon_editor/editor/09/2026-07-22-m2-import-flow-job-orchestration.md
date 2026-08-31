---
owner_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
milestone: M2
slice: import-flow-job-orchestration
status: source_updated_static_green_cargo_pending
related_code:
  - zircon_editor/src/core/asset/mod.rs
  - zircon_editor/src/core/asset/index.rs
  - zircon_editor/src/core/asset/import_flow/mod.rs
  - zircon_editor/src/core/asset/import_flow/error.rs
  - zircon_editor/src/core/asset/import_flow/flight.rs
  - zircon_editor/src/core/asset/import_flow/job.rs
  - zircon_editor/src/core/asset/import_flow/state.rs
  - zircon_editor/src/core/asset/import_flow/submit.rs
tests:
  - zircon_editor/tests/editor_asset_facade.rs
  - zircon_editor/src/core/asset/import_flow/tests.rs
  - zircon_editor/src/core/asset/import_flow/tests/concurrency.rs
  - tools/tests/test_editor09_import_flow_contract.py
---

# Editor09 M2.2 Import Flow Job Orchestration

Plan: `docs/plans/zircon_editor/editor/09-editor-asset-management.md`

Milestone: M2

Status: `source_updated_static_green_cargo_pending`

## 范围

本切片实现 Editor09 M2.2 的编辑器侧导入编排：复用 Plan14 `EditorJobSystem` 和 Runtime
`AssetManager::import_asset`，把 watch、既有 digest 失配及手动重导入统一成 typed request/ticket/result。
它不增加 worker pool、sidecar、digest、registry 或导入器 owner。

## 实施阶段

- [x] 注册并领取 8 文件精确 Session scope。
- [x] 以静态合同与 Rust 行为合同锁定 RED。
- [x] 建立 folder-backed `import_flow/` owner，并按 façade/error/job/state/test 拆分。
- [x] 同 URI 使用一个 Plan14 mutex group；URI+UUID 活跃计数避免排队任务提前清除 importing。
- [x] RAII lease 覆盖成功、失败、取消、panic 与提交拒绝，Runtime `CoreError` 保持 typed source。
- [x] 独立初审 `0/3/2` 暴露 URI→UUID 替换竞态、UUID 计数域错误与 pending-cancel 测试自锁；
  已统一 `state → index` 原子锁序、改为 flow 全局 UUID 计数并固定 Import test limit，同时补齐
  panic、shutdown submit rejection、owned status 与 0→1 progress 合同。
- [x] 修正后复审 `0/0/1` 仅要求锁定 registry 路径迁移回归；已补同一 UUID 的旧 URI job
  先结束而新 URI job 仍保持 `Importing` 的行为合同。
- [x] 最终独立复审 `0/0/0`。
- [x] 静态合同转 GREEN，精确 Rust 文件已格式化。
- [x] Editor02 将 `SharedEditorMessageBus::register_subscriber` 硬切为 typed `Result` 后，本切片的 job progress consumer 测试显式处理注册结果；无旧 infallible 调用残留。
- [x] 由 exact18 successor 将本 owner 挂载到 `core/asset/mod.rs` 的唯一 typed façade。
- [x] generation key 硬切为 `(UUID, normalized URI, Runtime source digest)`；同 generation 的
  watch/digest/manual observer 共享一个真实 Plan14 job、job id、结果与原因集合。
- [x] 新增 entry、估算驻留 byte、oldest-active-age 三类 typed admission limit；成功 flight 有界保留，
  失败/取消/panic flight 立即逐出并允许重试。
- [x] 移除 submit 的 state/index 双锁区；Runtime index 解析、flow reservation、`begin_import` 与 job submit
  分阶段执行，RAII 仍保证最后一个 UUID generation 才清除 `Importing`。
- [x] 专项静态合同更新为 7/7 GREEN，Rust 源码合同覆盖 10,000 次等价请求 storm、失败重试、三类预算
  与共享取消清理。
- [x] registered exact16 scope（含治理所需父计划路径）的 exact15 business candidate 独立初审发现
  admission ABA、UUID begin/clear 缝隙、cleanup-before-result、
  generation TOCTOU、completed TTL/byte 回收等 finding；已以 flight-owned admission、revision token、
  UUID phase token 和动态结果预算关闭，双重复审分别为 `0/0/0` 与 `0/0/1`，唯一计划元数据 Minor 已修正。
- [x] 将 899 行测试 owner 拆为 579 行基础合同与 `tests/concurrency.rs` 并发/预算合同，生产 `mod.rs`
  保持 267 行、`submit.rs` 独立承载准入状态机。
- [ ] 冻结 exact15 business source manifest，运行 Rust 聚焦门、failure fixed return 与受管 commit；父计划
  当前混合其他 Editor09 性能切片且既有 attribution 不属于本 Session，必须排除，禁止吸收。

## 测试阶段

- RED：生产 owner 缺失时，`test_editor09_import_flow_contract` 为 1 failure + 4 errors。
- GREEN：`python -m unittest tools.tests.test_editor09_import_flow_contract -v` 为 7/7。
- 待受管 Cargo 验收的 Rust 合同：成功与 owned status、typed backend failure、未知 URI、generation
  single-flight、共享 cancel 清理、失败重试、三类 admission limit、panic、shutdown submit rejection 与
  0→1 progress event sequence。
  同一 UUID 的 registry 路径迁移由独立回归锁定，最后一个跨 URI lease 释放前不得清状态。
- Cargo 验收必须使用含 `core/asset/mod.rs` 与 crate 外部 façade consumer 的新冻结 manifest；不得复用
  挂载前 snapshot，也不得绕过 Coordinator01 validation-copy Cargo local-path manifest graph 闭包故障。
- 2026-07-23 workspace edition 复核发现 `state.rs` oldest-age admission 使用 edition 2024 let-chain，
  与 workspace edition 2021 冲突；已以嵌套条件保持相同准入语义，并新增全 import-flow source guard。
  该 guard 先 RED 后转 GREEN，专项静态门现为 8/8；snapshot961 因 current hash 合法漂移而不得复用，
  后续受管 Cargo 前必须冻结 successor manifest。

## 架构裁决

- Runtime `AssetManager` 是现有 worker pool/request/completion 的唯一公开 owner；Editor09 不直接构造
  `AssetWorkerPool`，避免第二调度真源。
- `EditorAssetIndex` 仅保存 importing 等展示瞬态；registry、UUID/path、`.zmeta` v7、settings、digest
  与 artifact 仍由 Runtime/Plan10 权威持有。
- 同 UUID mutex group 只在活跃期驻留，最后一个 generation 结束并完成 index clear 后回收，避免路径迁移
  并发写同一来源，也避免导入过大量 UUID 后无界增长。
- M2.1 与 M2.2 不再以两个独立 manifest 争用 `core/asset/mod.rs`。当前 exact18 successor 同时领取
  import/dirty owner 与共享 façade；子模块保持私有，调用方只走 `core::asset` typed re-export，不保留
  nested-path 兼容入口。M2.1 index 继续是既有唯一投影 owner，本切片不复制其状态。

## 产出记录与时间

- 2026-07-22：状态 `source_complete_static_green_mount_pending`。完成 8 文件精确 scope、TDD
  RED→GREEN（静态 6/6）、folder-backed 导入编排、typed request/result/submit error、Plan14 Import job、
  Runtime AssetManager adapter、同 URI mutex group、URI+UUID 引用计数及 RAII cleanup。Rust 行为测试源码
  已覆盖成功/失败/串行/取消，但 owner 尚未挂载，故不宣称 Cargo GREEN；等待 M2.1 Text04 依赖回传并
  完成精确受管提交后再挂载、冻结、验证和复审，父 M2 保持 `pending`。
- 2026-07-22：独立初审 `0/3/2` 与复审 `0/0/1` 的全部 finding 已关闭：URI→UUID
  解析/reservation/importing 标记及 Drop 清理统一为 `state → index` 锁序，UUID 引用计数提升为
  flow 全局，Import=1 pending-cancel 合同不再自锁，并新增 panic、shutdown rejection、owned status、
  0→1 progress 及同 UUID 跨 URI 路径迁移回归。最终复审 `0/0/0`，静态合同保持 6/6 GREEN；
  挂载前仍不宣称 Rust/Cargo GREEN。
- 2026-07-22 性能复核：同URI mutex group当前只串行、不single-flight；同URI或同UUID/source
  generation的watch/digest/manual storm仍会逐项提交job，队列与URI label/progress分配随请求增长，
  submit还为原子迁移同时持state/index锁。新增
  [open failure](failure-2026-07-22-asset-import-duplicate-admission-backpressure.md)，由Editor09/14
  联动Runtime04/11实现generation-keyed共享ticket与entry/byte/age预算；不以扩大queue代替背压。
- 2026-07-22 Editor02 consumer hard-cut 回传：snapshot838 的 8 文件中仅本记录与
  `import_flow/tests.rs` 合法漂移；测试现对 `register_subscriber` 使用 `.unwrap()`，保持 typed failure
  可见且不改变生产 owner。当前 exact8 静态门 `6/6 GREEN`，rustfmt 与 diff-check 已刷新并通过；
  独立增量复审 `0/0/1` 的唯一计划措辞 Minor 已修正，最终极小复核 `0/0/0`。owner 仍未挂载，受管 Cargo 与
  milestone commit 继续 pending，禁止把本次 API 对齐提升为 M2.2 完成。
- 2026-07-22：状态 `source_mounted_static_green_review_clean_cargo_blocked`。exact18 successor 已在
  `core/asset/mod.rs` 私有挂载 `import_flow` 并重导出 6 个 typed API，新增 crate 外部 Rust consumer
  锁定唯一 `core::asset` 入口；挂载合同按 TDD 由新增项 RED 收敛为 import `7/7`，与 dirty 合并静态门
  `14/14`，exact Rust rustfmt 通过。独立初审 `0/0/2` 的双公共路径与字符串假绿均已修复，终审
  Critical/Important/Minor=`0/0/0`；baseline221 候选 snapshot916 已冻结 exact18 全部路径。
  受管 Cargo 仍被 Coordinator01 validation-copy manifest graph 闭包故障阻断；snapshot、managed commit
  未完成前，父 M2 保持 `pending`。
- 2026-07-22：状态 `source_complete_static_green_review_pending_cargo_blocked`。针对 open performance
  failure 将旧“逐请求入队、同 URI 仅串行”实现硬切为 generation single-flight：10,000 个等价 observer
  共享一个 job id/result/reason set，失败可重试，共享取消仅释放一次 importing 生命周期；新增
  `EditorAssetImportAdmissionLimits` 的 entry/byte/oldest-active-age typed 背压，并消除 state/index 同持锁。
  专项静态合同 7/7 GREEN、exact Rust rustfmt 通过；独立复审、受管 Cargo、百万级产品 trace、failure fixed
  return 与 milestone commit 仍 pending，故不宣称性能 failure 已关闭，父 M2 继续保持 `pending`。
- 2026-07-22：状态 `source_complete_static_green_review_clean_cargo_blocked`。registered exact16 successor
  纳入治理所需父计划路径；exact15 business candidate 纳入 `index.rs`、独立 `submit.rs` 与
  `tests/concurrency.rs`。父计划当前含其他 Editor09 性能切片且 attribution 属于既有 Session，故不进入
  本次 snapshot/commit manifest。generation identity 改为
  UUID+完整规范化 URI+完整 Runtime digest 精确相等，index revision 在返回 cached ticket/提交新 job 前复验，
  首 generation 的复验与 `Importing` begin 同锁线性化。flight 自有 admission 消除旧 job-id ABA，UUID
  `Starting/Ready/Clearing` token 阻断 stale clear，lease 在 state/index cleanup 后才发布 result；成功缓存
  以 completion time 做 hot-key 惰性 TTL，并把动态 `AssetStatusRecord` 字符串纳入 byte budget，超预算不缓存。
  TDD 静态门 7/7、Editor09 相关门 29/29、exact rustfmt/结构预算通过；两位独立复审的代码 finding 均关闭，
  最终 Critical/Important/Minor=`0/0/0`。受管 Cargo、百万级产品 trace、failure fixed return、final source
  snapshot 与 milestone commit 仍 pending，本 failure 和父 M2 保持未关闭。
- 2026-07-22：candidate snapshot961 已冻结 exact15 business manifest，preview 15/15 全部
  `would_change=false`；registered scope 中的 foreign-mixed 父计划明确未进入 snapshot。该证据只证明
  current-source 冻结与 attribution 完整，不替代受管 Cargo、百万级产品 trace、fixed return 或 commit。
- 2026-07-30 product reachability复核：`zircon_editor/src/ui/retained_host/app/assets/workspace.rs`的手动模型
  导入仍直接同步调用Runtime `AssetManager::import_asset`，本M2.2没有生产caller的旧判断仍成立，但这不是可接受
  的完成态。F4按钮必须挂载唯一`EditorAssetImportFlow` façade，并以一个compound request覆盖model、derived
  skeleton/clips和default material；禁止为每个派生URI提交独立job或在UI callback等待ticket。Runtime04负责一次
  candidate transaction，Runtime11负责stage/parse/derive/write。当前A个animation至少A+3次full import；managed
  product trace、Cargo与独立复审前本记录继续pending。
- 2026-08-23：状态 `source_updated_static_green_cargo_pending`。复审提交路径发现 shared flight admission
  与 UUID `Starting/Clearing` 曾通过 Condvar 等待；现已硬切为 `AdmissionPending` 与
  `UuidLifecycleTransitionPending` typed 返回，并将持续 Runtime generation 变更限制为一次内联复验后返回
  `RegistryGenerationSuperseded`。新增并发合同覆盖两类 pending 与重复 generation 变更时 backend/job submit=0；
  index 锁恢复集中至私有 helper。`test_editor09_import_flow_contract` 为 8/8、精确 rustfmt 与 diff-check 通过。
  本条仅记录 current-source 与静态合同，Rust 行为用例、受管 Cargo、产品 trace、独立复审、failure fixed return 和
  milestone commit 仍未完成，父 M2 继续 `pending`。
- 2026-08-23：状态保持 `source_updated_static_green_cargo_pending`。Runtime
  `AssetManager::import_asset` 返回 `None` 不再被 `EditorAssetImportResult` 表示为成功；结果的 status
  现为必备 `AssetStatusRecord`，缺失 committed status 统一返回 typed
  `EditorAssetImportExecutionError::RuntimeDidNotCommit`，且 lease 仍清理 importing 生命周期。公开 ticket 的无界
  `wait()` 也硬切为 `wait_until(Instant)`，超时显式保持 pending；新增行为合同并将 façade allowlist 同步为 8/8 GREEN，
  精确 rustfmt 与 diff-check 通过。未运行 Rust/Cargo，不构成产品接入、性能数据、failure fixed return 或里程碑验收，
  父 M2 继续 `pending`。
