---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
resolved_at: 2026-07-15
summary_slug: external-source-cubemap-contract-api-drift
origin_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
fixing_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
origin_child_dir: docs/plans/zircon_editor/editor/03
fixing_child_dir: docs/plans/zircon_runtime/shader/06
related_code:
  - zircon_runtime/tests/runtime_environment_external_cubemap_import_staging_contract.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap.rs
tests:
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild
  - runtime_environment_external_cubemap_import_staging_contract-d6b4a7673c12dcb4.exe --nocapture --test-threads=1
---


# Shader 06：External source cubemap 测试契约 API 漂移

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 来源执行切片：Editor03 operation factory/runtime wiring 的 Runtime 全量回归门
- 修复责任计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 交接原因：最低失败位于 Shader06 明确拥有的 source cubemap mip pyramid 与 source/PMREM 分离契约测试，不属于 Editor03 command/transaction 或 runtime operation 生命周期。

## 失败现象与复现证据

Windows 受管命令 `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild` 在生成 `runtime_environment_external_cubemap_import_staging_contract` 测试目标时返回 E0599 三处：

- `zircon_runtime/tests/runtime_environment_external_cubemap_import_staging_contract.rs:32` 仍调用已删除的 `SourceCubemapMipChain::texels()`；
- 同文件 `:221` 仍调用已删除的 `face_size()` 与 `mip_count()`；
- 当前生产 owner 已硬切为显式 `source_*` 与 `pmrem_*` 访问器，编译器仅提示 `pmrem_texels`、`pmrem_face_size`、`pmrem_mip_count`，但测试的 source face/mip offset 语义必须由 Shader06 owner按断言角色选择正确访问器。

本轮 `zircon_runtime` 生产构建已先通过；测试编译在执行任何 Runtime 测试前终止，因此 Editor03 全量 Runtime 行为门尚未完成。

## 最低共享层根因

`SourceCubemapMipChain` 已从含混的 `texels/face_size/mip_count` 硬切为角色明确的 `source_*` 与 `pmrem_*` API，但外部 cubemap 导入/暂存契约测试仍保留旧调用。最低失配是 Shader06 生产数据模型与其外部容器契约测试之间的 API 漂移。

## 架构修复验收

- 按每条断言的 source/PMREM 角色改用当前显式访问器：比较 source 与卷积结果时分别取 source/PMREM texels，计算 source face/mip offset 时使用 source shape。
- 不恢复 `texels/face_size/mip_count` 兼容 alias，不让测试通过含混入口重新混淆 source 与 PMREM。
- 先让 `runtime_environment_external_cubemap_import_staging_contract` 编译并通过，再重跑完整 `zircon_runtime` 受管测试门并回传 Editor03。

## 禁止临时方案

- 禁止添加旧方法 alias、兼容 shim、静默 fallback、test-only bypass 或调用点特例。
- 禁止删除、跳过或弱化“外部 source mip 不能直接充当 Zircon PMREM”的断言。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| EC-M1/EC-M2 | External source cubemap import/staging 契约测试 API 对齐 | `未通过-待-Shader06-owner-修复` | 2026-07-15 | Editor03 Runtime 全量受管门在 `runtime_environment_external_cubemap_import_staging_contract.rs:32,221` 报 E0599 x3；生产 `zircon_runtime` build 已通过，但测试二进制未生成。 |
| EC-M1/EC-M2 | 显式 source/PMREM 访问器修复与原始契约回归 | `通过-回传` | 2026-07-15 | 修正后受管 `zircon_runtime` package 编译已生成当前 `runtime_environment_external_cubemap_import_staging_contract-d6b4a7673c12dcb4.exe`；直接执行 3/3 通过，0 failed。Frameworks03 外部阻断已由 managed job `7d0c3e3ddfa148ce98e2350d8a3cc939` exit 0 修复；随后全包门重跑已越过 external cubemap 与 Frameworks03 目标，仅在后续外部并行目标终止，不影响本交接回传。 |

## 修复结果与回传

- 根因：External cubemap integration contract retained removed ambiguous SourceCubemapMipChain accessors after the source/PMREM hard cut.
- 架构修复：Use pmrem_texels versus source_texels for role comparison and source_face_size/source_mip_count for source layout offsets; no alias, shim, fallback, or production compatibility surface added.
- 验证：Current managed integration binary passed 3/3; Frameworks03 managed prerequisite job 7d0c3e3ddfa148ce98e2350d8a3cc939 exited 0; package validation reran beyond both repaired targets before a later foreign target failed.
- 回传：Editor03 may resume the external source cubemap contract gate; unrelated later package failures remain with their owning plans.
