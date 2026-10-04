---
related_code:
  - tools/dev/local-cargo.ps1
  - tools/dev/local_cargo.py
  - tools/jenkins_coordinator.py
  - tools/dev/zircon-session.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - zircon_runtime/src/asset/migration
  - zircon_runtime/src/asset/reference_resolver.rs
related_tests:
  - tools/tests/test_local_cargo.py
  - tools/tests/session-coordinator-smoke.Tests.ps1
  - tools/session_coordinator/tests
  - zircon_runtime/src/asset/tests/migration/project_commandlet/resolver_index.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/source_boundary.rs
plan_sources:
  - docs/tooling/coordinator-retirement.md
  - docs/plans/milestone-validation-policy.md
  - docs/plans/mvp/index.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/04/failure-2026-07-23-asset-migration-indexed-resolver-generation.md
status: in_progress
gate: baseline
last_refined: 2026-10-03
---

# 当前源码与验证基线恢复 Implementation Plan

- fixed 已修复：[validation ticket deletion manifest](00/fixed-2026-08-05-validation-ticket-deletion-manifest.md)；删除墓碑协议、目录/悬空链接重现拒绝与最终复审均已收敛，source-bound managed batch `30/30` GREEN；Runtime15 可恢复硬删除 manifest 票据。

> **For agentic workers:** REQUIRED SUB-SKILL: 使用 `subagent-driven-development`（推荐）或 `executing-plans` 按里程碑执行；开始前使用 `cross-session-coordination`，测试阶段使用 `zircon-dev-validation` 和 `support-first-regression-testing`。

**Goal:** 恢复机器可读的受管验证入口，完成 Runtime 04 migration resolver index 的单一 generation 接线，并使 `zircon_runtime`、`zircon_editor`、`zircon_app` 当前源码重新通过包级编译检查。

**Architecture:** 先核验当前独立验证入口，再修最早的 Runtime 04 公共编译原因，最后从 Runtime 向 Editor/App 逐层收敛编译。禁止在上层 call site 增加临时 fallback，也不把声明 `mod resolver_index` 当成 migration 架构已完成。

**Tech Stack:** PowerShell、Python、Rust、独立本地 Cargo 与 Jenkins、Windows 原生验证。

## 当前控制面（2026-10-03）

旧本地协调器及其 matrix、登记、心跳、租约、票据和关闭流程已退役。本页保留旧源码、命令、job ID 与回执作为历史证据；这些记录不要求恢复服务，也不授权重放旧请求。执行以[退役规则](../../cli-and-tooling/coordinator-retirement.md)和[验证策略](../milestone-validation-policy.md)为准。

当前本地证据使用 `tools/dev/local-cargo.ps1` 或 `python -B -m tools.dev.local_cargo`，物理输出与缓存只放在盘根 `D:\cargo-targets`、`E:\cargo-targets` 或 `F:\cargo-targets` 下，并通过 35 GiB 启动预检。独立 Jenkins 的功能测试和有边界提交还须核验当前 `active.json`、source/driver/tray、storage owner/keeper 与实际进程身份，遵循 [jenkins-coordination](../../../.codex/skills/jenkins-coordination/SKILL.md)。M8 唯一入口规则按实际激活证据执行，不改写其状态来绕过门槛。

本地命令通过、静态审阅和历史票据均不自动授予 Jenkins、里程碑、生产迁移或 F0-F5 接受。本计划的 M0.1、M0.2、M0.3 仍需各自适用的当前证据；未完成项保持开放。

---

## 1. 当前证据

- 2026-07-24 的 `super::resolver_index` unresolved import 已过时：`migration/mod.rs` 当前声明并 re-export `resolver_index`，`MigrationResolver` 只持有 `AssetRegistryIndex` 与 `MigrationResolverIndex`，resolver 内无 root walk、`PathBuf`/`fs` probe 或 `persisted_source_path_for_locator` fallback。
- 历史 coordinator JSON stdout 污染已修复：`tools/dev/zircon-session.ps1` 的两处 `Coordinator ready.` 都受 `if (-not $Json)` 门控；旧 smoke test 覆盖 cold register、daemon reuse、JSON `ConvertFrom-Json` 与 human readiness。这些服务启动和旧队列提交步骤已退役，不再执行。
- 当前最低编译基线已变化：2026-08-01 managed `zircon_runtime --no-default-features --lib` focused job `c630da233fc440559b1788eafebddf9a` 在 lib-test harness 有 16 个 owner-bound 编译错误；default-feature job `367cc00c08394ae4b4705d2fe3c6f44f` 有 53 个。M0.3 因此仍未完成，但不再由 JSON 协议或 resolver module registration 阻断。
- 当前 checkout 包含大量用户和其他任务变化。本计划不清理、不还原、不重排这些变化，通过明确路径归属、source fingerprint、原生进程/锁和逐层验证建立可复现基线。

## 2. 入口条件

- [ ] 已核对当前任务、可见活跃 chats 与相关交接；重叠范围具有明确 source owner。
- [ ] 已阅读 Runtime 04、Tooling 和受影响上层 plan 的现有 open failure 与回执；保留原请求身份，不重放未知结果。
- [ ] 已记录 scoped diff、修改前字节与索引归属；其他任务及用户变化保持原样，不使用退役租约接口。
- [ ] 已记录当前 commit、`Cargo.lock` 指纹、工作区路径级状态计数和 validator 脚本指纹，作为本里程碑 source-bound evidence。

## 3. 非目标

- 不关闭 Runtime 04 下与 MVP 无直接关系的 importer、watcher、virtual geometry 或大规模性能 failure。
- 不在本阶段构建或运行最终产品二进制；产品 build/start 属于 F0。
- 不强制当前共享 checkout 变为 clean，也不删除未跟踪文件。
- 不通过忽略 `ready` 后所有文本的宽松解析来掩盖 machine-readable 协议污染。

## 4. Owner 边界

| 范围 | 权威 owner | 本计划职责 |
|---|---|---|
| 当前 CLI/stdout 与命令证据 | `tools/dev/local_cargo.py`、独立 Jenkins 入口 | 保留实际退出码、机器可读结果与输入身份；旧 coordinator 只保留历史资料 |
| validator 与 target policy | `.codex/skills/zircon-dev`、`tools/dev/local_cargo.py` | 验证物理盘根、容量及原生执行结果；不把 dry run 作为通过 |
| migration inventory/resolver | Runtime 04 | 完成 index 的生命周期接线和 focused contract |
| Editor/App 编译 follow-up | 对应最低 owner | 只修阻断当前编译的共享根因；跨 owner 走 failure handoff |

## 5. M0.1 受管验证输出协议

### 目标

核验当前独立验证入口的输出、物理 target、输入身份与终止结果。旧 `zircon-session.ps1 ... -Json` 实现和历史测试保留为迁移依据，当前测试不启动旧 daemon、worker、tray 或队列。

### 历史实现切片与当前入口核验

- [x] `tools/tests/session-coordinator-smoke.Tests.ps1` 覆盖 cold register、daemon reuse，并直接 `ConvertFrom-Json` 解析 stdout。
- [x] 非 JSON 模式继续断言 `Coordinator ready.`，普通 CLI 反馈未被吞掉。
- [x] `tools/dev/zircon-session.ps1` 仅在 `-not $Json` 时输出 readiness；JSON starting/response 各保持单一 document 语义。
- [x] `validate-matrix.ps1` 继续严格 `ConvertFrom-Json`；不得恢复宽松截断或忽略前导文本。
- [x] validation ticket `source_manifest` 以 JSON `null` 封存归属删除路径；queue claim、copy overlay 与 materialized-copy 的路径重现统一判为 `snapshot_stale`。
- [ ] 运行当前本地入口的 focused 支持回归，检查错误退出码、物理输出拒绝、容量门与 dry-run 无编译语义。
- [ ] 若执行 Jenkins 功能测试，先核验激活/源码/进程证据，再使用固定模板与新的输入身份；精确 reconcile 已提交请求，不借新提交掩盖旧未知结果。

### 测试阶段：M0.1 Tooling Contract Gate

- [ ] 运行当前独立入口的 Python focused suites；历史 smoke 中会启动旧服务的测试不作为当前操作步骤。
- [ ] 从 Windows 原生环境核验当前入口的目标路径和机器可读结果；dry run 仅证明准备行为，不能关闭编译或接受门。
- [ ] Jenkins 适用时，按 `jenkins-coordination` 的 snapshot/driver/request/verify 契约运行固定模板并核对实际归档字节与 native termination。
- [ ] 失败时修复最低工具 owner 或返回具体阻塞；继续不依赖该失败的独立工作，不恢复旧控制面。

### 退出证据

- [ ] 当前入口的输出、实际退出码和错误边界有行为回归证据。
- [ ] 当前编译/测试请求具有明确输入与终止结果；dry run、queued/accepted 与历史 receipt 不算通过。
- [ ] 当前工具与 Jenkins 的适用接受门分别记录，没有将本地结果升级为迁移或里程碑接受。

## 6. M0.2 Runtime 04 migration resolver generation

### 目标

一次安全 inventory scan 发布 immutable `MigrationResolverIndex`，migration resolver 只做 pure lookup；逻辑 locator、root-relative path、physical identity 和 compound sidecar binding 在同一 generation 内一致。

### 实现切片

- [x] 以 [`../zircon_runtime/runtime/04/failure-2026-07-23-asset-migration-indexed-resolver-generation.md`](../zircon_runtime/runtime/04/failure-2026-07-23-asset-migration-indexed-resolver-generation.md) 为唯一 failure owner，重新盘点 `run.rs`、`scan.rs`、`sidecar.rs`、`resolver.rs` 和 `resolver_index.rs` 当前调用图。
- [x] `migration/mod.rs` 已注册并在 crate 内 re-export `resolver_index`；不存在孤立 `mod` 声明。
- [x] 让 scan owner 从已经 canonicalized、拒绝 symlink/reparse 的 regular-file inventory 构建 `MigrationSourceProjection`；禁止 index 自己再次访问文件系统。
- [x] 让 sidecar 解析发布经过验证的 compound binding；重复 registry identity、重复 locator、ambiguous root 和 missing source 返回现有 typed error/issue kind。
- [x] `MigrationResolver::new` 当前只接收 `AssetRegistryIndex` 与 `MigrationResolverIndex`；per-reference roots/FS fallback 已删除。
- [x] 扩充 `resolver_index.rs` 测试：唯一 source、compound zmeta hint、duplicate registry 优先级、ambiguous source、missing source、跨 root 相同相对路径、link/reparse 拒绝和 index generation 一致性。
- [x] 添加源码 guard，阻止 migration resolver 恢复 filesystem fallback 或第二次 root scan。

M0.2 current-source handoffs:

- fixed 已修复：[deferred-lighting-cache-test-hard-cut](00/fixed-2026-07-28-deferred-lighting-cache-test-hard-cut.md)
- Runtime 04: [`asset migration streaming transaction journal`](../zircon_runtime/runtime/04/failure-2026-07-22-asset-migration-streaming-transaction-journal.md).

M0.2 continues from the lowest passing test target while these owners complete their declared upward validation.

### 测试阶段：M0.2 Runtime 04 Resolver Gate

- [ ] 在 Windows 原生验证 lane 中先运行 `zircon_runtime` package check；按当前独立入口及适用 Jenkins 门保留实际结果。
- [ ] 运行 migration resolver/index/source-boundary focused tests，并包含所有新增 typed failure 分支。
- [ ] 运行 Runtime 04 asset migration parent batch，确认 scan、sidecar、transaction 和 idempotent migration 未回归。
- [ ] 若上层失败，先回到 index/inventory generation；不得修改 Editor/App 适配失败的 resolver 语义。

### 退出证据

- [ ] `zircon_runtime` 当前源码编译通过。
- [x] migration resolver 不包含 filesystem lookup/fallback，所有查询来自单一 index generation。
- [ ] open failure 已完成 upward validation 并按 canonical failure/fixed 流程返回。

## 7. M0.3 Runtime → Editor → App 编译收敛

### 目标

在同一 source fingerprint 上逐层证明三个根包可检查，为 F0 产品构建提供稳定输入。

### 实现切片

- [x] 生成 validation manifest，按 `zircon_runtime`、`zircon_editor`、`zircon_app` 顺序列出 feature/profile、受影响 contract 和已知 open failure。
- [ ] 每次只处理当前最早编译错误；先判断是否属于本计划已修改边界、现有 owner 变化或新的跨计划 failure。
- [ ] 属于其他 owner 的最低原因写入该 owner 的 numbered failure 目录，并继续不依赖该失败的检查；不得在上层添加兼容 alias 或 feature bypass。
- [ ] 每次根因修复后重新运行最低包 focused check，再向上运行依赖包。
- [ ] 完成 `git diff --check`、触及 Rust 文件的 formatter check 和 source guards。

### M0.3 Validation Manifest

| 顺序 | 包 / profile | 受管 compile gate | 受影响 contract | 已知 open failure |
|---|---|---|---|---|
| 1 | `zircon_runtime` / default | 当前独立入口 `check -p zircon_runtime --locked` | Runtime04 single-inventory indexed resolver；Runtime15 receipt-test hard cut | [Runtime04 indexed resolver generation](../zircon_runtime/runtime/04/failure-2026-07-23-asset-migration-indexed-resolver-generation.md)；[Runtime15 plan-status receipt compile debt](../zircon_runtime/runtime/15/failure-2026-08-02-plan-status-receipt-test-compile-debt.md) |
| 2 | `zircon_editor` / default | 当前独立入口 `check -p zircon_editor --locked` | Runtime facade、RHI neutral presenter、Editor host compile boundary | [Frameworks01 RHI WGPU presenter/backend contract test owner](../zircon_runtime/frameworks/01/failure-2026-08-02-rhi-wgpu-presenter-and-backend-contract-test-owner.md) |
| 3 | `zircon_app` / default | 当前独立入口 `check -p zircon_app --locked` | Runtime/Editor entry composition 与 app startup compile boundary | fresh gate 前没有可复用的 current-source lowest failure；实际诊断按最低 owner 处理并保留归属 |

三个包按同一 current-source manifest 顺序验证；任一相关 compile input 变化都须重新判定受影响证据，不允许用旧 ticket、feature bypass 或上层 alias 继续。本地结果只形成独立命令证据；Jenkins 接受使用核验通过的当前入口、固定 `managed-cargo-check-v2` 模板与精确请求身份，里程碑接受另按验证策略审查完整范围。

### 测试阶段：M0.3 Current-Source Compile Gate

- [ ] 使用当前独立入口完成 Runtime package check 和适用 focused batch。
- [ ] 使用当前独立入口完成 Editor package check 和适用 focused batch。
- [ ] 使用当前独立入口完成 App package check 和适用 focused batch。
- [ ] 三个批次使用兼容 Windows toolchain/profile 与相同输入身份；相关源码变化后更新 manifest 并重跑受影响检查。
- [ ] 核对本任务的 Cargo/rustc 终止与实际回执；foreign jobs/锁和旧队列保持原状，不手工清理。

### 退出证据

- [ ] Tooling、Runtime、Editor、App 在 current source 上通过各自声明的检查与适用接受门。
- [ ] 所有未解决问题已有明确 fixing owner 和 failure 链接，不存在“临时忽略后继续”的未知阻断。
- [ ] F0 可以从相同源码指纹开始产品 profile build。

## 8. 阶段退出清单

- [ ] M0.1、M0.2、M0.3 全部通过各自测试阶段。
- [ ] validator 的 JSON 协议有行为测试，而不只是手工确认。
- [ ] Runtime 04 resolver index open failure 完成架构接线和 upward validation。
- [ ] `zircon_runtime`、`zircon_editor`、`zircon_app` 当前源码包级编译均为绿色。
- [ ] 没有删除、回退或覆盖进入本 Session 前的用户变化。
- [ ] 只写一条本阶段 accepted outcome，不记录每次编译尝试。

## 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|

## Current-source M0.1 recheck (2026-09-19)

本次只刷新 M0.1 的 tooling/protocol 证据，仍不改变本计划的
`in_progress` 状态，也不提前解除 M0.2、M0.3 或 MVP 00 的 gate。

- coordinator smoke 的 `-KernelOnly`、`-JsonClient`、`-StrictJsonParser`、
  `-ValidatorDryRun` 四个 slice 均输出 `PASS`；JSON client 覆盖 cold/warm
  daemon、real launcher、strict JSON 和 clean stop。
- focused Python validation batch 为 `126/126`，`0` failure、`0` error、`0`
  skip，耗时 `364.910s`。
- `validate-matrix.ps1 -DryRun -SkipBuild -SkipTest
  -RunExportPlatformContract -ExportContractPlatform headless` 实际执行到
  headless export contract，输出 `[OK]` 并以 exit code `0` 返回；该批次不启动
  Cargo，也不做 artifact admission。

详细收据见
[`2026-09-19-m0-1-tooling-contract-recheck.md`](00/2026-09-19-m0-1-tooling-contract-recheck.md)。
M0.1 的 current-source acceptance、M0.2 resolver upward validation 和
M0.3 Runtime→Editor→App 受管编译仍待同一 source fingerprint 下完成；当前
unmanaged target artifact 与脏的外部 `E:\Git\zr_vm` 仍是 compile gate 风险。

## Code Review 同步结论 (2026-07-30，2026-08-01 落实)

- JSON readiness 污染、resolver module registration 与 resolver FS fallback 三条旧 blocker 已同步到正文和 checklist，不再作为待实施工作。
- 2026-08-03 Runtime04 review repair 已将 linked current/retired sidecar 决策收回单一 inventory generation，并把 source-presence lookup 收敛为排序后的 O(logN) 查询；1/1k/100k references、1/4 roots 的双向顺序回归已落地，独立复审为 Critical/Important/Minor=`0/0/0`。
- 2026-08-03 M0.1 validation-ticket 删除墓碑协议已落地；focused Python batch 16/16 通过，首轮独立复审 `C0/I2/M1` 的 copy-race 分类、真实 baseline deletion 覆盖与 manifest 输入覆盖均已修复，最终独立复审 `C0/I0/M0`。manifest `d7e44074...` / ticket `00dfbdb27a9e4e85935a5f85a7c4f462` 已受理但无 terminal receipt。
- 计划状态保持 `in_progress`：M0.2 implementation 已完成，manifest `a587940b...` 的 package-check 与 focused tickets 已受理但尚无 terminal receipt；M0.3 的 Runtime→Editor→App current-source compile gate 尚未 GREEN。
- 下游不得继续引用 2026-07-24 的旧 blocker，但也不得仅凭静态修正解除 `blocked_by_00`；解除条件仍是 M0.1 focused、M0.2 resolver batch 与 M0.3 三包受管编译在同一 source fingerprint 下通过并写入 accepted outcome。
