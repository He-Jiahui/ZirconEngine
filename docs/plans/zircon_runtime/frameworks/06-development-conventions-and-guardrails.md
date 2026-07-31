---
related_code:
  - .github/workflows/ci.yml
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate.rs
  - tools/plugin_structure_audits/capability.py
  - tools/check_conventions.py
  - tools/check-conventions.ps1
  - tools/tests/test_check_conventions.py
  - tools/tests/test_frameworks_06_ci_toolchain_contract.py
  - docs/cli-and-tooling/check-conventions.md
  - Cargo.toml
plan_sources:
  - docs/plans/zircon_runtime/frameworks/index.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/engine-architecture/workspace-root-rules-and-hard-cutover.md
  - docs/engine-architecture/large-file-ownership-m1.md
  - docs/engine-architecture/hard-cutover-migration-smells-m1.md
  - docs/engine-architecture/non-network-server-naming-m1.md
  - docs/engine-architecture/generated-code-boundary.md
last_refined: 2026-07-31
status: in_progress
---

# 06 · 开发规范总纲与守卫机制

## 1. 目标

引擎开发需要系统性纪律，而纪律只有变成机器可执行的守卫才可持续。本计划做两件事：

1. **规范单源化**：把散落在 `engine-code-structure-convention.md`、engine-architecture 各 M1 文档、`.claude/.codex` 技能、审计脚本注释里的规则，收敛为一份分层的《Zircon 开发规范总纲》，每条规则标注：级别（MUST/SHOULD）、适用范围、守卫方式、豁免流程。
2. **守卫机制化**：每条 MUST 规则对应一个自动守卫（编译器 / 守卫测试 / 审计脚本 / CI 步骤），没有守卫的 MUST 视为未完成。

## 2. 现状与差距

- 规则总纲与 tracked CLI 文档已经提供人类/AI 共用入口，用户优先的 structure/review authorities 也已同步；剩余差距是历史细节文档和 G7 路径仍持续漂移，不能把总纲落地误报为全仓收敛。
- 统一 `tools/check_conventions.py` 已聚合 Frameworks05 layering、Runtime structure convention、Rust 1.94.1 `cargo fmt --all --check`、Runtime Interface/App scoped `clippy --no-deps -D warnings` 与 docs/G7 审计，并由 CI 安装 fmt/clippy 后调用；feature/profile matrices 由计划 03 的 CI job 持有。未完成项是 workspace/Runtime 全量 clippy、计划 04 Rust `cargo-zircon` plugin checker、`deny.toml`/`cargo deny check`、真实分支全矩阵 acceptance，以及仍为 RED 的全库 G7。
- 错误处理、日志、panic 策略、unsafe 审查与公共 API rustdoc 已分别由权威总纲 `GEN-Q1..Q7`、`GEN-D4` 成文；当前缺口是这些规则仍部分依赖评审，自动化 G3/专项守卫及豁免统计尚未完整落地。

## 3. 规范总纲结构（权威文档：docs/plans/zircon_runtime/frameworks/development-conventions.md，2026-07-02 已落稿）

| 章 | 内容 | 主要来源（收编不重写） |
|----|------|----------------------|
| C1 结构 | 三包边界、core 脊柱、内部 crate 分层与依赖方向（计划 01）、根文件薄化、1000 行门槛、docs 镜像 | workspace-root-rules、large-file-ownership、structure-convention |
| C2 迁移 | 硬切换、禁 legacy/compat/shim/迁移 bridge、命名禁区（非网络 server）、生成代码边界 | hard-cutover-smells、non-network-server-naming、generated-code-boundary |
| C3 模块与插件 | 描述符单源、InitLevel/四阶段（计划 02）、feature 命名与放置（计划 03）、插件声明单源（计划 04） | 本计划集 |
| C4 解耦 | 跨域引用三形态（extract DTO/registry/handle）、契约纯度、禁邻域内部类型（计划 05） | 本计划集 |
| C5 代码质量 | 错误处理（域级 thiserror 错误树、运行时路径禁 panic/unwrap、诊断必须携带上下文）、日志分级与 cadence（Dev profile 独占默认日志）、unsafe 必须注释 invariant 并集中在 FFI/RHI 层、公共 API rustdoc 覆盖 | 新成文 |
| C6 验证 | milestone-first 节奏、测试分层（内核单测/契约测试/守卫测试/集成/冒烟）、测试文件预算 | milestone-first-workflow-policy、structure-convention |

## 4. 守卫矩阵（MUST 规则 → 守卫）

| 守卫 | 拦截什么 | 形态 |
|------|---------|------|
| G1 依赖方向 | 上层 crate 反向依赖、app/editor/插件直连 `zr_*`、asset/graphics 引 ui 内部等 | 已由 Frameworks05 layer-direction 与 Runtime structure guards 接入统一 runner；Cargo crate 边界继续天然强制。current static 28/28，不替代 managed Runtime structure gate |
| G2 结构门 | 根文件行为化、大文件超标、命名禁区、迁移气味词 | Python 与 Rust structure guards 已收编到统一 runner；规则表严格解析 63 rules / 49 MUST，拒绝重复 marker、空规则和未知 guard。current contract tests 19/19，managed Rust gate pending |
| G3 fmt/clippy | 格式与 lint | CI 的 convention job 已显式安装与 runner 相同的 Rust 1.94.1 + rustfmt/clippy，并由 1/1 toolchain contract 锁定；runner 执行 `cargo fmt --all --check`，并对 `zircon_runtime_interface`/`zircon_app` 执行 scoped `--no-deps -D warnings`。Runtime/workspace 全量 lint 仍归 M3 渐进收紧，不用全局 allow 绕过 |
| G4 feature 矩阵 | feature 组合断裂 | 计划 03 profile/feature CI matrices 已落地；runtime selection 双真相与完整 acceptance 仍由计划 03 保持开放 |
| G5 插件一致性 | manifest 单源漂移、catalog 漏注册、符号缺失 | 声明/生成 manifest 静态 parity 已有守卫；Rust `cargo-zircon plugin check` 尚未实现，不能把 Python validator 冒充该工具或 CI 门 |
| G6 依赖治理 | license/重复版本/安全通告、重型依赖越层 | 重型依赖越层已有局部守卫；仓内尚无 `deny.toml`，`cargo deny check` 与 license/advisory/duplicate policy 尚未接入 CI |
| G7 docs 勾稽 | 模块文档 related_code 悬空路径 | current-owner 审计已接入统一 runner/CI 并持续 hard cut 退役路径；全库当前仍为 RED，只有违规明细归零后才可验收 |

## 5. 里程碑

### M0 规范总纲成文（与阶段 0 并行，先行生效）

实现切片：总纲已于 2026-07-02 落稿到 `docs/plans/zircon_runtime/frameworks/development-conventions.md`，含 GEN/RT/ED/PL/IF/WF 分域规则与守卫勾稽；被收编文档的“权威已移至总纲，本文保留细节”勾稽、`docs/engine-architecture/index.md` 路由，以及 C5 对应的 `GEN-Q`/`GEN-D4` 规则均已完成。后续自动化守卫覆盖属于 M1/M2/M3，不回写为 M0 规范缺失。

测试阶段：docs-only；验收证据 = 总纲入库（已完成）+ 7 份 §1/§3 来源文档勾稽行 + 用户指定优先的 `engine-code-review-findings-2026-06.md` 补充审查勾稽 + `docs/engine-architecture/index.md` 收录。原“8 份”计数把没有独立落盘文档的 milestone-first 流程规则也计入来源，现按 §3 实际列出的 7 份文件校正；补充审查文档作为优先路由单独计入，不虚构 milestone-first 第 8 份文档。

### M1 CI 基础门（G3 部分 + G7）

实现切片：已完成。`ci.yml` 的 convention job 显式安装 Rust 1.94.1 fmt/clippy 并调用统一 runner；runner 聚合 layering、Rust structure、fmt、Interface/App scoped clippy 与 docs/G7，PowerShell/Python wrapper 和 tracked CLI 文档已落地。新增 CI/toolchain contract 同时读取 workflow 与 runner，禁止 workflow 退回 `stable` 而 runner 继续调用未安装的 named toolchain。

测试阶段：
- `python -B -m unittest tools.tests.test_frameworks_06_ci_toolchain_contract tools.tests.test_check_conventions -v` current static **20/20**；
- 分支上 CI 全绿一轮（fmt/clippy 存量违规同批清理，interface/app 体量小可控）；
- 验收证据：workflow 截图/日志 + check-conventions 脚本文档化（tracked `docs/cli-and-tooling/check-conventions.md`；本地忽略的 `CLAUDE.md` 命令段仅作为工作区便利入口，不充当入库证据）。

### M2 结构守卫统一入口（G1/G2）

实现切片：代码与静态合同已完成。交叉引用扫描已常驻，Python/Rust guards 进入同一 CI runner，规则表按 63 rules / 49 MUST 严格解析并由 19/19 contract tests 锁定；Frameworks05 layer-direction fresh 28/28。managed `structure_convention`、fmt/clippy 组合门与真实分支 CI acceptance 仍待 current-source 证据，故 M2 不提前标 completed。

测试阶段：
- `cargo test -p zircon_runtime structure_convention --locked` 全绿 + CI job 全绿；
- 验收证据：总纲守卫勾稽表无"MUST 无守卫"行（clippy 全量与 G4/G5/G6 标注"随对应计划落地"）。

### M3 渐进收紧与收口（G3 全量 / G6）

实现切片：runtime 各域按计划 01 拆分节奏逐 crate 进 clippy 零警告名单；cargo-deny 配置与 CI 接入（配合计划 01 M4）；豁免流程落地（`#[allow]` 必须带 `// EXEMPT(规则ID): 理由` 注释，守卫统计豁免数量趋势）。

测试阶段：CI 全矩阵绿；验收证据：clippy 名单覆盖全部成员 crate；豁免清单首期报告。

## 6. 风险与回退

- **守卫误伤节奏**：milestone-first 政策优先——守卫跑在 CI 与里程碑测试阶段，不强加到每个实现切片；本地 check-conventions 是自助工具不是强制钩子。
- **存量 clippy 债务**：绝不一次性 `-D warnings` 全仓；allowlist 递减制并把名单进度记录在本文件状态表。
- **规范双源漂移**：总纲生效后，规则修改只允许改总纲并同步守卫；来源文档只保留细节论证。守卫 G7 顺带检查总纲勾稽表的守卫 ID 有效性。

## 7. 状态与产出记录

2026-07-31 状态校正：M0 completed；M1 实现 code-complete，CI/toolchain contract 1/1 与既有 runner contract 19/19 组成 current static 20/20，真实分支 CI acceptance pending；M2 静态实现与独立 review 已完成（runner contract 19/19、Frameworks05 layer-direction 28/28），managed Runtime structure/fmt/clippy 组合证据 pending；M3 未完成。G5 Rust `cargo-zircon` 和 G6 `cargo-deny` 均不存在，G7 全库仍 RED。开放的 [animation-scene-hook-guard-stale-path](../frameworks/05/failure-2026-07-29-animation-scene-hook-guard-stale-path.md) 已在 Frameworks05/Plugins04 完成静态 hard cut，仍待 Plugins04 managed compile/fixed return；queued/running validation 只延迟 accepted closeout，不令本 Session blocked，也不停止其他 G7/规范收敛。

- fixed 已修复：[scene-test-support-file-budget](06/fixed-2026-07-13-scene-test-support-file-budget.md)
- fixed 已修复：[rustfmt-path-attributed-typed-canvas](06/fixed-2026-07-13-rustfmt-path-attributed-typed-canvas.md)
- fixed 已修复：[workbench-projection-file-budget-regression](06/fixed-2026-07-13-workbench-projection-file-budget-regression.md)

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

- 产出记录：[`06/2026-07-12-development-conventions-and-guardrails-output-records.md`](06/2026-07-12-development-conventions-and-guardrails-output-records.md)、[`06/2026-07-13-development-conventions-and-guardrails-output-records.md`](06/2026-07-13-development-conventions-and-guardrails-output-records.md)
- G7 current-owner 记录：[`06/2026-07-18-g7-editor-transaction-owner-doc-hardcut-batch11.md`](06/2026-07-18-g7-editor-transaction-owner-doc-hardcut-batch11.md)（两份 Editor 命令文档已从删除的 flat history owner 硬切到 `engine/history.rs`；`EditorTransactionEngine`、`HistoryStore` 与 gizmo capture 分别同步到 `engine/transaction.rs`、`engine/history.rs`、`editor_state_viewport.rs` 真实 owner；focused 0 violations，追加复审修正 2 个 Important 后最终 Critical/Important/Minor=0/0/0）
- G7 current-owner 记录：[`06/2026-07-18-g7-core-error-current-owner-doc-hardcut-batch14.md`](06/2026-07-18-g7-core-error-current-owner-doc-hardcut-batch14.md)（Frameworks01/02 两份记录的 3 个退役 `core/framework/error.rs` front-matter 路径已硬切到唯一 `core/runtime/error.rs`；focused G7 为 0，历史时序归因修正后最终复审 Critical/Important/Minor=0/0/0）
- G7 current-owner 记录：[`06/2026-07-18-g7-export-profile-current-owner-doc-hardcut-batch15.md`](06/2026-07-18-g7-export-profile-current-owner-doc-hardcut-batch15.md)（Runtime01 技术选型记录已从退役 `plugin/export_profile.rs` 硬切到 `core/framework/project/export_profile.rs`，同步显式 RuntimeProfileId/fatal-plan、zstd/zip/tar 与 `in_progress` current facts；focused G7 为 0，最终复审 Critical/Important/Minor=0/0/0）
- G7 current-owner 记录：[`06/2026-07-18-g7-dynamic-session-registry-owner-doc-hardcut-batch16.md`](06/2026-07-18-g7-dynamic-session-registry-owner-doc-hardcut-batch16.md)（结构规范已从退役 flat `dynamic_api/session/registry.rs` 硬切到 folder-backed `registry/mod.rs`，同步 current route/child 与直接 consumer 拓扑；focused G7 为 0，最终复审 Critical/Important/Minor=0/0/0）
- G7 current-owner 记录：[`06/2026-07-18-g7-text-geometry-current-owner-doc-hardcut-batch17.md`](06/2026-07-18-g7-text-geometry-current-owner-doc-hardcut-batch17.md)（Text03 计划已从删除的 flat interface `text_geometry.rs` 硬切到 folder-backed `text_geometry/mod.rs` 与 `source_map.rs`，同步 command decoration 直连及 interface re-export → Runtime geometry/hit-test 消费拓扑；focused G7 为 0，两轮 Important 修正后最终复审 Critical/Important/Minor=0/0/0）
- G7 current-owner 记录：[`06/2026-07-18-g7-input-manager-current-owner-doc-hardcut-batch18.md`](06/2026-07-18-g7-input-manager-current-owner-doc-hardcut-batch18.md)（Plugins09 开放 failure 已从删除的 flat `input/input_manager.rs` 硬切到中立 trait、生产 manager 与 frame event-buffer owners，同步 Runtime 内部 cursor/raw-delta 合并和 host ABI 前仍未关闭的责任边界；focused G7 为 0，最终复审 Critical/Important/Minor=0/0/0）
- G7 current-owner 记录：[`06/2026-07-18-g7-sdf-font-bake-current-owner-doc-hardcut-batch19.md`](06/2026-07-18-g7-sdf-font-bake-current-owner-doc-hardcut-batch19.md)（EditorLayout17 已从删除的 scene-renderer SDF bake owner 硬切到 Runtime Text `text/sdf/font_bake.rs`，保持编辑器验收契约/Runtime Text 实现权分离；focused G7 为 0，最终复审 Critical/Important/Minor=0/0/0）
- G7 current-owner 记录：[`06/2026-07-18-g7-reference-owner-doc-hardcut-batch20.md`](06/2026-07-18-g7-reference-owner-doc-hardcut-batch20.md)（Bevy `EditableText` reference owner 已硬切到 `editing.rs`，MUI shape token reference owner 已硬切到 `shape.ts`，历史 baseline 保留但 current machine path 不再指向删除文件；focused G7 为 0，最终复审 Critical/Important/Minor=0/0/0）
- G7 current-owner 记录：[`06/2026-07-18-g7-bevy-material-bind-group-owner-doc-hardcut-batch21.md`](06/2026-07-18-g7-bevy-material-bind-group-owner-doc-hardcut-batch21.md)（Render19 已从删除的 Bevy PBR 私有 `material_bind_groups.rs` 硬切到 current `bevy_render/src/material_bind_groups.rs` 通用 owner，并明确 `bevy_pbr/src/material.rs` 只作为 consumer；focused G7 为 0，证据状态修正后最终复审 Critical/Important/Minor=0/0/0）
- G7 current-owner 记录：[`06/2026-07-18-g7-runtime-absorption-evidence-owner-doc-hardcut-batch22.md`](06/2026-07-18-g7-runtime-absorption-evidence-owner-doc-hardcut-batch22.md)（Render05 fixed 记录已从删除的 `.codex/sessions` fixture 硬切到 Runtime15 tracked archive 与 `runtime_absorption/current_source_fixture.rs` current owners；focused G7 为 0，证据状态修正后最终复审 Critical/Important/Minor=0/0/0）
- G7 current-owner 记录：[`06/2026-07-18-g7-fullscreen-plan-owner-doc-hardcut-batch23.md`](06/2026-07-18-g7-fullscreen-plan-owner-doc-hardcut-batch23.md)（Editor02 的 Shader04 fixed 记录已从删除的 fullscreen descriptor owner 硬切到 `graphics/shader/builtin_global_shader_contracts.rs` current owner，并保持 consuming-builder hard cut；focused G7 为 0，证据状态修正后最终复审 Critical/Important/Minor=0/0/0）
- M2 scene component owner hard-cut：[`06/2026-07-19-scene-component-owner-hardcut.md`](06/2026-07-19-scene-component-owner-hardcut.md)（删除 796 行 flat `scene.rs`，按 identity/hierarchy/transform/activation/camera/mesh_renderer/physics 等 domain owner 硬切；reflection guard 逐类型绑定真实 owner 与相邻 `ZrReflect` derive；current-source exact 1/1、8528 filtered、复审 Critical/Important/Minor=0/0/0；Frameworks06 总计划仍保持 `in_progress`。）
- M2 unified G1/G2 convention gate：[`06/2026-07-19-m2-unified-g1-g2-convention-gate.md`](06/2026-07-19-m2-unified-g1-g2-convention-gate.md)（唯一 runner 现按严格 header/separator/data-row 状态逐行验证精确 63 rules/49 MUST 与受控 guard 词表，拒绝重复表标记、空规则正文及 SHOULD 未知非空 guard；JSON stdout 保持单对象，传播子门退出码并将启动失败结构化为 `launch_error`。PowerShell 多 gate 契约已从错误的重复参数名硬切为单数组 `-Only`，Python 入口继续使用可重复 `--only`；静态契约 19/19、layering 28/28。snapshot1208 exact5 独立复审 Critical/Important/Moderate/Minor=`0/0/0/0`、Ready；managed structure/fmt/clippy 尚待 Runtime 全输入安静窗口，故切片与 M2 均不声明完成。）
- 当前状态：M0 已完成。M1 进行中：本地聚合工具、模块文档、契约测试与 CI 单一入口已落地，历史全库 fmt、G7 与首批 Runtime Interface/App scoped clippy 通过证据由编号归档持有；`--no-deps` 明确只提升首批包的 lint，Runtime 依赖债务保留给 M3 渐进收紧，不使用 allowlist。优先结构/评审守卫已有 current Runtime binary 完整验收，Editor Workbench projection 预算 fixed 回迁，Render OIT limit 与 build-mesh-draws lightmap sync 锚均硬切到当前事实。2026-07-13 已将 6 份 Runtime text/graphics/UI 文档中的 16 条退役 SDF owner 机器路径硬切到唯一现存 owner；2026-07-16 又在 30 份 clean 文档的 `related_code` / `implementation_files` 中硬切 236 条退役 Text owner 路径，所选两类旧前缀归零且 132 个唯一新目标均存在。2026-07-18 Batch11 将两份 clean Editor 命令文档的 4 条 flat history owner 引用硬切到当前 transaction/history owner，并按追加复审修正 command apply/commit 时序与 gizmo owner，focused 违规归零；Batch12 又从 extension-registry 文档 front matter 删除 2 条把测试函数锚点伪装成文件路径的记录，保留唯一真实测试 owner 文件，focused 违规归零且未放宽 G7 解析规则；Batch13 已从 Audio 中立契约文档删除不存在的 Sound 私有 channel-layout 文件路径，focused G7 归零，但独立复审发现 Sound root 仍有 crate-visible `AudioChannelLayout`/test-only `AudioSpeakerChannel` alias，已作为 Frameworks03 failure 优先修复，Batch13 尚未通过复审或验收；Batch14 将 Frameworks01/02 记录中的 3 个退役 CoreError owner 路径硬切到唯一 `core/runtime/error.rs`，focused G7 归零且不改历史错误语义；Batch15 又把 Runtime01 技术选型记录的退役 ExportProfile owner 硬切到 `core/framework/project/export_profile.rs`，同步显式 RuntimeProfileId 与 fatal-plan contract；Batch16 将结构规范中的 flat DynamicApi registry owner 硬切到 folder-backed route/child owners；Batch17 将 Text03 的 flat interface text geometry owner 硬切到 current folder-backed decoration/source-map owners；Batch18 将 Plugins09 开放 failure 的 flat InputManager owner 硬切到 current trait/production manager/frame-buffer owners，同时保持 host ABI 前 coalescing failure 开放；Batch19 将 EditorLayout17 的 scene-renderer SDF bake owner 硬切到 current Runtime Text owner并保持验收/实现权分离；Batch20 将当前 Bevy/MUI reference tree 中两个退役 owner 硬切到 `editing.rs` / `shape.ts`，不把历史 baseline 伪装成 current path；Batch21 将 Render19 的 Bevy material bind-group reference 从已删除的 PBR 私有文件硬切到 `bevy_render` 通用 owner，保持 PBR consumer 与渲染底座 owner 分离；Batch22 将 Render05 fixed 记录的 `.codex/sessions` 退役 fixture 从 machine owner 降为历史失败证据，current related_code 只指向 Runtime15 tracked archive 与共享 fixture constant owner；Batch23 将 Shader04 fullscreen descriptor 的删除文件从 fixed record machine owner 硬切到 builtin global shader contracts，继续锁定 consuming builder 而不恢复旧 helper。全库 G7 仍为 RED；文档数与检查路径数会随并发工作树增减，不在父计划固化，违规明细才是后续收敛依据。其余所有权漂移与并发 dirty owner 必须继续按独立清单收敛；真实分支 CI 也尚无实际执行证据，因此不声明 M1 或计划 06 完成。精确历史验收记录仍只由上述编号归档持有。
