# Plugins 01 ZrVM Language standard SystemSet output record

Plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
Milestone: M1 / standard plugin SystemSet rollout
Status: implementation completed; managed focused test pending authorization
Date: 2026-08-01

## Scope Delivered

- ZrVM Language runtime 公开并从 crate 根导出唯一标准集合常量 `ZR_VM_LANGUAGE_MAIN_SYSTEM_SET = "zr_vm_language.main"`。
- runtime descriptor 与生成的 `zircon_plugins/zr_vm_language/plugin.toml` 均声明该 main set。
- behavior bridge、FixedUpdate/Update/Last 三个 dispatcher 和 GC step 共 5 个 runtime scene system 全部加入 `zr_vm_language.main`。
- 保持 behavior bridge 的 `First`、各 dispatcher 的原阶段、GC 的 `Last` 及 `after(last dispatcher)` 约束不变。
- focused registration test 覆盖精确常量、runtime module manifest 投影、owner-filtered 精确 5 系统及完整集合成员关系。

## Fresh Evidence

- 测试先行静态 RED：测试合同已存在时，production descriptor 与 TOML 均尚无 `zr_vm_language.main`；随后只补齐合同所需实现。
- Rust 1.94.1、edition 2021 scoped `rustfmt --check`：通过。
- 四文件限定范围 `git diff --check`：通过，仅有工作树既有行尾转换提示。
- `python tools/audit_plugin_structure.py --json --repo-root E:\\Git\\ZirconEngine`：`manifest_schema_violations = 0`、`generated_manifest_header_violations = 0`、`runtime_plugin_descriptor_single_source_violation_count = 0`、`runtime_registration_builder_violation_count = 0`、`registration_compatibility_shim_sites = 0`。全局既有 `dist_abi_projection_violations = 37` 不归属本切片，本记录不声明 broad audit 全绿。
- 当前源码 SHA-256：`plugin.toml` = `8C66C0576E91A625E4D55D87BC4FEF876F82B759BCA71FEA2DB81FB8143B5579`；`lib.rs` = `B18034638DBA829FFCFB273FC121FB06D5431B05D931745D0B07E3593588EF34`；`plugin.rs` = `9545D5FC44109A23A456C67F34EC0BEEE7F55061125EEDE5A1C82E14D6520ACA`；registration tests = `AADC67C6EB747ECC5659EE92780BA8CB428A9D934510F0909C4433F551206384`。
- 独立只读复核：Critical 0 / Important 0 / Minor 0；未运行 Cargo、未编辑源码。

## 状态和完成项目

| 项目 | 状态 | 证据 |
|---|---|---|
| `zr_vm_language.main` 标准 SystemSet | completed | production 常量、crate 导出与精确字面量测试已落位。 |
| descriptor / TOML 投影 | completed | 两侧只声明 `zr_vm_language.main`。 |
| 5 个 runtime system membership | completed | behavior bridge、3 dispatcher、GC step 全部加入唯一 main set。 |
| 阶段与 GC 后序约束保留 | completed | 原阶段选择及 `after(last dispatcher)` 保持不变并有既有测试。 |
| 静态格式、结构与独立评审 | completed | scoped checks 通过，最终 C0/I0/M0。 |
| 受管 focused Cargo test | pending authorization | 目标为 `zircon_plugin_zr_vm_language_runtime` 的 `zr_vm_runtime_systems_join_main_system_set`；现有外部 validation-copy `5945e3ef29d74bd69602adca02e243b5` 只可在显式受管授权后按 FIFO 消费，不重建、不重试、不清理。 |

## Remaining Scope

- 本记录只完成 Plugins01 标准 SystemSet rollout 的 ZrVM Language 切片，不关闭 Plugins01 或 ZrVM 总计划。
- 待协调器明确授权后执行 Windows 受管精确测试并回填实际 test target 结果。
- Net owner 对账、中央结构守卫及 Plugins01 其余 failure gate 继续推进。
