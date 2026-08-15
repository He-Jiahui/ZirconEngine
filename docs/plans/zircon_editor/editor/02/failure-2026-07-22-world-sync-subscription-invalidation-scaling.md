---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: world-sync-subscription-invalidation-scaling
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/inspection/mod.rs
  - zircon_runtime/src/scene/inspection/subscription.rs
  - zircon_runtime/src/scene/inspection/subscription/tests.rs
  - zircon_runtime/tests/runtime_world_sync_subscription_table.rs
tests:
  - cargo test -p zircon_runtime --test runtime_world_sync_subscription_table --locked --jobs 1 -- --nocapture --test-threads=1
  - 1/1k/100k watch and mutation-storm scale fixtures
---

# Editor02：World sync subscription invalidation规模交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime scene inspection新增subscription增量性能审查，PERF-MVP-468
- 修复责任计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 交接原因：Editor02拥有WatchKey、SubscriptionTable、gateway pump与view dirty协议；Runtime inspection只是最低共享实现落点。
- 生命周期键：`world-sync-subscription-invalidation-scaling`

## 失败现象与复现证据

`invalidate_subtree`为每个WatchKey重新判断variant；每个Subtree watch又新建BTreeSet并从同一entity沿parent chain走到root/cycle，复杂度O(watches×depth×log depth)。`invalidate_all_assets`扫描全部异构key并collect临时Vec，component type invalidation为map lookup分配String。spawn/reparent/reload storm的pending facts在frame flush前没有显式count/bytes预算。

新增实现尚未接入全部mutation throat；本交接不否定其token lifecycle/确定性flush基础合同，也不把尚未运行的动态规模门写成通过。

## 最低共享层根因

单一`BTreeMap<WatchKey,...>`方便通用注册，却让触发端缺少按variant和root/component/asset identity的直接索引；subtree判断以“每watch重新走entity ancestry”实现。事实队列只有帧末flush，没有producer burst预算/coalesce政策。

## 架构修复验收

- SubscriptionTable按variant拥有direct maps：world tokens、subtree root→tokens、component type id→tokens、asset id→tokens；by-token仍为唯一unwatch反查。
- 单结构fact只构造或借用一次bounded ancestor chain，逐ancestor root直接取tokens；reparent before/after各一次，不随watch总数重复走链。
- component type使用interned/borrowed identity，lookup不分配String；asset reload只访问asset index，不扫描其他key或collect临时Vec。
- facts按语义coalesce并有count/bytes/age预算与overflow诊断；critical structure facts不静默丢失。
- watches/depth/facts 1/1k/100k记录ancestor walks/visited alloc、key probes、pending peak/age/drop和p95：ancestor walk≤1/fact、工作近depth+matched tokens、队列/RSS有界。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止保留generic by-key和四张typed map的双注册truth；typed maps可由单一registration authority原子维护。
- 禁止缓存上一次ancestor Vec但无hierarchy generation失效；rename/reparent/despawn必须精确失效。
- 禁止只给pending Vec reserve或提高上限来替代burst budget/coalesce/backpressure。

## 修复结果与回传

Open state: `2026-08-05 subscription support slice independent second review 0/0/0；structural in-process watch lifecycle、reflected query、typed/dynamic component、asset watch、editor pump 与 V5 ABI source static green；live-token collision increment、renewed second review 与受管 Cargo 验收待完成`; no dynamic pass or fixed return is claimed.

- generic `by_key` 已删除；`by_token` 原子维护 world/subtree/component/asset typed indexes。
- subtree invalidation 每 fact 只构造一条 cycle-guarded ancestor chain；component lookup 借用 `&str`；aggregate reload 只遍历 asset index。
- pending facts 以 entity/scene/reload key 合并，并受 count/估算 bytes/generation age 预算约束；overflow/age breach 留下 dirty resync 与累计诊断。
- r3 静态 TDD 从 `5 failed / 1 passed` 收敛至 `6/6 GREEN`；100k integration fixture 已落盘但尚未取得受管终态。
- failure `related_code` 已从不可哈希的目录占位符收敛为本修复实际拥有的 exact4 Rust 文件，供 source-bound failure priority 与 fixed return 审计；未吸收未修改的 interface/editor 目录。
- 2026-07-22 editor consumer补证：WorldWatchMap同view多token已用borrowed mark把ViewInstanceId clone降为
  unique dirty views；但每batch仍建立seen/duplicate/unknown三套BTreeSet，`InvalidationBatch.dirty`没有
  count/bytes/canonical标志。最终Runtime flush须发布bounded sorted-unique batch/cursor，Editor normal快路不再
  O(D logD)重验，malformed transport才进入诊断慢路；Cargo/100k产品证据未完成，failure保持open。
- 2026-08-05 structural forward repair：`LevelSystem` 现在持有唯一 session-owned
  `SubscriptionTable`；live `World` 附加不可 clone/deserialize 的内部 sink，避免 staging/snapshot
  污染 live invalidation。`spawn_node`、typed/bundle/node-record spawn、despawn 与 reparent before/after
  都以 `WorldFact` 写入该表，InProcess gateway/稳定 handle 能 watch、unwatch 与 drain。reflected query
  支持 type-path with/without、selected fields 与 generation-hint short circuit。component/asset throat、editor
  pump 与 ABI 尚未接线，故不回传 fixed。
- 2026-08-05 typed component forward repair：普通 `insert/remove/get_mut` 和 bundle preflight 写入都从其
  唯一 mutation hook 调用同一 subscription table 的 borrowed type-path index；`Name` 的真实 reflection
  type path 回归覆盖成功路由。dynamic component 和 asset reload 各有独立 mutation throat，仍待接线。
- 2026-08-05 dynamic component forward repair：attach、remove 以及 VM/non-VM JSON property update 的四个
  成功提交点直接使用 canonical dynamic component id 命中同一 typed index；`tests.Health` 连续更新回归
  验证同帧仅生成一个 dirty token。asset reload、editor pump 与 ABI 仍待接线。
- 2026-08-05 asset reload forward repair：`RuntimeDynamicSession::tick_scene_asset_reload` 从唯一 frame
  apply report 构造 `AssetReloadApplied` DTO，再由 LevelSystem 写入 session table；无活动 report 不入队，
  pending count 与 applied/failed/stale 保留在协议事实中。editor pump 与 ABI 仍待接线。

## 产出记录与时间

| 里程碑 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|
| PERF-MVP-468 / Editor02 M2.1 source repair | `source_complete_static_green_validation_pending` | 2026-07-22 | exact8 源码、回归、模块文档与 failure record 已更新；typed direct routing、single ancestry walk、bounded semantic coalesce、overflow diagnostics 和 100k fixture 已落盘，静态合同 `6/6 GREEN`。Cargo、性能原始输出、独立复审与 canonical failure return 待完成。 |
| Editor02 M2.1 subscription support-slice independent review | `source_static_green / independent_second_review_green / m2_wiring_and_managed_validation_pending` | 2026-08-05 | exact4 Rust scope 通过 rustfmt 与 scoped diff 静态检查；独立二审 `Critical/Important/Minor = 0/0/0`。`record_fact` 尚未由生产 mutation throat 调用，gateway 与 editor pump 接线也未完成；未执行 Cargo，failure 保持 open。 |
| Editor02 M2.1 structural in-process watch lifecycle | `source_static_green / renewed_second_review_pending / managed_validation_pending` | 2026-08-05 | exact11 Runtime/editor/test files：LevelSystem session table、non-cloning World sink、structural fact throats及 InProcess gateway/handle watch lifecycle 已落盘；rustfmt 与 scoped diff 通过。component/asset/query/pump/ABI 未纳入本切片，未执行 Cargo，failure 保持 open。 |
| Editor02 M2.1 reflected query in-process boundary | `source_static_green / renewed_second_review_pending / managed_validation_pending` | 2026-08-05 | exact6 Runtime/editor/test files：editor-visible reflected component type-path with/without filter、selected field values、generation-hint `NotModified` 和 InProcess/handle 转发已落盘；rustfmt 与 scoped diff 通过。component/asset/pump/ABI 未纳入本切片，未执行 Cargo，failure 保持 open。 |
| Editor02 M2.1 typed component watch throat | `source_static_green / renewed_second_review_pending / managed_validation_pending` | 2026-08-05 | exact3 Runtime/editor/test files：普通与 bundle typed mutation hooks 直接命中 borrowed component type index，真实 `Name` reflection type-path 回归已落盘；rustfmt 与 scoped diff 通过。dynamic component/asset/pump/ABI 未纳入本切片，未执行 Cargo，failure 保持 open。 |
| Editor02 M2.1 dynamic component watch throat | `source_static_green / renewed_second_review_pending / managed_validation_pending` | 2026-08-05 | exact2 Runtime/editor/test files：dynamic component attach/remove/VM 与 non-VM JSON update 成功路径均命中 canonical type id，连续 `tests.Health` 更新回归已落盘；rustfmt 与 scoped diff 通过。asset/pump/ABI 未纳入本切片，未执行 Cargo，failure 保持 open。 |
| Editor02 M2.1 asset reload fact throat | `source_static_green / renewed_second_review_pending / managed_validation_pending` | 2026-08-05 | exact2 Runtime files：动态 scene reload frame report 在唯一 session tick 出口转换为 `AssetReloadApplied`，无活动不入队、DTO 保留 apply/failed/stale/pending；映射回归、rustfmt 与 scoped diff 通过。editor pump/ABI 未纳入本切片，未执行 Cargo，failure 保持 open。 |
| Editor02 M2.2 pump + M3.2 V5 ABI hard cut | `source_static_green / renewed_second_review_pending / managed_validation_pending` | 2026-08-05 | `WorldSyncPump` 每 editor frame 只 drain 一次、发布 immutable world facts 并投影 token→view dirty；SessionGateway/LoadedRuntime/runtime export/interface table 统一硬切至 24-field `ZrRuntimeApiV5`，query/watch/unwatch/drain 均有 ABI owner、panic wrapper、loader-required pointer 与 owned-buffer 合同测试。动态 API 子审计为 51/51 source、24 fields、22 wrappers、legacy hits 0、risks 0；Python compile、rustfmt check、scoped diff 均通过。完整 structural audit 64s 超时，不作通过证据；未执行 Cargo，M2 hierarchy incremental projection 与 M3.1 dirty binding 仍未完成，failure 保持 open。 |
| Editor02 M2 hierarchy anchor + M3.1 binding model hard cut | `source_partial / managed_validation_requeued / downstream_layout09_failure_open` | 2026-08-05 | runtime inspection 以 base rows + immutable sparse overrides 发布 Name mutation 的变更行与祖先 subtree hash，不为 WorldSync delta 物化整表；结构 dirty 先经 `SceneInspectionMessage` 的 entity/parent/depth/subtree-hash anchor 投递，只有额外非结构 UI 脏化才刷新 workbench snapshot。`WorldWatchBinding` 已硬切为保存 `WatchRegistration` 的显式 `depends_on: Vec<WatchKey>`；5,000-node rename 断言只重建 changed anchor 与 parent hash，且 source-only 观测确认 sparse override 为 2、未物化 row slice。审计同时确认 `watch_edit_world_for_view` 尚无 view-lifecycle caller，且 retained hierarchy 无 scene message consumer；不可将 binding model 误记为 M3.1 end-to-end green。最低 shared cause 已前向交接至 Layout09 `retained-hierarchy-dirty-refresh-full-snapshot-fallback`。订阅 scratch 变更已使旧 runtime receipt `9adebc0696074bc793f68cedd6004864` 与 editor receipt `d32e3d0e401d422cbefff905656985f8` 不能再代表完整 current source；新的 8-file runtime reservation 为 `6780156c423843c58edde3e31e401c87`（manifest fingerprint `182d630d8055aa5b90449d11fe44fee14e25ba6d85d3a4b0e0e1601247ea9f20`）。全部仅为未轮询 receipt，failure 保持 open。 |
| Editor02 M2.1 subscription ancestry scratch reuse | `source_forward_repair_static_green / managed_validation_requeued / independent_second_review_pending` | 2026-08-05 | `SubscriptionTable` 以 session-owned `Vec<EntityId>` 与 cycle-guard `HashSet<EntityId>` 复用单次 ancestry walk 的工作区；热态同深度 subtree invalidation 不再构造临时 chain 或增长 scratch 容量，容量增长才计入 allocation diagnostics。新增回归冻结该热路径，`rustfmt --check` 与 scoped diff 通过。两文件 scratch-only reservation `b0dd74d8aefb422482c9eabed58f3f3f` 未启动即由本 session 释放，避免它覆盖不了 hierarchy 其余 current inputs；替代的 8-file Windows reservation 是 `6780156c423843c58edde3e31e401c87`，fingerprint `182d630d8055aa5b90449d11fe44fee14e25ba6d85d3a4b0e0e1601247ea9f20`。仅有 pending receipt，未轮询、不作 green 证据；failure 保持 open，待二审和受管终态。 |
| Editor02 M2.1 borrowed dirty-view batch projection | `source_forward_repair_static_green / editor_managed_validation_successor_pending / independent_second_review_pending` | 2026-08-05 | `WorldSyncPump` 不再把每个 projected `ViewInstanceId` clone 后交给 bus，也不为同一 runtime batch 的每个 view 单独获取 bus mutex；`EditorMessageBus` 与 `SharedEditorMessageBus` 新增 borrowed dirty entry 和 `ViewDirtySet` batch entry，最终在一次锁持有中委托既有 `ViewDirtySet::mark_ref`，仅首次进入脏集时复制 id。回归冻结 single-view borrowed 链、pump -> shared bus -> dirty-set batch 链，以及 shared bus 合并既有 mask 与新 view 的行为；rustfmt、scoped diff 与旧逐-view pump 调用静态审计通过。当前 session 的唯一 CPU reservation 已保留给更完整的 8-file Runtime hierarchy gate，故本 editor-only 切片待其协调器周期结束后创建新的 current-source managed reservation；未轮询、不作 green 证据，failure 保持 open。 |
| Editor02 M3.1 exact view-watch idempotence | `source_forward_repair_static_green / editor_managed_validation_successor_pending / independent_second_review_pending` | 2026-08-05 | `WorldWatchMap::token_for` 以同一 view、单一 explicit `WatchKey` 与相同 mask 精确匹配既有 token；`WorldSyncPump::watch_view` 在调用 runtime 前返回该 token，因此重复注册不再生成 runtime subscription。不同 key 或 mask 不会误合并，gateway generation replacement 后旧 bindings 已清空而可重建。回归覆盖重复注册 token 相等、watch map 保持一项、结构 mutation 只命中一次及不同依赖不复用；`tools.tests.test_editor02_world_sync_watch_map_contract` 已 `6/6 GREEN`，且 rustfmt/scoped diff/旧路径静态审计通过。Layout09 仍须实际调用 watch/unwatch lifecycle，未轮询或声称 managed green，failure 保持 open。 |
| Editor02 M2.1 canonical dirty-token projection | `source_forward_repair_static_green / managed_validation_queued / independent_second_review_pending` | 2026-08-05 | `SubscriptionTable` 的 ordered dirty set 被提升为 `InvalidationBatch` strict-ascending/unique contract；editor normal path 仅作线性 canonical 检查并直接 token→view 投影，不再为已规范 batch 分配 `seen`、`duplicates`、`unknown` 三套 `BTreeSet`。乱序或重复 wire input 继续进入诊断慢路并保留原有重复/未知 token 报告。 | current-source interface/runtime/editor tickets 分别为 `518893fa6c154fe296bdc2164a6e5943`、`b9b1df9714d64f04967094b201e81fe1`、`8904afd7a8d74866a4b630745e901cda`；均仅有 queued receipt，未轮询且不作为 green 证据。failure 保持 open，待二审与受管终态。 |
| Editor02 M2.1 live watch-token collision safety | `source_forward_repair_static_green / independent_second_review_pending / managed_validation_pending` | 2026-08-05 | `WorldSyncPump::watch_view` 在 runtime 返回 token 后再次同步 gateway generation，允许新 session 合法复用旧 opaque token 值；同 session 的已绑定 token 才 typed-reject，并保留既有 editor binding。删除会对 collision token 执行补偿 `unwatch` 的分支，避免错误撤销旧订阅而让 local map 悬空；普通本地 bind 失败后的新 token 清理语义不变。 | 新 Rust 回归锁定 collision rejection 后原 binding 仍存在；WorldSync Python contract 增加 returned-token generation recheck、collision branch 不得 `unwatch_world` 的结构断言，现为 `7/7 GREEN`。`rustfmt --check --edition 2024`、Python compile 与 scoped diff 检查通过；Cargo 和独立二审尚未执行，failure 保持 open。 |
