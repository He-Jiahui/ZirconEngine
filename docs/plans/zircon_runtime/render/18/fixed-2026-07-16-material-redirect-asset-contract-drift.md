---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: material-redirect-asset-contract-drift
origin_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
fixing_plan: docs/plans/zircon_runtime/shader/03-module-imports-and-cross-references.md
origin_child_dir: docs/plans/zircon_runtime/render/18
fixing_child_dir: docs/plans/zircon_runtime/shader/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/tests/material_shader_redirect_dependency_contract.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime_interface/src/resource/asset_reference.rs
tests:
  - validate-matrix.ps1 -Package zircon_runtime -SkipBuild -VerboseOutput
  - cargo test -p zircon_runtime --test material_shader_redirect_dependency_contract --no-default-features --features target-server --locked
resolved_at: 2026-07-16
---


# SH03: material redirect asset contract drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 来源执行者：`render18-af-m2-rebase-20260715`
- 来源执行切片：AF-M3 `zircon_runtime` broad validation。
- 修复责任计划：`docs/plans/zircon_runtime/shader/03-module-imports-and-cross-references.md`
- 交接原因：失败文件是 SH03 建立的 shader-import redirect 材质契约；Render18 只发现它，不能为通过上层验证而吸收或削弱资产/Shader03 的契约修复。

## 失败现象与复现证据

`validate-matrix.ps1 -Package zircon_runtime -SkipBuild -VerboseOutput` 在编译
`zircon_runtime/tests/material_shader_redirect_dependency_contract.rs` 时停止，Cargo 未进入
Render18 的测试主体：

- `MaterialAsset::from_toml_str` 不存在（E0599），第 18 行的内存 TOML 构造已不符合当前 MaterialAsset API。
- 两个 `AssetReference` 比较点不再满足当前类型契约（E0599/E0308）：第 35 行 closure 比较，以及第 44、54 行的 match guard 比较。

该测试本身是 SH03 于 2026-07-04 为 redirect include material readiness 加入的验收契约；其计划记录曾以相同 test target 通过 2/2。当前错误说明资产构造/引用身份 API 已演进，而此测试仍保留被淘汰的调用形状。

## 最低共享层根因

Shader redirect 的测试把 MaterialAsset TOML 解析和 AssetReference 的直接相等性当作稳定公共契约。当前资产层已经移除或收紧这些表面 API，但没有将 SH03 的 redirect-readiness 验收迁移到唯一的 canonical asset-import path 与 canonical reference-identity comparison。问题位于 SH03 与资产契约交界，而不是 Render18 的高级光照实现。

## 架构修复验收

- 确认 MaterialAsset 的唯一生产构造路径；把该测试迁移到该 importer/document path，或在资产契约中恢复真正需要的公共 API，二者只能保留一个事实来源。
- 确认 AssetReference 的规范身份比较方式；测试、redirect readiness 与诊断使用同一方式，不添加 test-only `PartialEq`、别名或按 locator 的临时比较。
- `cargo test -p zircon_runtime --test material_shader_redirect_dependency_contract --no-default-features --features target-server --locked` 必须恢复 2/2。
- 重新执行来源 `validate-matrix.ps1 -Package zircon_runtime -SkipBuild -VerboseOutput`；只有其越过该编译边界，Render18 才能继续判定自身 gate。

## 禁止临时方案

- 不得删除、忽略或弱化 redirect 缺失依赖的两条断言。
- 不得为测试恢复已废弃的 `from_toml_str` shim，或为 AssetReference 添加仅服务测试的比较实现。
- 不得在 Render18 代码中复制资产构造或引用比较规则。

## 修复结果与回传

- 根因：The SH03 contract test retained retired MaterialAsset TOML construction and direct AssetReference equality; its project fixture also bypassed the canonical compound shader persisted-reference writer.
- 架构修复：The test now builds MaterialAsset through ZMaterialDocument, compares dependency identity by AssetUuid, scans the compound shader package before serializing via ProjectManager::persist_runtime_reference, and then imports the material.
- 验证：Managed Windows job b2dbd653f82a4cfe92411053bd00aea2 ran cargo test -p zircon_runtime --test material_shader_redirect_dependency_contract --no-default-features --features target-server --locked: 2 passed, 0 failed, exit 0; rustfmt and scoped diff checks passed; fresh independent review Critical 0 Important 0 Minor 0.
- 回传：Shader03 redirect material contract now exercises the canonical document, compound package importer, persisted .zmeta reference writer, and AssetUuid identity path; Render18 may resume its dependent gate.
