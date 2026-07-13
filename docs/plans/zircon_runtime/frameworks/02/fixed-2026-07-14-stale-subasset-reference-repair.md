---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
resolved_at: 2026-07-14
summary_slug: stale-subasset-reference-repair
origin_plan: docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/02
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
related_code:
  - zircon_runtime/src/asset/reference_resolver.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/ingest/import_model.rs
tests:
  - E:/cargo-targets/zircon-engine/pool/0ab59ce30aa63b5c52717a92c1e2e1341f595b8959f221b5793a88271a9c4a4c/debug/deps/zircon_runtime-f8774ee8510e12dc.exe asset::reference_resolver::tests::resolution_reports_guid_path_repair_dangling_and_conflict_states --exact --nocapture --test-threads=1
  - E:/cargo-targets/zircon-engine/pool/0ab59ce30aa63b5c52717a92c1e2e1341f595b8959f221b5793a88271a9c4a4c/debug/deps/zircon_runtime-f8774ee8510e12dc.exe asset::importer::ingest::import_model::tests::importer_outcome_exposes_complete_guid_repair --exact --nocapture --test-threads=1
---


# Runtime 04：旧 subasset 标签阻断 GUID 修复

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / stale subasset repair` | 2026-07-13 | base asset 存在但 stale `#Mesh0` label 不存在时，resolver 错误进入 `Dangling`；需要先按 base locator 查 registry，再形成 GUID/subasset repair。 |
| `OPEN / current-source compile regression` | 2026-07-14 | Editor07 current-source exact 在进入 Editor 前被 `reference_resolver.rs:123,134` 的两项 E0308 挡住：`AssetUri::parse` 当前要求 `&str`，实现直接传入 `format!(...)` 的 `String`。日志 `.codex/tmp/editor07-focused-document-current-exact-20260714.log`。该错误位于本 failure 已登记的同一最低 owner，不创建重复 handoff；修复后须先恢复 Runtime 编译，再执行原 stale-subasset 行为验收。 |
| `FIXED / current-source lower and upward repair gates passed` | 2026-07-14 | 当前 Runtime lib-test 成功编译（7,959 tests inventory）。resolver 原测试含新增 stale-subasset 负例为 1/1；原 importer 复现为 1/1；所属两个过滤组各 1/1。完整 `asset::` 为 676 passed / 37 failed，其中不再包含本交接两项；剩余失败属于迁移命令、链接安全、缓存/观察器和共享 graphics/UI 夹具，不能回推到本 resolver 修复。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md`
- 来源执行切片：Frameworks 02 M3 Windows Runtime lib 完整测试门
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：最低共享故障位于 Runtime 04 所有的 project asset reference resolver；Frameworks 02 的 module/provider 聚焦回归已通过，不能在模块装配层绕过资产解析错误。

## 失败现象与复现证据

当前源码 Windows Runtime lib-test binary 已成功生成（7,863 tests inventory）。完整库测试在 Asset/Project 测试簇出现多项失败；最早可独立复现的失败为：

- `asset::importer::ingest::import_model::tests::importer_outcome_exposes_complete_guid_repair`：0/1，exit 101。
- 实际错误：`ReferenceResolutionError::Dangling { guid: c211..., path: "assets/models/hero.glb" }`。
- 该 fixture 的 base asset 文件与 registry row 均存在，stale reference 仅额外带 `sub = "Mesh0"`，测试期望以 path hint 找到 base asset、修复 GUID，并移除无效 subasset 标签。
- 完整门日志：`.codex/tmp/frameworks02-runtime-lib-full-20260713.log`；在 620/7863 前已出现多项 Runtime 04 Asset/Project 失败，因此来源会话停止后续执行且不声明 broad Runtime green。

## 最低共享层根因

下层普通 reference resolver 用例 `resolution_reports_guid_path_repair_dangling_and_conflict_states` 为 1/1 passed，说明 root mapping、safe path 与 base GUID/path 修复正常。失败收束到 `reference_resolver::entry_by_hint(...)`：它把 stale reference 的 `#Mesh0` 先拼入 registry locator，再执行 `entry_by_path`。registry 只拥有 base asset `res://models/hero.glb`，因此 lookup 返回空，解析在生成 `ReferenceRepairKind::Guid`/subasset repair 之前错误地落入 `Dangling`。

## 架构修复验收

- 在 reference resolver 最低层增加“base path 存在但 stale subasset label 不存在”的负例，明确 base asset lookup 与 subasset 修复的顺序。
- `importer_outcome_exposes_complete_guid_repair` 原样通过，并保留 `repair.stale.sub() == Some("Mesh0")`、`repair.resolved.sub() == None`。
- 重新执行 `asset::reference_resolver`、`asset::importer::ingest::import_model`，再向上执行 Frameworks 02 的 Runtime lib 完整门或至少完整 `asset::` filter。

## 禁止临时方案

- 禁止在 model importer 捕获 `Dangling` 后自行重试、删除 subasset 或伪造 repair。
- 禁止给 registry 添加别名 locator、兼容 entry、重复 base/subasset truth 或 test-only bypass。
- 禁止削弱 GUID/subasset repair 断言，或让模块/provider 测试跳过 Asset/Project 失败。

## 修复结果与回传

- 根因：entry_by_hint queried stale labeled locator before authoritative base locator, so a valid base asset was misclassified as dangling before GUID/subasset repair
- 架构修复：resolve and query the base AssetUri first, prefer an exact labeled entry when present, otherwise fall back to the same base entry and let repair_between produce the single authoritative repair
- 验证：current Runtime lib-test compile exit 0; resolver exact 1/1; importer original reproduction 1/1; resolver/import_model filters each 1/1; asset filter 676 passed/37 unrelated failed with both handoff tests green
- 回传：stale subasset labels now repair through the base asset without importer retry or registry aliases; Frameworks02 may resume its Runtime gate
