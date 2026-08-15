---
related_code:
  - .github/workflows/ci.yml
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate.rs
  - tools/plugin_structure_audits/capability.py
  - tools/check_conventions.py
  - tools/convention_exemptions.py
  - tools/check-conventions.ps1
  - tools/tests/test_check_conventions.py
  - tools/tests/check_conventions/document_paths.py
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
last_refined: 2026-08-09
status: in_progress
---

# 06 · 开发规范总纲与守卫机制

## 1. 目标

引擎开发需要系统性纪律，而纪律只有变成机器可执行的守卫才可持续。本计划做两件事：

1. **规范单源化**：把散落在 `engine-code-structure-convention.md`、engine-architecture 各 M1 文档、`.claude/.codex` 技能、审计脚本注释里的规则，收敛为一份分层的《Zircon 开发规范总纲》，每条规则标注：级别（MUST/SHOULD）、适用范围、守卫方式、豁免流程。
2. **守卫机制化**：每条 MUST 规则对应一个自动守卫（编译器 / 守卫测试 / 审计脚本 / CI 步骤），没有守卫的 MUST 视为未完成。

## 2. 现状与差距

- 规则总纲与 tracked CLI 文档已经提供人类/AI 共用入口，用户优先的 structure/review authorities 也已同步；剩余差距是历史细节文档和 G7 路径仍持续漂移，不能把总纲落地误报为全仓收敛。
- 统一 `tools/check_conventions.py` 已聚合 Frameworks05 layering、Runtime structure convention、Rust 1.94.1 `cargo fmt --all --check`、Runtime Interface/App scoped `clippy --no-deps -D warnings` 与 docs/G7 审计，并由 CI 安装 fmt/clippy 后调用；feature/profile matrices 由计划 03 的 CI job 持有。计划 04 Rust `cargo-zircon plugin new/check/validate` 与 G6 `deny.toml`/双 workspace CI matrix 已实现。未完成项是 workspace/Runtime 全量 clippy、G5/G6 真实 managed/CI acceptance，以及仍为 RED 的全库 G7。
- 错误处理、日志、panic 策略、unsafe 审查与公共 API rustdoc 已分别由权威总纲 `GEN-Q1..Q7`、`GEN-D4` 成文；Rust allow 豁免自动守卫及按成员/规则统计已落地，当前缺口是其余仍依赖评审的规则尚未全部转为自动化 G3/专项守卫。

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
| G2 结构门 | 根文件行为化、大文件超标、命名禁区、迁移气味词 | Python 与 Rust structure guards 已收编到统一 runner；规则表严格解析 63 rules / 49 MUST，拒绝重复 marker、空规则和未知 guard。current convention contract tests 26/26，managed Rust gate pending |
| G3 fmt/clippy | 格式与 lint | CI 的 convention job 已显式安装与 runner 相同的 Rust 1.94.1 + rustfmt/clippy，并由 1/1 toolchain contract 锁定；runner 执行 `cargo fmt --all --check`，并对 `zircon_runtime_interface`/`zircon_app` 执行 scoped `--no-deps -D warnings`。Runtime/workspace 全量 lint 仍归 M3 渐进收紧，不用全局 allow 绕过 |
| G4 feature 矩阵 | feature 组合断裂 | 计划 03 profile/feature CI matrices 已落地；runtime selection 双真相与完整 acceptance 仍由计划 03 保持开放 |
| G5 插件一致性 | manifest 单源漂移、catalog 漏注册、符号缺失 | Rust `cargo-zircon plugin new/check/validate` 已实现，`plugin check` 覆盖 manifest 单源、catalog/workspace/feature 勾稽与 dist artifact 符号探测并接入 CI；当前为 implementation-complete / managed E2E acceptance pending，Python validator 只保留回归层而非兼容入口 |
| G6 依赖治理 | license/重复版本/安全通告、重型依赖越层 | 根 `deny.toml` 与 root/plugin workspace CI matrix 已实现：advisory ignore 为空，未知 registry/Git source 与 wildcard 依赖拒绝，duplicate version 显式告警，copyleft 仅保留现有 crate 级 MPL-2.0 例外。静态策略契约 4/4、402 manifest wildcard 盘点 0 项和独立复审 C0/I0/M0 已通过；此前 exact7 受管策略契约 4/4 GREEN，当前父计划状态更新后需刷新不可变验证，真实 cargo-deny/CI acceptance 仍待完成 |
| G7 docs 勾稽 | 模块文档 related_code/implementation_files/tests 悬空路径 | current-owner 审计已接入统一 runner/CI；`tests:` 仅审计整条声明的具体仓库文件并拒绝绝对/越界路径，不把命令、glob、模板、远程 URI 或构建产物误报为 owner。用户优先 structure authority、review findings 与 module-convention 已分别从 186/185/103 项降为 0；已完成的 Runtime05 计划只做 35→0 元数据维护，不重开里程碑。G7 路径 validator 已按父目录缓存普通 leaf，同时对 `..`、symlink/junction leaf 保持 full resolve；Windows reparse 判定使用 Python 3.11 可用的 `lstat` attributes，不静默依赖 3.12 API。current-source old-field report 等价且 full-resolution -78.3926%，31/31 static GREEN。fresh exact7 内容复审为 C0/I0/M0，但 snapshot1615 漏掉 runner 必需的 untracked `tools/convention_exemptions.py`，已按原子提交规则拒绝；fresh exact8 不可变二次审查为 C0/I0/M0。5-sample 墙钟因外部 docs 漂移被拒绝，performance attestation/managed closeout pending。这些 owned 明细均为 0，但共享工作树仍为 RED，全局计数因并发 owner 输入不冻结，只有全库明细归零后才可验收 |

## 5. 里程碑

### M0 规范总纲成文（与阶段 0 并行，先行生效）

实现切片：总纲已于 2026-07-02 落稿到 `docs/plans/zircon_runtime/frameworks/development-conventions.md`，含 GEN/RT/ED/PL/IF/WF 分域规则与守卫勾稽；被收编文档的“权威已移至总纲，本文保留细节”勾稽、`docs/engine-architecture/index.md` 路由，以及 C5 对应的 `GEN-Q`/`GEN-D4` 规则均已完成。后续自动化守卫覆盖属于 M1/M2/M3，不回写为 M0 规范缺失。

测试阶段：docs-only；验收证据 = 总纲入库（已完成）+ 7 份 §1/§3 来源文档勾稽行 + 用户指定优先的 `engine-code-review-findings-2026-06.md` 补充审查勾稽 + `docs/engine-architecture/index.md` 收录。原“8 份”计数把没有独立落盘文档的 milestone-first 流程规则也计入来源，现按 §3 实际列出的 7 份文件校正；补充审查文档作为优先路由单独计入，不虚构 milestone-first 第 8 份文档。

### M1 CI 基础门（G3 部分 + G7）

实现切片：已完成。`ci.yml` 的 convention job 显式安装 Rust 1.94.1 fmt/clippy 并调用统一 runner；runner 聚合 layering、Rust structure、fmt、Interface/App scoped clippy 与 docs/G7，PowerShell/Python wrapper 和 tracked CLI 文档已落地。新增 CI/toolchain contract 同时读取 workflow 与 runner，禁止 workflow 退回 `stable` 而 runner 继续调用未安装的 named toolchain。

测试阶段：
- `python -B -m unittest tools.tests.test_frameworks_06_ci_toolchain_contract tools.tests.test_check_conventions -v` current static **31/31**；
- 分支上 CI 全绿一轮（fmt/clippy 存量违规同批清理，interface/app 体量小可控）；
- 验收证据：workflow 截图/日志 + check-conventions 脚本文档化（tracked `docs/cli-and-tooling/check-conventions.md`；本地忽略的 `CLAUDE.md` 命令段仅作为工作区便利入口，不充当入库证据）。

### M2 结构守卫统一入口（G1/G2）

实现切片：代码与静态合同已完成。交叉引用扫描已常驻，Python/Rust guards 进入同一 CI runner，规则表按 63 rules / 49 MUST 严格解析并由 26/26 convention contract tests 锁定；Frameworks05 layer-direction fresh 28/28。managed `structure_convention`、fmt/clippy 组合门与真实分支 CI acceptance 仍待 current-source 证据，故 M2 不提前标 completed。

测试阶段：
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter structure_convention` 全绿 + CI job 全绿；
- 验收证据：总纲守卫勾稽表无"MUST 无守卫"行（clippy 全量与 G4/G5/G6 标注"随对应计划落地"）。

### M3 渐进收紧与收口（G3 全量 / G6）

实现切片：runtime 各域按计划 01 拆分节奏逐 crate 进 clippy 零警告名单。Rust 豁免守卫实现已落地到统一 runner：从 workspace manifest 盘点全部 `#[allow]`/`#![allow]`，对首批零警告成员 `zircon_app`/`zircon_runtime_interface` 强制紧邻 `// EXEMPT(已知 MUST 规则ID): 非空理由`，并按成员/规则报告未收紧存量趋势。2026-08-08 exemption lexer/inventory 已抽取到独立 module；抽取后静态契约 23/23、直接 Python 门与 PowerShell `exemptions` JSON 门均 GREEN。2026-08-09 最新共享树库存为 421 个候选文件、148 个实际属性、0 个严格成员违规；该计数是会随并发源码变化的库存快照，不冒充不可变 acceptance。独立二次复审 C0/I0/M0。G6 的 cargo-deny policy 与 CI 双 workspace matrix 已实现，静态契约、独立复审和 exact7 受管策略契约均 GREEN；真实 cargo-deny/CI acceptance 尚待完成。managed 默认 runner acceptance 与 workspace 全量 clippy 也仍待完成，因此 M3 不提前标 completed。

测试阶段：CI 全矩阵绿；验收证据：clippy 名单覆盖全部成员 crate；豁免清单首期报告。

## 6. 风险与回退

- **守卫误伤节奏**：milestone-first 政策优先——守卫跑在 CI 与里程碑测试阶段，不强加到每个实现切片；本地 check-conventions 是自助工具不是强制钩子。
- **存量 clippy 债务**：绝不一次性 `-D warnings` 全仓；allowlist 递减制并把名单进度记录在本文件状态表。
- **规范双源漂移**：总纲生效后，规则修改只允许改总纲并同步守卫；来源文档只保留细节论证。守卫 G7 顺带检查总纲勾稽表的守卫 ID 有效性。

## 7. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前状态：M0 已完成；M1 实现 code-complete，但真实分支 CI acceptance 仍待完成；M2 静态实现与独立复审已完成，managed Runtime structure/fmt/clippy 组合证据仍待补齐；M3 Rust 豁免守卫功能实现、模块边界 closeout与抽取后二次复审均已完成，G6 实现、静态契约和独立复审已完成，此次父计划/G7 更新后需刷新不可变策略证据，真实 cargo-deny/CI acceptance 和 workspace 全量 clippy 仍待补齐。G5 已 implementation-complete / managed E2E acceptance pending；G7 三份用户优先/结构 owner 与 Runtime05 元数据均已归零、共享工作树仍为 RED且全局计数不冻结。snapshot1615/exact7 因遗漏 `tools/convention_exemptions.py` 已拒绝；fresh exact8 不可变二次审查 C0/I0/M0，managed closeout pending，父计划不提升完成状态。Frameworks05/Plugins04 的 animation scene-hook hard cut 已完成静态修复，仍等待 managed compile 与 fixed return。

- 迁入产出记录：[2026-08-01 产出与性能交接归档](06/2026-08-01-plan-output-and-performance-handoffs.md)
- fixed 已修复：[scene-test-support-file-budget](06/fixed-2026-07-13-scene-test-support-file-budget.md)
- fixed 已修复：[rustfmt-path-attributed-typed-canvas](06/fixed-2026-07-13-rustfmt-path-attributed-typed-canvas.md)
- fixed 已修复：[workbench-projection-file-budget-regression](06/fixed-2026-07-13-workbench-projection-file-budget-regression.md)
- 当前关联开放 failure：[animation-scene-hook-guard-stale-path](../frameworks/05/failure-2026-07-29-animation-scene-hook-guard-stale-path.md)
