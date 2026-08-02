# Plugins 01 AI standard SystemSet output record

Plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
Milestone: M1 / standard plugin SystemSet rollout
Status: implementation completed; managed focused test pending authorization
Date: 2026-08-01

## Scope Delivered

- AI runtime 公开唯一标准集合常量 `AI_MAIN_SYSTEM_SET = "ai.main"`。
- `AiRuntimePlugin` descriptor 与 `zircon_plugins/ai/plugin.toml` 同步声明 `ai.main`，保持代码描述符、生成清单输入和分发清单的单一合同。
- perception 与 behavior 两个 runtime scene system 都显式加入 `ai.main`，允许其他插件只依赖 set 级 before/after 约束，不耦合具体 system id。
- registration 测试覆盖精确常量、两个系统的集合成员关系及生成 manifest 的 `system_sets` parity。

## Fresh Evidence

- Rust 1.94.1、edition 2021 的 scoped `rustfmt --check`：通过。
- 四文件限定范围 `git diff --check`：通过，仅有工作树既有行尾转换提示。
- `python tools/audit_plugin_structure.py --json --repo-root E:\\Git\\ZirconEngine`：`manifest_schema_violations = 0`、`generated_manifest_header_violations = 0`、`runtime_plugin_descriptor_single_source_violation_count = 0`、`runtime_registration_builder_violation_count = 0`、`registration_compatibility_shim_sites = 0`。全局既有 `dist_abi_projection_violations = 37` 不归属本切片，本记录不把 broad audit 声明为全绿。
- 当前源码 SHA-256：`plugin.toml` = `9BBE9F639C155CD107CF9647ADF693DDE86C73A0450947FF954607F72DE80578`；`plugin.rs` = `C9A97F3CE5150C7BB16B6A3B76F5EA75F3E467DEAF7E9C76CEE85E06FB549EBB`；production registration = `ADB9556A6CA4077601B8BD36ECC86363939F14E1F07AA4F3532F359EC3235395`；registration tests = `6AB9B7C71CE961789A932BD8BD60C3153DE70D3B35616C1156F03B9AF48E5CBB`。
- 独立只读复核：Critical 0 / Important 0 / Minor 0；未运行 Cargo、未编辑源码。

## 状态和完成项目

| 项目 | 状态 | 证据 |
|---|---|---|
| `ai.main` 标准 SystemSet 常量 | completed | production 常量与精确字面量测试已落位。 |
| descriptor / TOML 投影 | completed | 两侧均声明唯一 `ai.main`。 |
| runtime system membership | completed | perception、behavior 共 2 个系统均 `.in_set(AI_MAIN_SYSTEM_SET)`。 |
| generated manifest parity | completed | 测试比较 generated/runtime module 的 `system_sets`。 |
| 静态格式、结构与独立评审 | completed | scoped checks 通过，最终 C0/I0/M0。 |
| 受管 focused Cargo test | pending authorization | 目标为 `zircon_plugin_ai_runtime` 的 `ai_runtime_systems_join_main_system_set`；现有外部 validation-copy `5945e3ef29d74bd69602adca02e243b5` 只可在显式受管授权后按 FIFO 消费，不重建、不重试、不清理。 |

## Remaining Scope

- 本记录只完成 Plugins01 标准 SystemSet rollout 的 AI 切片；不关闭 Plugins01、AI 总计划或全插件 rollout。
- 待协调器明确授权后，使用当前源码执行 Windows 受管精确测试并把实际 test target 结果回填本记录。
- Net 切片由其现有 owner 独立推进；Navigation、ZrVM 和中央结构守卫仍按 Plugins01 计划继续。
