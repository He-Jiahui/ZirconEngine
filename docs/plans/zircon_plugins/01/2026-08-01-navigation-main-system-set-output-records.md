# Plugins 01 Navigation standard SystemSet output record

Plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
Milestone: M1 / standard plugin SystemSet rollout
Status: implementation completed; managed focused test pending authorization
Date: 2026-08-01

## Scope Delivered

- Navigation runtime 公开并从 crate 根导出唯一标准集合常量 `NAVIGATION_MAIN_SYSTEM_SET = "navigation.main"`。
- runtime descriptor 与生成的 `zircon_plugins/navigation/plugin.toml` 均声明 `navigation.main`。
- `navigation.agent_tick` 显式加入 `navigation.main`，同时保持既有 `after("ai.behavior_tick")` 约束不变。
- focused registration test 覆盖精确常量、runtime module manifest 投影、非空 owner-filtered runtime system 集合及完整集合成员关系。

## Fresh Evidence

- 测试先行静态 RED：测试合同已存在时，production descriptor 与 TOML 均尚无 `navigation.main`；随后仅补齐合同所需实现。
- Rust 1.94.1、edition 2021 scoped `rustfmt --check`：通过。
- 四文件限定范围 `git diff --check`：通过，仅有工作树既有行尾转换提示。
- `python tools/audit_plugin_structure.py --json --repo-root E:\\Git\\ZirconEngine`：manifest schema、generated manifest header、runtime descriptor single-source、runtime registration builder 与 compatibility shim 相关计数均为 0。全局既有 `dist_abi_projection_violations = 37` 不归属本切片，本记录不声明 broad audit 全绿。
- 当前源码 SHA-256：`plugin.toml` = `C89BD3768462745F51F1D9951297148D3B7905AE53464E106BFF806B786EB597`；`lib.rs` = `DD9ACC1D235305673F67AD9E66DC2B03686F8ACB3644A6CF341C12B8BC819257`；`plugin.rs` = `D7B9C0EA4E17DAF0EEBDB25FF83D465B2D755835D0B3BC7FB119BAD2F1D8C92D`；registration tests = `A3E1FB8C69CF9B607400438B13DA77BEA3120B9630D5EB3091DCEA78EF2DD6DC`。
- 独立只读复核：Critical 0 / Important 0 / Minor 0；未运行 Cargo、未编辑源码。

## 状态和完成项目

| 项目 | 状态 | 证据 |
|---|---|---|
| `navigation.main` 标准 SystemSet | completed | production 常量、crate 导出和精确字面量测试已落位。 |
| descriptor / TOML 投影 | completed | 两侧只声明 `navigation.main`。 |
| runtime system membership | completed | owner-filtered 非空系统集合全部等于唯一 main set。 |
| AI 后序约束保留 | completed | production 与独立测试均保留 `after("ai.behavior_tick")`。 |
| 静态格式、结构与独立评审 | completed | scoped checks 通过，最终 C0/I0/M0。 |
| 受管 focused Cargo test | pending authorization | 目标为 `zircon_plugin_navigation_runtime` 的 `navigation_runtime_systems_join_main_system_set`；现有外部 validation-copy `5945e3ef29d74bd69602adca02e243b5` 只可在显式受管授权后按 FIFO 消费，不重建、不重试、不清理。 |

## Remaining Scope

- 本记录只完成 Plugins01 标准 SystemSet rollout 的 Navigation 切片，不关闭 Plugins01 或 Navigation 总计划。
- 待协调器明确授权后执行 Windows 受管精确测试并回填实际 test target 结果。
- ZrVM 与中央结构守卫继续按 Plugins01 计划推进。
