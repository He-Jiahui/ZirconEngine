---
report_id: Interface11
kind: current-source-failure-review
status: open
review_status: review_complete
implementation_status: source_exhaustive_validation_pending
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

`docs/plans/optimize/zircon_runtime_interface/01/failure-2026-08-27-world-query-transform-snapshot-item-count.md` 仍然是 open，因为它尚未取得 RuntimeHost、Editor05 和 Plugins05 的受管回归回执；但当前源码已不再重现该 failure 的最低层缺口。`zircon_runtime_interface::world_sync::WorldQueryResult` 包含 `TransformSnapshot`，Runtime 的 dynamic frame counter 已在 `zircon_runtime/src/dynamic_api/frame.rs:165-182` 将它按一个 item 计数，Runtime Host 的 `zircon_runtime_host/src/foreign_output/item_count.rs:79-93` 现在也有显式 `TransformSnapshot { .. } => 1` arm，并由 focused test 覆盖全部六个当前变体。Cargo/上层 replay 尚未执行，因此不能把历史 E0004 视为已验收关闭。

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

### 3.3 Host consumer history and current source

历史 failure snapshot 中，`zircon_runtime_host/src/foreign_output/item_count.rs` 的 `world_query_item_count` 只处理 ComponentRows、HierarchyRows、InspectionFields、EntityMissing、NotModified，缺少 `WorldQueryResult::TransformSnapshot { .. } => 1`，也没有编译期穷举测试。由于该函数被 `encode_world_query_payload` 调用，缺 arm 会在普通 library compile 阶段失败，而不是仅在 transform 请求运行时失败。

当前源码已补上显式 arm，并在 `zircon_runtime_host/src/foreign_output/tests.rs` 构造 ComponentRows、HierarchyRows、InspectionFields、TransformSnapshot、EntityMissing、NotModified 六个变体，断言对应结构化 item count。这个静态修复不等于受管编译或上层 replay 通过。

这解释了 failure artifact 的两次 E0004：上层测试数量为零并不代表 viewport/navigation 没有问题，而是它们从未通过 Host shared layer 的编译入口。

## 4. 差距与影响

| ID | severity/status | 发现 | 影响 |
|---|---|---|---|
| I11-P1-01 | P1 / source fixed, validation pending | 历史 Host `world_query_item_count` 曾非穷举并漏掉 TransformSnapshot；当前源码已补显式 arm 和六变体 focused regression，Runtime producer 与 Host consumer 的 item contract 已对齐。 | 受管 RuntimeHost 编译、Editor05 viewport、Plugins05 Navigation replay 和 failure closeout 仍待执行；不能把静态修复当作产品验收。 |

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
| G1 exhaustive compile | Host 对当前 `WorldQueryResult` 变体穷举，新增 variant 会故意触发编译错误 | Static source pass：六个显式 arms；受管 compile pending |
| G2 count parity | Runtime producer 与 Host consumer 对六类结果返回相同 item count | Static source/test pass：producer 与 focused Host test 均为六变体口径；受管 parity pending |
| G3 focused Host test | 直接构造 TransformSnapshot 并验证 count=1；所有既有 variants 无回归 | Test source present and rustfmt/static checks pass；managed execution pending |
| G4 Editor05 replay | viewport 测试真正执行且不在 Host E0004 前停止 | Not run；历史 job 为 compile stop |
| G5 Plugins05 replay | navigation 测试真正执行且不在 Host E0004 前停止 | Not run；历史 job 为 compile stop |
| G6 failure closeout | fixing plan 有 canonical fixed artifact，旧 failure 状态可追溯 | Open |

## 7. 路由与边界

- I11-P1-01 的最低 owner 是 `zircon_runtime_host`，不得转嫁给 Editor05/Plugins05。
- Interface09 继续拥有 Host foreign-output owner/admission/budget 总体架构；Interface10 继续拥有 world-sync DTO 的 snapshot/page/resync 设计。本报告只补充一个直接编译 failure 的 current-source evidence。
- 不改动 `docs/plans/mvp` 的门禁状态；该 failure 解释了为什么上层产品 gate 不能被静态源码存在替代。
- Tooling/Rust 迁移按用户要求排除。本轮未查询、轮询、等待或实时跟踪协调器，也未回滚共享工作树其他改动。

在 G1-G6 全部具备证据前，I11-P1-01 保持 `P1 / Open`，不得把“源码未来应增加一行”写成已修复事实。

## 8. 2026-09-19 current-source reconciliation

The shared source now contains the minimum exhaustive repair and focused
regression described above. Current SHA-256 fingerprints are:

```text
zircon_runtime_host/src/foreign_output/item_count.rs 7E643D75BC34A77A5A596A59351184AD679A10197B613A7C7255B7D00F66E322
zircon_runtime_host/src/foreign_output/tests.rs 6A4D4B4D0E88B6CB352F5BF0BBC2689546944230768714F9D41F6936DF577661
zircon_runtime_interface/src/world_sync/query.rs 56F357DDDD79E119CA894B195966658025731B67BC102377A386FDA62A7AAA47
zircon_runtime/src/dynamic_api/frame.rs 0EAF943FF6C0CA122AD18F29150A4EDB090DD69115F1F156330CB8322D3B3DFC
```

Scoped source inspection, `rustfmt --edition 2021 --check`, and `git diff
--check` are the available current-source evidence. No Cargo, managed
validator, Editor05/Plugins05 replay, Miri, fuzz, or real DLL/GPU run was
performed in this reconciliation. The related failure therefore remains open
until those gates produce terminal receipts.
