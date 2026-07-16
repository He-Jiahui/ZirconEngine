# Frameworks 06 G7 Owner Hard Cut Batch 3

> 本文件记录 `06-development-conventions-and-guardrails.md` 的 G7 docs 勾稽修正批次；父计划仍持有完整里程碑定义。

| 切片 | 完成项目 | 状态锚 | 日期 | 当前状态与证据 |
|---|---|---|---|---|
| M1 / G7 | Project contract 与 derived reflection owner hard cut, batch 3 | `frameworks_06_g7_owner_batch3_accepted_global_red` | 2026-07-16 | **batch accepted / global G7 RED**。将 3 份原本 clean 的架构文档中 33 个重复 machine-path violation 从退役 `plugin/export_profile.rs`、`plugin/project_plugin_manifest/*` 与 `scene/reflect/fixed/*` owner 硬切到唯一现存 `core/framework/project/{export_profile,project_plugin_manifest}/*`、`scene/reflect/builtin_reflection/registration.rs`、`scene/components/scene.rs` 及其 reflection child owners；同步把 Reflection 旧“manual fixed adapter / future derive / compatibility surface”叙述硬切到当前 `ZrReflect` derive、`RuntimeTypeRegistration` 构造、builtin registration 安装、generic World bridge 与单一 `WorldReflection` DTO router，不保留旧 adapter tree、alias 或第二注册面。fresh `python tools/check_conventions.py --only docs --json` 从 batch2 后 308 missing / 62 affected docs 收敛到 275 / 59；exact scope violation 为 0，14 个新增唯一源码目标全部存在，scoped `git diff --check` 通过。独立 review 首轮为 0 Critical / 1 Important / 0 Low；修正 derived registration 安装链与旧 fixed 术语后，最终 fresh re-review 为 0/0/0。G7 仍全局 RED，因此不声明 M1 或 Plan06 完成。 |

## 精确文档范围

- `docs/engine-architecture/runtime-editor-pluginized-export.md`
- `docs/runtime-plugins/bevy-parity-matrix.md`
- `docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md`

## 验收边界

- 本批为 docs-only G7 support correction，不运行 Cargo，也不冒充分支 CI。
- fresh 独立 review 已达 0 Critical / 0 Important；通过 coordinator maintenance finalize 提交精确 4 文件 manifest。
- 后续批次继续从剩余 275 missing 中选择 clean owner；foreign dirty 文档保持原会话归属。
