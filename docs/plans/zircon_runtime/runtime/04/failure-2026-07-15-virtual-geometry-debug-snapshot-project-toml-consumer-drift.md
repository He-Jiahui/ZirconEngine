---
handoff_kind: failure
status: open
created_at: 2026-07-15
summary_slug: virtual-geometry-debug-snapshot-project-toml-consumer-drift
origin_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_editor/editor/03
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/tests/virtual_geometry_debug_snapshot_contract.rs
  - zircon_runtime/src/asset/assets/model
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/project
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild -TestTarget virtual_geometry_debug_snapshot_contract -TestThreads 1 -VerboseOutput
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild -VerboseOutput
---

# Runtime 04: Virtual Geometry debug snapshot 仍调用退役的 project TOML API

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `IMPLEMENTED / 受管门被Frameworks05外部编译失败阻断` | 2026-07-15 | 10 个旧 `to_toml_string()` 已清零，5 个临时项目 fixture 通过真实 `ProjectManager` 向 10 个 `to_project_toml_string` 调用提供 `persist_runtime_reference`；`rustfmt --check`、scoped `git diff --check` 通过，独立复核 P0/P1/P2 均为 0。Windows 受管 full package job `8c2266701b6f49079719471e932f6fd7` 与 focused integration job `b4bff1bc67fe4e7d962b54a91fcc53f6` 均未再报告本 failure 的 E0599，但分别在 Frameworks05 Text hard-cut consumer 层以 75/44 个 E0308/E0603 提前停止，未到达 VG test binary；当时移交的 Frameworks05 failure [现已回传 fixed](fixed-2026-07-15-text-hard-cut-runtime-consumer-type-drift.md)，本 lifecycle 继续保持 open，不声明通过。 |
| `IMPLEMENTED / 已越过Frameworks05编译 / 受Plugins13 fixture漂移阻断` | 2026-07-15 | Frameworks05 fresh library job `9af67024670242beaac743a5c7dde856` 退出 0；Runtime04 随后以同一 Windows 兼容池运行 focused job `1e7cdd7825024a08b236b2edd07c67b9`，测试目标完成编译并实际进入 7 个 VG tests，原 10 个 E0599 未复发。结果为 `0 passed / 3 failed / 4 ignored`，三个运行用例均在业务断言前被根级 Virtual Geometry support descriptor 缺少 AsyncCompute workload 阻断；该独立问题已移交 [Plugins13 open failure](../../../zircon_plugins/13/failure-2026-07-15-virtual-geometry-runtime-support-compute-workload-drift.md)。本 lifecycle 仍保持 open，等待 Plugins13 修复后重跑原断言。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 来源执行切片：M3.2 operation factory/runtime V2 完整 Runtime 受管回归门
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：失败位于 Runtime04 拥有的资产序列化消费边界；Editor03 与 Runtime10 的 operation ABI 已完成编译和专项验证，不应恢复退役资产 API 或在编辑器事务层绕过项目解析器。

## 失败现象与复现证据

2026-07-15 Windows 受管命令：

```powershell
./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild
```

已完成 `zircon_runtime` 主库测试目标及多项 integration test 的编译，随后仅在 `zircon_runtime/tests/virtual_geometry_debug_snapshot_contract.rs` 停止。编译器报告 10 个 `E0599`：`ModelAsset` 与 `SceneAsset` 已无 `to_toml_string()`，旧调用位于 526/538、917/929、1347/1359、1507/1519、1621/1633 行；当前接口要求经项目 resolver 调用 `to_project_toml_string(resolver)`。

该失败发生在 operation ABI、Navigation runtime/editor 和 `zircon_app` 受管门均通过之后，未出现 Editor03、Runtime10 或 Shader04 编译错误。历史广门记录也已把同一 integration test 的退役资产 API 调用归为 Runtime04 外部项。

## 最低共享层根因

项目资产序列化已硬切为 resolver-aware 合同，但 Virtual Geometry debug snapshot integration fixture 仍按无项目上下文的旧 API 写出 `ModelAsset`/`SceneAsset`。这是 Runtime04 测试消费者未随项目序列化边界迁移，不是 Virtual Geometry 剔除算法或 Editor03 事务执行错误。

## 架构修复验收

- fixture 使用真实项目 resolver 经 `to_project_toml_string(resolver)` 写出模型与场景，引用解析规则与生产项目资产路径一致。
- `virtual_geometry_debug_snapshot_contract` 测试目标完成编译并通过其原有断言；不得删除、忽略或弱化 Virtual Geometry 产品合同。
- 重新执行受管 `zircon_runtime` package gate，确认 10 个 `E0599` 清零并允许 Editor03/Runtime10 上行回归继续。

## 禁止临时方案

- 不得恢复 `to_toml_string()` 别名、兼容 trait、旧路径 re-export、隐式默认 resolver 或测试专用 bypass。
- 不得把 resolver-aware 项目引用降级为无上下文字符串，或用手写 TOML 绕过资产序列化 owner。
- 不得修改 Shader04 文件，也不得削弱完整 Runtime 回归门来隐藏失败。

## 修复结果与回传

Open state: `源码修复已落地 / Plugins13 上行 fixture 与受管动态验收待完成`。2026-09-24 当前 `zircon_runtime/tests/virtual_geometry_debug_snapshot_contract.rs`（SHA-256 `d97bd0b8143d4e315360e0723b29d1a799995197e41d1a60af92c33cc3b5d9d0`）没有旧 `to_toml_string()` 调用：五处测试 fixture 均通过真实 `ProjectManager::open`，十处 model/scene 写出均通过 `to_project_toml_string(|reference| project.persist_runtime_reference(reference))`。ModelAsset 与 SceneAsset 现行接口均要求该 resolver，因此没有恢复旧 API 或添加序列化旁路。上面的 2026-07-15 focused job 已完成编译且实际执行 3 个非 ignored 用例，但 `0 passed / 3 failed / 4 ignored`，失败属于 [Plugins13 support descriptor drift](../../../zircon_plugins/13/failure-2026-07-15-virtual-geometry-runtime-support-compute-workload-drift.md)；其当前 `[64,1,1]` support 修正仍待受管 Cargo 验收，且上行 graphics fixture 的 `[1,1,1]` 分歧尚未由 owner 收束。不能复用旧 job 为本 failure 的产品通过证据。

本次 `failure-roll-01a084c8-runtime04-vg-project-toml-r2` 仅在协调器确认原 failure 文件 SHA-256 `3ff7f0909aa7ce120cc0ded145cddc490e0b5e87637a58b7f44ee99b94e551b2` 与已归档 owner 归属一致、转移指纹 `248190fe57b88982288e93032528abe6afd1c5ee06e49e5d54b89bd15943fbe7` 无冲突后，纠正记录状态与 focused/full 验收命令；未编辑或吸收现行 VG 测试源码。外部 `E:\Git\zr_vm` 工作树仍有非本会话改动，受管 Windows `--locked` 的 focused 原始断言、Runtime package 全量、关联上游与最终独立 C/I/M 审查均未通过；保持 `status: open`，不执行 return、closeout 或通知。

2026-09-24 独立只读复审对测试源码和本记录修正后哈希 `d77b0bdd705c74dffcfb099a501b54da721fc9f6387dbec35a7c871879357f92` 报告 `Critical=0 / Important=0 / Moderate=0`。审查发现的唯一旧链路指向已删除的 Frameworks05 open 文件，已改为上表实际存在的 fixed 回传记录并保留原始 75/44 编译诊断；审查没有执行 Cargo 或声明本 failure 的最终验收通过。Plugins13 另一路上行 synthetic workload 分歧由 [Render01 新 handoff](../../render/01/failure-2026-09-24-virtual-geometry-compiled-fixture-dispatch-extent-drift.md) 继续负责，不归入本 Session 的源码快照。

## 2026-09-26 rolling successor source handoff

- Successor Session `failure-roll-01a084c8-runtime04-vg-project-toml-r3` owns
  only this failure document under audited ownership-transfer fingerprint
  `6673c987cb2ed328432fe4e79da1dd0ecb76305e33e4f8f01770bb72087f0f44`.
  The pre-review boundary is snapshot `3911`, with document SHA
  `2148919b2afb7b93ca6254d6222e0a6e96d5e92863d034ac669eb47e61eb69d0`.
  Runtime04 asset-project sources and the Plugins13/graphics paths are dirty
  foreign work; this successor does not claim, edit, or absorb them.
- The current direct consumer fixture remains byte-identical to the recorded
  source evidence: `zircon_runtime/tests/virtual_geometry_debug_snapshot_contract.rs`
  SHA-256 `d97bd0b8143d4e315360e0723b29d1a799995197e41d1a60af92c33cc3b5d9d0`.
  Its static contract still contains no `to_toml_string()` call, uses the real
  `ProjectManager::open` resolver, and routes all ten model/scene writes through
  `to_project_toml_string(|reference| project.persist_runtime_reference(reference))`.
- There is no newer managed static ticket to reuse for this document-only slice.
  The existing focused Windows run remains diagnostic only (`0 passed / 3 failed /
  4 ignored`) because Plugins13's `[64,1,1]` support descriptor failed before
  the business assertions; the full Runtime package, Plugins13 support repair,
  Render01 extent handoff, and external `E:\Git\zr_vm` cleanliness remain
  separate gates. The 2026-09-24 independent review's
  **Critical=0 / Important=0 / Moderate=0** is retained as static document/source
  review evidence, not dynamic acceptance.
- Focused `virtual_geometry_debug_snapshot_contract`, Runtime04 package Cargo,
  upstream Plugins13/Render01 fixture consistency, canonical `fixed-*` return,
  closeout, and WeCom receipt remain pending. Failure status remains `open`.
- Final r3 review receipt: reviewer `review_editor03_gizmo_private` verified
  snapshot `3912` and document SHA
  `e581c596b657c66c47ecdb0875f0c56c4bdb3a1d34cf5729c3b5c14d2f175a8c`,
  including the no-managed-static-ticket boundary and foreign-source separation;
  **Critical=0 / Important=0 / Moderate=0**. This receipt does not promote the
  diagnostic Cargo run or any deferred upstream/product gate.
