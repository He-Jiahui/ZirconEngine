---
handoff_kind: failure
status: open
created_at: 2026-07-15
summary_slug: virtual-geometry-debug-snapshot-project-toml-consumer-drift
origin_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_editor/editor/03
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
related_code:
  - zircon_runtime/tests/virtual_geometry_debug_snapshot_contract.rs
  - zircon_runtime/src/asset/assets/model
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/project
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild
---

# Runtime 04: Virtual Geometry debug snapshot 仍调用退役的 project TOML API

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `IMPLEMENTED / 受管门被Frameworks05外部编译失败阻断` | 2026-07-15 | 10 个旧 `to_toml_string()` 已清零，5 个临时项目 fixture 通过真实 `ProjectManager` 向 10 个 `to_project_toml_string` 调用提供 `persist_runtime_reference`；`rustfmt --check`、scoped `git diff --check` 通过，独立复核 P0/P1/P2 均为 0。Windows 受管 full package job `8c2266701b6f49079719471e932f6fd7` 与 focused integration job `b4bff1bc67fe4e7d962b54a91fcc53f6` 均未再报告本 failure 的 E0599，但分别在 Frameworks05 Text hard-cut consumer 层以 75/44 个 E0308/E0603 提前停止，未到达 VG test binary；已移交 [Frameworks05 open failure](../../frameworks/05/failure-2026-07-15-text-hard-cut-runtime-consumer-type-drift.md)，本 lifecycle 继续保持 open，不声明通过。 |
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

Open state: `待修复`; no pass is claimed.
