# Editor02 M1.1 WorldSync contract closeout

Plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
Milestone: M1.1
Status: completed
Files: ["zircon_runtime_interface/src/world_sync/query.rs", "zircon_runtime_interface/src/tests/world_sync_contracts.rs", "docs/zircon_runtime_interface/world_sync.md", "docs/plans/zircon_editor/editor/02/2026-07-16-m1-1-world-sync-contract-closeout.md"]

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与验收边界 |
|---|---|---|
| `IMPLEMENTED / EXACT ACCEPTANCE` | 2026-07-16 | `zircon_runtime_interface::world_sync` 四文件继续作为唯一 transport-neutral DTO owner；本收口在 `WorldQuery::result_for_generation` 固定饱和世代必须返回 rows、非短路结果按稳定 entity identity 升序 canonicalize，并以 current serde/typed watch/invalidation 合同验证 wire surface。父计划定义受保护且不进入 business manifest，父 M1、M1.2 与 M1.3 不随本切片提升。 |
| `NO LEGACY COMPATIBILITY` | 2026-07-16 | 未增加旧字段、facade、alias、编辑器状态载荷或第二查询实现；未知 retired wire 字段继续拒绝，runtime/editor 领域行为不得下沉到接口 DTO。 |
| `EVIDENCE OWNED BY COORDINATOR` | 2026-07-16 | exact 4-file manifest 的 managed validation、independent review、fingerprint 与 commit SHA 由 Coordinator 事件账本绑定；本记录不写入会改变自身哈希的 run/commit 结果。 |

## Scope delivered

- 保留 `query.rs`、`watch.rs`、`invalidation.rs` 与 `mod.rs` 的四文件物理边界，未引入兼容模块。
- `NotModified` 仅在非饱和权威世代与 hint 精确相等时返回；`u64::MAX` 强制返回 rows。
- rows 在 DTO helper 中按 `EntityId` canonicalize，组件映射继续使用 `BTreeMap`。
- 父计划定义不进入 business manifest；M1.1 的具体完成状态只记录在本编号子计划中，M1.2/M1.3 及父 M1 状态保持不变。

## Fresh testing evidence

- exact current-source validation 由 Coordinator 的 managed validation evidence 绑定到本文件清单与输入指纹。
- focused contract 覆盖 query serde、matching/stale/missing hint、saturated generation、row canonicalization、typed watch token、invalidation batch 与 unknown-field rejection。

## Review

- 独立 reviewer 必须检查 exact 4-file manifest、饱和世代语义、稳定排序、wire compatibility hard cut 与父计划状态；review evidence 只接受 `Critical=0 / Important=0`。

## 后续门禁

- M1.2 继续负责 runtime world generation、split inspection 与 subtree hash。
- M1.3 继续负责深 hierarchy/cycle edge 投影硬化及其独立 failure chain。
- M1.1 不得被用作父 M1 或 Editor02 总计划完成证据。
