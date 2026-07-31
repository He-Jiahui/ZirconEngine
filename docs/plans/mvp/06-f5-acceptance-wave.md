---
related_code:
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - tools/zircon-session.ps1
  - tools/session_coordinator
  - .github/workflows/ci.yml
  - .github/workflows/profile-feature-contract.yml
  - zircon_app
  - zircon_runtime
  - zircon_editor
related_tests:
  - tools/tests
  - zircon_runtime/src/dynamic_api/session/tests/foundation_render.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
planned_code:
  - tools/mvp/Invoke-MvpAcceptance.ps1
  - .github/workflows/mvp-editor-windows.yml
planned_tests:
  - tools/tests/mvp-acceptance.Tests.ps1
  - zircon_app/tests/editor_mvp_authoring.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/mvp/01-f0-reproducible-bootstrap.md
  - docs/plans/mvp/02-f1-project-and-assets.md
  - docs/plans/mvp/03-f2-scene-runtime.md
  - docs/plans/mvp/04-f3-persistence.md
  - docs/plans/mvp/05-f4-basic-authoring.md
status: blocked_by_f4
gate: F5
last_refined: 2026-07-24
---

# F5 验收波次 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: 使用 `executing-plans` 关闭验收波次；进入测试前使用 `verification-before-completion`、`zircon-dev-validation`、`prefer-windows-validation` 和 `requesting-code-review`。F5 不允许与未完成的 F0-F4 并行。

**Goal:** 在 coordinator 管理的干净 validation copy 上完成一次批量 build/test、创建全新 MVP 项目、执行 F1-F4 产品闭环，并让 runtime/editor 对同一项目连续运行两次，保存结构化诊断和真实视觉证据。

**Architecture:** 一个 acceptance driver 负责 source-bound validation manifest、staging、项目创建、运行次序、超时、证据归档和最终一致性检查；它调用正常产品入口和现有测试，不复制 engine/editor 行为。Windows CI 复用同一最小 smoke contract，防止后续回归。

**Tech Stack:** Windows coordinator validation copy、PowerShell acceptance driver、Cargo workspace/profile gates、staged runtime/editor、PNG/window capture、GitHub Actions Windows runner。

---

## 1. 入口条件

- [ ] F0、F1、F2、F3、F4 子计划退出清单全部完成且各有 accepted milestone 记录。
- [ ] 所有直接阻断 MVP 的 failure 已按 canonical fixed lifecycle 完成 upward validation。
- [ ] F5 Session 已注册为独占 validation lane；其他 Session 不再修改 F0-F4 compile inputs。
- [ ] coordinator 已创建 source-bound validation copy；其 commit/patch/input fingerprint 与待验收源码一致。
- [ ] Windows machine 的 WGPU adapter、显示会话、磁盘空间和批准 target/build roots 满足产品运行要求。

## 2. F5 证据包

每次 acceptance run 生成一个不提交到源码树的 evidence root，至少包含：

```text
manifest.json
build/
  profile-contract-summary.json
  workspace-summary.json
project/
  ZirconMvpFixture/
product/
  editor-create/
  runtime-before-edit-run-1/
  runtime-before-edit-run-2/
  editor-authoring/
  editor-reopen-run-1/
  editor-reopen-run-2/
  runtime-after-edit/
captures/
  runtime-before-edit.png
  editor-before-edit.png
  editor-after-reopen.png
  runtime-after-edit.png
comparison/
  persisted-state-before.json
  persisted-state-after.json
  reopened-state.json
```

`manifest.json` 记录 source fingerprint、toolchain、target、adapter/backend、staging manifest hash、project identity、每个 process exit code、开始/结束时间和 evidence 相对路径。不得把日志全文复制到计划状态表。

## 3. 非目标

- 不在 F5 修新功能；发现行为缺口应退回最低未满足子计划。
- 不把当前共享脏 checkout 当作 clean environment。
- 不要求关闭所有 53 个相关 open failure，只关闭直接阻断 F0-F5 的部分。
- 不用缓存命中、历史 binary、单次成功运行或手工目测代替重复验收。

## 4. M6.1 Acceptance driver 与证据完整性

### 目标

提供可重复运行、失败即停、保留最小证据的 Windows acceptance driver；所有步骤使用同一 source/project identity。

### 实现切片

- [ ] 新建 `tools/mvp/Invoke-MvpAcceptance.ps1`，参数只包括 coordinator validation copy、批准 build/evidence root 和超时；不接受 repo-local target。
- [ ] driver 在开始时验证 F0-F4 status、source fingerprint、clean validation copy、磁盘空间和 WGPU/display prerequisites。
- [ ] driver 为每个子进程设置独立 working directory、stdout/stderr 文件和 timeout；非零退出或 timeout 立即停止上层步骤。
- [ ] driver 创建一次 `ZirconMvpFixture`，后续所有 runtime/editor run 只引用该 canonical root。
- [ ] driver 每个阶段解析结构化 summary，验证 project identity、scene URI、entity identity、transform 和 refs 未被替换。
- [ ] driver 生成 evidence manifest 并校验所有相对路径、hash、exit code、截图尺寸和非空像素。
- [ ] 新建 `tools/tests/mvp-acceptance.Tests.ps1`，使用 fake child processes 覆盖成功、非零退出、timeout、缺文件、hash mismatch、project identity drift 和 evidence cleanup。
- [ ] acceptance driver 不解析 Cargo target 目录猜产物；只消费 coordinator/staging manifest。

### 测试阶段：F5 Driver Contract Gate

- [ ] 运行 PowerShell driver focused tests，覆盖成功和全部 failure branches。
- [ ] 在不运行 Cargo 的 dry-run/fixture 模式验证 manifest schema 和 process ordering。
- [ ] 运行一次只到 prerequisite/staging 检查的 current validation copy smoke，确认不写 repo target。
- [ ] 失败时修 driver/tooling，不修改 engine behavior 让 driver 通过。

### 退出证据

- [ ] driver 对任何 child failure/timeout 返回非零并指出具体阶段。
- [ ] evidence manifest 能检测缺失、空白图、identity drift 和 stale artifact。
- [ ] driver 没有 engine/editor 行为旁路。

## 5. M6.2 Clean batched build 与 focused suite

### 目标

在干净 validation copy 中完成 wave-level workspace/profile build 与 F0-F4 focused regression，且所有结果绑定同一 source fingerprint。

### 实现切片

- [ ] 生成 wave validation manifest：packages、feature profiles、test filters、shared interfaces、product evidence 和已接受 deferral。
- [ ] 使用 validator 完成 profile feature contract；保持 `--locked`。
- [ ] 在 execution-wave 级别运行 workspace build/test；这是允许使用 broad workspace gate 的时点。
- [ ] 运行 F0 tooling/startup、F1 project/asset、F2 foundation render/input、F3 persistence、F4 authoring focused suites。
- [ ] shared DTO/schema/manifest 有变化时增加 RuntimeInterface→Runtime→Editor→App boundary batch。
- [ ] failure 按最低共享层诊断；focused repair 绿色后才重跑 wave batch。
- [ ] 完成 formatter、`git diff --check`、plan output audit 和 source guard。

### 测试阶段：F5 Clean Validation Gate

- [ ] `validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract` 全矩阵通过。
- [ ] `validate-matrix.ps1` 的 workspace build/test wave 通过；若工具将 build/test拆分，二者必须使用同一 managed pool/source copy。
- [ ] F0-F4 focused suites 全部通过，且测试名/数量与 manifest 匹配。
- [ ] validation 完成后 coordinator 中无 active job，Windows 无遗留 Cargo/rustc/product process。

### 退出证据

- [ ] clean validation copy 的 profile、workspace、focused suites 全绿。
- [ ] 没有使用当前 checkout 的历史 target artifact。
- [ ] repaired failures 和 accepted deferrals 均有 owner，不存在未知失败。

## 6. M6.3 两次连续产品运行

### 目标

acceptance driver 从全新项目开始执行完整闭环，并证明连续运行不依赖第一次进程残留状态。

### 执行顺序

- [ ] staged editor 创建 `ZirconMvpFixture`，保存 F1 registry/settings summary 后退出。
- [ ] runtime run 1 打开项目，接受输入、呈现 F2 帧、保存 PNG/diagnostics 后退出。
- [ ] runtime run 2 打开同一未修改项目，重复 F2 断言；与 run 1 比较 project/scene identity 和 draw/light counts。
- [ ] editor authoring run 打开同一项目，走 F4 chain 选择 cube、提交 fixed transform delta、SaveProject、等待 completion 后退出。
- [ ] editor reopen run 1 打开修改后项目，保存 selected entity/Inspector transform/window screenshot 后退出。
- [ ] editor reopen run 2 再次打开同一项目，重复 persisted comparison，证明不依赖上一次 editor state。
- [ ] runtime-after-edit 打开修改后项目，确认 primitive 仍可见、refs 有效并保存最终 PNG。
- [ ] 每次进程结束后检查 process tree、项目目录 rename probe 和 file handle release。

### 测试阶段：F5 Product Acceptance Gate

- [ ] driver 从头执行以上顺序，不跳过已有 evidence 阶段。
- [ ] runtime 两次运行均有非空帧、draw/light/pass/input/teardown 诊断。
- [ ] editor 两次重开均观察相同 persisted entity/transform/refs，且不同于 authoring 前 transform。
- [ ] before/after runtime 帧保持可见 primitive；editor before/after 截图显示同一 entity 和修改后的 X value。
- [ ] evidence validator 检查 manifest、hash、identity、exit code、截图和比较结果。

### 退出证据

- [ ] 同一 canonical project 贯穿 create、render、edit、save、reopen、render-after-edit。
- [ ] runtime/editor 连续运行均不依赖 stale cache、未释放 owner 或历史 process。
- [ ] 结构化 diagnostics 与视觉 evidence 相互一致。

## 7. M6.4 Windows CI 防回归

### 目标

CI 至少保护 editor/runtime profile build 和可自动化的 MVP integration path；本地真实窗口/设备证据仍由 release validation lane 承担。

### 实现切片

- [ ] 新建 `.github/workflows/mvp-editor-windows.yml`，使用 `windows-latest`、stable toolchain 和 locked dependencies。
- [ ] CI 构建 `zircon_app --bin zircon_editor --no-default-features --features target-editor-host` 与固定 runtime desktop binary。
- [ ] 运行 project template/registry、F2 foundation render（adapter 可用时）、F3 roundtrip 和 `editor_mvp_authoring` integration tests。
- [ ] 对 Windows runner 无可用 GPU/interactive desktop 的情况使用明确 skip/fail policy；不得把空白 capture 当作通过。
- [ ] 上传 test summary、diagnostics 和允许生成的 capture artifact；不上传完整 Cargo target。
- [ ] 保持现有 Linux CI/profile matrix，Windows lane 是补充而不是替代。

### 测试阶段：F5 CI Gate

- [ ] 本地 lint/解析 workflow YAML，确认 package/features/test filters 与 F5 manifest 一致。
- [ ] 在 PR/branch 上运行 Windows workflow，所有 mandatory jobs 通过。
- [ ] 人为破坏一个 template ref 或 F4 persisted assertion，确认对应 CI test 会失败后恢复。
- [ ] 检查 artifact retention 不包含 secret、绝对用户路径或超大 target tree。

### 退出证据

- [ ] Windows CI 能发现 editor binary build、template/F2/F3/F4 integration 回归。
- [ ] Linux 既有 CI 未被弱化或删除。
- [ ] GPU/display 无法在 hosted runner 验证的部分明确保留给本地 F5 product gate。

## 8. F5 阶段退出清单

- [ ] M6.1、M6.2、M6.3、M6.4 全部通过。
- [ ] clean validation copy、source fingerprint、staging manifest、project identity 和 evidence manifest 一致。
- [ ] workspace/profile/focused batches 全绿，无未知失败。
- [ ] runtime/editor 对同一项目连续运行两次并干净退出。
- [ ] F4 persisted delta 在 editor/runtime 重开后仍可观察。
- [ ] PNG/窗口 capture 非空、构图有效、无 UI 重叠遮挡关键值。
- [ ] Windows CI smoke 已合入并通过。
- [ ] 所有状态记录遵守一里程碑一条，不在 index 或 session note 复制具体证据。

## 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|

## Code Review 建议 (2026-07-30)

### 与代码现状不符，需修订

- front-matter 的 `planned_code` / `planned_tests` 列出的四个产物在当前源码树中均已存在，不再是「planned」：`tools/mvp/Invoke-MvpAcceptance.ps1`（352 行）、`.github/workflows/mvp-editor-windows.yml`（97 行）、`tools/tests/mvp-acceptance.Tests.ps1`（350 行）、`zircon_app/tests/editor_mvp_authoring.rs`（335 行）都已落地。M6.1「新建 `tools/mvp/Invoke-MvpAcceptance.ps1`」「新建 `tools/tests/mvp-acceptance.Tests.ps1`」与 M6.4「新建 `.github/workflows/mvp-editor-windows.yml`」应改为「核对既有实现是否满足声明的 driver/CI contract（child failure/timeout 非零退出、evidence manifest 校验、identity drift 检测、Windows job 覆盖）」。注意 front-matter 本身不要改（docs 门禁校验路径），此处只提示正文与状态描述需同步。

### 验证缺口

- 因为 acceptance driver、CI workflow 和 authoring integration test 已存在，F5 的真实缺口不是「新建」而是「执行并归档证据」：建议正文明确 M6.2/M6.3 仍需在 coordinator 干净 validation copy 上产出 §2 证据包（PNG/window capture、persisted-state 比较、manifest hash），并核对 `mvp-editor-windows.yml` 是否已在 CI 实跑且对 F2/F3/F4 回归有效。当前无法从静态检查确认这些产物是否已被真实运行过。
