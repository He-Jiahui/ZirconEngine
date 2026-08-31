---
report_id: Interface11
kind: current-source-failure-review
status: open
review_status: review_complete
implementation_status: source_fix_missing
baseline: 8aabbee3e99dc919f6da4611e3a44e8463a7fe7f
source_fingerprint: d489cce6b82f5b0a5b93defd16692b278310448585ac6584e75925287e1483c8
related_failure: docs/plans/optimize/zircon_runtime_interface/01/failure-2026-08-27-world-query-transform-snapshot-item-count.md
related_reports:
  - docs/plans/optimize/zircon_runtime_interface/09-runtime-host-foreign-output-safe-owner-admission-budget-fuse-observability-current-source-review.md
  - docs/plans/optimize/zircon_runtime_interface/10-serialization-project-resource-reflection-world-sync-public-data-contract-current-source-review.md
tests_not_run: true
---

# Runtime Host world-query item accounting failure 当前源码复核

## 1. 结论

`docs/plans/optimize/zircon_runtime_interface/01/failure-2026-08-27-world-query-transform-snapshot-item-count.md` 仍然是 open，且当前源码仍然重现其最低层根因。`zircon_runtime_interface::world_sync::WorldQueryResult` 已包含 `TransformSnapshot`，Runtime 的 dynamic frame counter 已在 `zircon_runtime/src/dynamic_api/frame.rs:165-182` 将它按一个 item 计数，但 Runtime Host 的 `zircon_runtime_host/src/foreign_output/item_count.rs:79-93` 的显式 `match` 没有该 arm。当前 source 不是“已经 fixed 等待回归”，而是仍然会在 Rust 编译阶段触发 non-exhaustive pattern E0004，阻止 Editor viewport 和 Plugins navigation 上层测试执行。

这是一个 P1 共享编译阻断，不是 Editor05 或 Navigation 的产品逻辑问题，也不应通过 wildcard、降低预算或删除 `TransformSnapshot` 来绕过。当前应保持 failure open，先修复 Host 的最低共享层并用受管 focused gate，再向上重放两个消费者测试。

## 2. 证据范围

| 集合 | files | lines | bytes | 指纹/状态 |
|---|---:|---:|---:|---|
| Host counter + Interface query/mod + Runtime producer | 4 | 647 | 21,152 | `d489cce6b82f5b0a5b93defd16692b278310448585ac6584e75925287e1483c8` |
| Open failure artifact | 1 | 约 75 | 约 4 KiB | `docs/plans/optimize/zircon_runtime_interface/01/failure-2026-08-27-world-query-transform-snapshot-item-count.md` |

主选择集包含：

- `zircon_runtime_interface/src/world_sync/query.rs`：query/result enum、request item count、transform result 构造。
- `zircon_runtime_interface/src/world_sync/mod.rs`：world-sync 公共 re-export。
- `zircon_runtime_host/src/foreign_output/item_count.rs`：Host JSON item accounting。
- `zircon_runtime/src/dynamic_api/frame.rs`：Runtime producer-side world query encoding/counter。

本轮只做静态 current-source review，没有运行 Cargo、managed validator、Editor05 viewport、Plugins05 navigation、Miri、fuzz 或真实 DLL。failure artifact 中记录的两次 managed reproductions（Editor05 job `62f609eb...`、Plugins05 job `0d26b703...`）是历史动态证据；本轮没有把它们伪装成新的通过证据。

## 3. 当前调用链

### 3.1 Interface result authority

`world_sync/query.rs` 在 `WorldQuery` 中定义 `TransformSnapshot` 请求（约第 79-83 行），在 `WorldQueryResult` 中定义 `TransformSnapshot { generation, world_replacement_epoch, entity, transform }`（约第 256-277 行）。同一文件的 `transform_snapshot_result` 在实体不存在时返回 `EntityMissing`，因此消费者必须对 `TransformSnapshot` 和 `EntityMissing` 都保持显式处理。

`WorldQuery::request_item_count` 将 Hierarchy、InspectionFields、TransformSnapshot 都视为一个请求 item；这不是对 result payload 的穷尽性证明，不能被 Host counter 当作替代。

### 3.2 Runtime producer authority

`zircon_runtime/src/dynamic_api/frame.rs` 的 `world_query_item_count` 对以下结果有完整 arm：ComponentRows、HierarchyRows、InspectionFields、TransformSnapshot、EntityMissing、NotModified。TransformSnapshot 的明确口径是 `1`，不把 position/rotation/scale 三个 scalar 分别计数。该实现是当前 producer-side 行为参考。

### 3.3 Host consumer defect

`zircon_runtime_host/src/foreign_output/item_count.rs` 的 `world_query_item_count` 当前只处理 ComponentRows、HierarchyRows、InspectionFields、EntityMissing、NotModified。缺少 `WorldQueryResult::TransformSnapshot { .. } => 1`，也没有编译期穷举测试。由于该函数被 `encode_world_query_payload` 调用，缺 arm 会在普通 library compile 阶段失败，而不是仅在 transform 请求运行时失败。

这解释了 failure artifact 的两次 E0004：上层测试数量为零并不代表 viewport/navigation 没有问题，而是它们从未通过 Host shared layer 的编译入口。

## 4. 差距与影响

| ID | severity/status | 发现 | 影响 |
|---|---|---|---|
| I11-P1-01 | P1 / Open | Host `world_query_item_count` 对 `WorldQueryResult` 非穷举，漏掉 TransformSnapshot；Runtime producer 与 Host consumer 的 item contract 已分叉。 | 普通 Host library compile 失败；所有依赖 Host world-query encoding 的上层 gate 在测试主体执行前停止。 |

已有 Interface09 的 P1-040（producer/Host 重复 item counter）和 P1-043（缺少统一 page/cursor abstraction）继续拥有架构级父问题，本报告不重复登记。I11-P1-01 只记录当前源码中的直接编译阻断和该 failure 的 current disposition。

当前状态不是以下任一项：

- 不是“用 `_ => 1` 处理未知 variant”的可接受修复；这会隐藏下一次 public DTO 演进。
- 不是“删去 TransformSnapshot”或把它降成 EntityMissing；这会破坏现有 interface contract 和 replacement epoch 语义。
- 不是“只修改 Host 单文件即可宣布完成”；failure 要求保留当前 mixed-tree 组合，并回放 RuntimeHost、Editor05、Plugins05 三层 gate。

## 5. 工程化修复路线

### M0 · 最低共享层穷举修复

在同一 owner change 中给 Host `world_query_item_count` 增加 `TransformSnapshot { .. } => 1`，保持 Runtime producer 的一项口径。保留显式 arms，让未来新增 `WorldQueryResult` variant 在编译期暴露所有下游消费者。

### M1 · 单一 item-accounting authority

把 item-accounting 规则移到 interface 或生成的 contract module，Host/Runtime 只调用同一个纯函数或由 schema manifest 生成的 visitor。若 ownership 约束不允许 interface 依赖 serde JSON 细节，则至少生成 variant coverage manifest，并用 cross-crate golden test 证明 Host 与 Runtime 的每个 variant 结果相等。

### M2 · focused regression

建立一个不依赖真实 DLL 的 Host focused test，构造所有 `WorldQueryResult` variants，断言 TransformSnapshot 为 1、EntityMissing/NotModified 仍为 1、rows 口径不变；测试必须在普通 `zircon_runtime_host` library gate 中执行。禁止只测函数文本或只测 ComponentRows。

### M3 · 上层 replay 与 failure 生命周期

按 failure artifact 的顺序执行：

1. RuntimeHost focused `world_query_item_count` gate。
2. Editor05 viewport focused gate，确认测试主体实际执行而非 compile-before-test。
3. Plugins05 Navigation focused gate，确认第二个独立 consumer 也通过。
4. 将完整 reproduction、source fingerprint、Cargo exit、测试计数和 cleanup receipt 写入 fixing plan 的 canonical `fixed-*` artifact；在此之前保持原 failure open。

## 6. 验收 Gate

| Gate | 必须证明 | 当前状态 |
|---|---|---|
| G1 exhaustive compile | Host 对当前 `WorldQueryResult` 变体穷举，新增 variant 会故意触发编译错误 | Fail：TransformSnapshot arm 缺失 |
| G2 count parity | Runtime producer 与 Host consumer 对六类结果返回相同 item count | Fail：Host 无法编译，且没有 parity test |
| G3 focused Host test | 直接构造 TransformSnapshot 并验证 count=1；所有既有 variants 无回归 | Not run / no current evidence |
| G4 Editor05 replay | viewport 测试真正执行且不在 Host E0004 前停止 | Not run；历史 job 为 compile stop |
| G5 Plugins05 replay | navigation 测试真正执行且不在 Host E0004 前停止 | Not run；历史 job 为 compile stop |
| G6 failure closeout | fixing plan 有 canonical fixed artifact，旧 failure 状态可追溯 | Open |

## 7. 路由与边界

- I11-P1-01 的最低 owner 是 `zircon_runtime_host`，不得转嫁给 Editor05/Plugins05。
- Interface09 继续拥有 Host foreign-output owner/admission/budget 总体架构；Interface10 继续拥有 world-sync DTO 的 snapshot/page/resync 设计。本报告只补充一个直接编译 failure 的 current-source evidence。
- 不改动 `docs/plans/mvp` 的门禁状态；该 failure 解释了为什么上层产品 gate 不能被静态源码存在替代。
- Tooling/Rust 迁移按用户要求排除。本轮未查询、轮询、等待或实时跟踪协调器，也未回滚共享工作树其他改动。

在 G1-G6 全部具备证据前，I11-P1-01 保持 `P1 / Open`，不得把“源码未来应增加一行”写成已修复事实。
