---
related_code:
  - zircon_runtime_interface/Cargo.toml
  - zircon_runtime_interface/src/serialization
  - zircon_runtime_interface/src/tests/boundary.rs
  - docs/zircon_runtime_interface/serialization.md
plan_sources:
  - docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
status: slice-accepted-parent-m3-pending
---

# Editor11 M3.1 二进制版本壳

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 完成项目与证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M3 | 3.1 Binary 编码选型与文本等价合同 | `实现完成-interface 263/263-binary 13/13-M3测试阶段待M3.2` | 2026-07-14 | `zircon_runtime_interface/src/serialization/binary/` 已按固定 wire、envelope、encode/decode、自描述 `BinaryValue` 声明与双向 JSON 转换拆分；永久 wire 为 `ZRPAYLD\0` + little-endian `u16` wire version + 显式 varint/little-endian/reject-trailing bincode body。Text/Binary 共用 `PayloadHeader`、`VersionedSchema` 与值域迁移链，旧 `UnsupportedFormat(Binary)` 生产分支、错误变体和拒绝测试已硬删除，无兼容 reader。`binary_contract` 与 `binary_malformed_contract` 先于生产实现落地，覆盖 current/legacy 往返、Text→Binary→Text canonical 等价、JSON 数值域、确定性/体积合同、schema/wire future、magic/截断/尾随字节、重复 key 与非有限浮点。Layout15 暴露的 11 个 E0364/E0365/E0603 已通过叶子与 value owner 同步收敛为最窄 `pub(in crate::serialization)` 修复，binary root 仅在 `cfg(test)` 下向父域开放测试 helper；已 [fixed 回传 Layout15](../../editor_layout/15/fixed-2026-07-14-binary-value-visibility-compilation.md)。首次整包复现只剩 `manifest_dependencies_stay_contract_only`，根因是 `bincode` 已从 dev 迁为生产序列化依赖而显式白名单未同步；守卫现将其列入合同/序列化依赖并清空 dev 白名单。Windows 受管 job `8be9b8f3a08f4b7f85da9115c65637ca` fresh clean 后执行 `cargo test -p zircon_runtime_interface --locked`，结果 263 passed / 0 failed，doc-tests 通过；新二进制测试复核 13/13，边界守卫 1/1。256-row 选择夹具为 binary 12094 bytes、canonical text 32554 bytes，二进制缩小 62.85%。M3.2 cook consumer、5k-entity 场景夹具和完整 M3 独立审查仍未完成。 |
| M3 | 3.1 Binary 永久 wire 安全收口 | `切片验收-interface 281/281-独立复审0/0/0-父M3待M3.2` | 2026-07-16 | 初次独立审查为 Critical/Important/Minor=`0/3/0`，三项 Important 均已按底层值域修复：① 递归 `BinaryValue` 改为非递归 flat `BinaryNode` stream，并在前缀/值域层固定 64 MiB body、16 MiB string/key、1,000,000 container entries、2,000,000 nodes、128 depth 上限；② `BinaryEnvelope` field order 与 `BinaryNode` variant order 明确写入 wire-v1 合同，新增 exact golden bytes 同时锁住 header、全部 node variant 与 bincode options，改变顺序必须 bump wire version；③ 删除不可达 Decimal 路径，只保留 JSON 可达的 `i64/u64/finite f64`，Text/Binary 共用的前置 Serde guard 对 `NaN/±∞` 返回 `WriteError::NonFiniteFloat`，不再让 `serde_json` 静默归一为 `null`。decode 先单独读取并校验 `PayloadHeader`，新增 future schema header + invalid value body 合同证明 payload 不会先被解码。首次 re-review 关闭旧三项但新增 writer 未镜像 64 MiB reader body cap 的 Important；RED job `f2b9ae3ae8e544538f54506f1e0ed6d6` / run `7ed714818064494b91bdfe67da0b8d1b` 先证明缺少 typed error，随后 writer 使用同一 bincode limit 并返回 `WriteError::BinaryPayloadTooLarge`，GREEN job `7e80283433204fa69c532680ec0a2036` / run `aba491a1a1c440edb4ef8c8ad5fe5d2f` 通过 focused contract。最终 Windows canonical reservation `f7a470093bfd4449ab1a4b1f841da444`、job `30d14024f27c4c26a32e7537c93b1bd9`、run `08c3a06c27d1470d9e5aa41d7066b29d` 执行 `cargo test -p zircon_runtime_interface --locked --offline -- --test-threads=1`，结果 library 278/278、integration 3/3、doc-tests 0/0，exit 0、live PIDs 空。最终独立复审 Critical/Important/Minor=`0/0/0`，exact24 hash=`70e0389e5fa650e5a16fb742c2a5f5dbbd25de171dc6dbfac2d1a68c7d2e8e4e`；`git diff --check`、scoped `rustfmt`、计划产出审计与 164 个 failure/fixed handoff 审计均通过。M3.1 切片验收，父 M3 仍等待 M3.2 cook consumer 与 5k-entity/matrix。 |
| M3 | 3.1 协调器受管提交 | `阻塞-workflow_topology_missing-未暂存未提交` | 2026-07-16 | `milestone prepare --session-id editor11-m3-1-binary-payload-r1-20260716 --milestone M3.1` 返回 `workflow_topology_missing: Plan has no zircon-workflow block or milestone headings`。根因归属 Plan11 计划定义：现有 `### M3 二进制格式与等价性` 不符合协调器只接受 `zircon-workflow` block 或 `## Milestone M3:` 的拓扑合同；父计划文件当前包含外部未提交改动且不在 exact24 租约，故本 Session 不越权修改、不普通 `git add/commit`、不伪造 run。该失败已写回 Editor11 对应子计划，待 Plan11 计划维护 owner 在保留现有内容的前提下补齐受保护 topology 后，重新 prepare/import review/managed validation/commit exact24。 |

## 架构判定

- Owner：`zircon_runtime_interface::serialization`，是 runtime/editor/Hub/headless 可共享的中性持久化合同，不把二进制实现放进 Editor UI 或 runtime scene owner。
- 基础能力：现有 `VersionedSchema + MigrationChain<Value>` 足够；新增 backend 只把相同值域编码成紧凑字节，不复制 schema 注册表或迁移路径。
- 拒绝方案：直接 `bincode<T>` 会要求保留旧 Rust DTO；在壳内保存 JSON 字节不具备紧凑产物价值；新增 Postcard 会重复工作区已锁定 bincode 的职责。
- 硬切换：Binary 此前从未形成可读 wire，因此本切片只有一个永久 v1 wire，不保留 `UnsupportedFormat` 或临时格式兼容入口。

## 测试阶段待办

- 完成 M3.2 CookAssets 消费点接线，并执行逐 schema Text↔Binary 与 cook consumer 门禁。
- 补齐 5k-entity 场景夹具的文本/二进制字节数与耗时；256-row 选择夹具已记录实际字节数。
- 完成 M3 整体独立审查后才允许把本记录状态提升为里程碑验收。
