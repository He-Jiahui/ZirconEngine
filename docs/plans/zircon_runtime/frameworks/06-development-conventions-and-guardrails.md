---
related_code:
  - .github/workflows/ci.yml
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate.rs
  - tools/plugin_structure_audits/capability.py
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
---

# 06 · 开发规范总纲与守卫机制

## 1. 目标

引擎开发需要系统性纪律，而纪律只有变成机器可执行的守卫才可持续。本计划做两件事：

1. **规范单源化**：把散落在 `engine-code-structure-convention.md`、engine-architecture 各 M1 文档、`.claude/.codex` 技能、审计脚本注释里的规则，收敛为一份分层的《Zircon 开发规范总纲》，每条规则标注：级别（MUST/SHOULD）、适用范围、守卫方式、豁免流程。
2. **守卫机制化**：每条 MUST 规则对应一个自动守卫（编译器 / 守卫测试 / 审计脚本 / CI 步骤），没有守卫的 MUST 视为未完成。

## 2. 现状与差距

- 规则已多而好，但分布在 ≥8 份文档 + 2 套技能目录 + Python 审计脚本，新贡献者无单一入口；部分规则仅存在于技能文件（AI 会话可见，人类贡献者不可见）。
- 守卫不成体系：结构守卫测试（`module_convention_gate.rs` 等）与 Python 审计并存但覆盖面与 CI 挂接不完全；CI 无 clippy、无 fmt 门（fmt 命令在文档但 workflow 未见强制）、无依赖方向守卫、无 feature 组合矩阵（归计划 03）、无 cargo-deny。
- 错误处理、日志、panic 策略、unsafe 审查、公共 API 文档密度等"代码质量类"规范尚无成文条目。

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
| G1 依赖方向 | 上层 crate 反向依赖、app/editor/插件直连 `zr_*`、asset/graphics 引 ui 内部等 | 计划 01 后大部分由 Cargo 天然强制；剩余同层规则用守卫测试扫 `cargo metadata` + use 交叉引用（计划 05 M1 的扫描脚本常驻化） |
| G2 结构门 | 根文件行为化、大文件超标、命名禁区、迁移气味词 | 既有结构守卫测试 + Python 审计收编，统一由一个 `cargo test -p zircon_runtime structure_convention` 入口跑全 |
| G3 fmt/clippy | 格式与 lint | CI 加 `cargo fmt --all --check`；clippy 分两步走——先 `-p zircon_runtime_interface -p zircon_app` 零警告门，runtime 大库按域渐进收紧（allowlist 递减制），终态 workspace `-D warnings` |
| G4 feature 矩阵 | feature 组合断裂 | 计划 03 M2 的 CI 矩阵 job |
| G5 插件一致性 | manifest 单源漂移、catalog 漏注册、符号缺失 | 计划 04 的 `cargo zircon plugin check` 入 CI |
| G6 依赖治理 | license/重复版本/安全通告、重型依赖越层 | `cargo deny check` + 守卫测试断言（如 `zr_asset` 依赖树无 wgpu） |
| G7 docs 勾稽 | 模块文档 related_code 悬空路径 | 审计脚本：抽取 docs 头部路径存在性检查，入 CI |

## 5. 里程碑

### M0 规范总纲成文（与阶段 0 并行，先行生效）

实现切片：总纲落稿于 `docs/plans/zircon_runtime/frameworks/development-conventions.md`（**已于 2026-07-02 完成**，含 GEN/RT/ED/PL/IF/WF 分域规则与守卫勾稽）；剩余切片：被收编文档头部加"权威已移至总纲，本文保留细节"勾稽行（不删原文档，避免破坏既有引用网）；`docs/engine-architecture/index.md` 收录指向；C5 代码质量章（GEN-Q 组）评审定稿。

测试阶段：docs-only；验收证据 = 总纲入库（已完成）+ 8 份来源文档勾稽行 + `docs/engine-architecture/index.md` 收录。

### M1 CI 基础门（G3 部分 + G7）

实现切片：ci.yml 加 fmt 门、interface/app 两包 clippy 零警告门、docs 勾稽脚本；本地 `tools/` 加一键 `check-conventions` 脚本聚合全部守卫。

测试阶段：
- 分支上 CI 全绿一轮（fmt/clippy 存量违规同批清理，interface/app 体量小可控）；
- 验收证据：workflow 截图/日志 + check-conventions 脚本文档化（CLAUDE.md 命令段）。

### M2 结构守卫统一入口（G1/G2）

实现切片：交叉引用扫描（计划 05 M1 产物）常驻为守卫测试；Python 审计与 Rust 结构守卫在 CI 中统一 job；G2 全清单与总纲条目一一勾稽（总纲每条 MUST 标注守卫 ID）。

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
