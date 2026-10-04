---
handoff_kind: failure
status: open
created_at: 2026-07-12
summary_slug: project-asset-reference-full-gate-regressions
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_editor/editor/10
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/project
  - zircon_editor/src/core/project/tests/boundary.rs
  - zircon_editor/src/tests/host/asset_references.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup
  - zircon_editor/src/tests/workbench/project
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests -TestFilter project_authority_core_has_no_ui_dependency_or_retired_template_generator -TestThreads 1 -VerboseOutput
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests -TestFilter bootstrap_and_startup -TestThreads 1 -VerboseOutput
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests -TestFilter project -TestThreads 1 -VerboseOutput
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests -TestFilter asset_references -TestThreads 1 -VerboseOutput
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests -TestThreads 1 -VerboseOutput
---

# Editor 10：当前全量门 ProjectAuthority / AssetRef 回归

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：Editor03 / Editor08 M1 统一行为门
- 修复责任计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 交接原因：工程与资产引用业务不属于命令注册或事务内核；必须由 Editor10 在唯一 ProjectAuthority / AssetRef 权威上精确归因。

## 失败现象与复现证据

Windows 受管 job `520d85713df249afae31661a7697ad07` 使用 `cargo test -p zircon_editor --lib --locked --jobs 1` 完成编译并进入测试后，至少 10 个 ProjectAuthority / AssetRef 归属用例失败：工程边界守卫、场景/动画/物理引用追踪、创建与打开、损坏 workspace 回退、preset 文件往返、welcome 回退，以及工程文档 roundtrip/renderable-template scaffold。原始失败名保存在 `D:/cargo-targets/editor08-m1-rerun4-20260712.log`。

测试 harness 随后在全包资源停滞中被终止，未生成逐项 panic summary；因此本记录不臆测产品实现与旧断言谁错误。命令 registry、when/palette 与事务引擎专属用例在同一 binary 中已通过，不能把本组失败归到 Editor03/08。

## 最低共享层根因

当前可证实的最低归属是 Editor10 的 `ProjectAuthority`、工程文档/模板和 AssetRef/reference tracking 合同。精确根因须由功能 owner 以 fully-qualified exact 单线程复现取得 assertion/typed error 后再细分；不得在上层命令或宿主测试中吞错。

### 2026-07-12 22:44 精确复现补充

Editor 15 将当前 test binary 按顶层 owner 拆分后，`core::` 65 tests 得到 64 passed / 1 failed。
唯一失败为
`core::project::tests::boundary::project_authority_core_has_no_ui_dependency_or_retired_template_generator`：
`zircon_editor/src/core/project/tests/boundary.rs` 递归扫描 `core/project/**/*.rs` 时没有排除自身，导致测试源码
中声明 forbidden needles 的字符串字面量被当成 6 个生产违规：`crate::ui`、`super::ui`、
`DEFAULT_PBR_WGSL`、`DEFAULT_CUBE_OBJ`、`library_root(`、`runtime_cache_root(`。

这提供了此前缺失的 exact panic summary。Editor10 owner 应让结构守卫只检查生产源码，或以不自命中的
结构化方式定义规则；不得删除真实禁用词检查，也不得恢复退休路径。

## 架构修复验收

- 分别运行 project boundary、bootstrap/startup、workbench project roundtrip 与 asset references exact 组，记录每项真实 assertion/typed source。
- 保持 `ProjectAuthority`、manifest v2、`.zircon/` 与 GUID-first `AssetRef` 单一权威；如断言仍要求退役路径/DTO，应硬切测试而不是恢复兼容面。
- 聚焦组全绿后重新运行 `cargo test -p zircon_editor --lib --locked --jobs 1`，并确认 Editor03/08 gate 不再被本组阻断。

## 禁止临时方案

- 禁止恢复旧 UI startup project DTO、`library/`、`.zircon-cache`、路径 ID fallback 或第二份 asset reference registry。
- 禁止批量忽略失败、把 typed error 改成空成功，或在 Editor08 command handler 中复制工程业务逻辑。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| Editor10 M1/M2 / Editor03+08 M1 | ProjectAuthority / AssetRef 当前全量门回归 | `open-待功能owner精确复现` | 2026-07-12 | job `520d85713df249afae31661a7697ad07` 完成编译并复现至少 10 个 project/reference 失败；原始日志 `D:/cargo-targets/editor08-m1-rerun4-20260712.log`，全包因线程/资源停滞未形成 panic summary。 |
| Editor10 M1/M2 / Editor15 M1 | ProjectAuthority core boundary exact failure | `open-已取得精确根因` | 2026-07-12 | 当前 Editor test binary 的 `core::` 分片 64/65；唯一失败为 boundary guard 扫描自身并命中 6 个 forbidden needle 字面量。 |
| Editor10 M1/M2 / Editor09 M1 | 当前源码完整门停滞前复现 | `open-继续由功能owner处理` | 2026-07-13 | job `e81ed19d256f40c28ddb2437e9a18460` 再次记录 ProjectAuthority boundary、asset references 与两个 workbench project roundtrip 失败；asset-reference exact 为 `left=[]`、`right=6 GUIDs`。日志 `.codex/tmp/editor09-m1-full-lib-test-r2-20260713.log`；不在 Editor09 复制 ProjectAuthority/reference registry。 |
| Editor10 M1/M2 | 当前源码功能修复 | `open-实现与静态门完成-独立复审/受管行为门待执行` | 2026-07-17 | boundary guard 已排除自身测试目录且保留全部生产禁用词；ProjectAuthority generation、document roundtrip 与 AssetRef consumer 当前源码已收束，详见 `2026-07-17-project-authority-reference-regression-closeout.md`。未取得 current-source exact Cargo 输出前不改 fixed。 |
| Editor10 M1/M2 | clean-HEAD 不可变完整门 | `open-目标测试未执行-上层编译阻断` | 2026-08-05 | 协调器 validation-copy job `ac4153eb48e74c5bb47194928a5772f8`、run `459c4c003f4140e5babeb8c04a8d3b8f` 固定 HEAD `94cf43ec3` 与输入哈希 `e19f08dcdad2a54765b4795551cc5998a23ba1b8c9d1559dcc3a0a50c5824346`，显式包含 17 个 renderable-empty 模板和 pinned `zr_vm` 外部源；Runtime 成功编译后，Editor test binary 因 686 个无关编译错误退出 `101`，`0 tests`。首个 `project` 聚焦组未执行，后续 `asset_references` 与全量门被 `&&` 正确阻止；因此不执行 `failure return`。 |

## 修复结果与回传

- 状态：`open / 待修复`；Editor10 完成 exact 归因和修复后在本文件回填验证，并向来源 Editor08 回传。
- 2026-08-05 的不可变副本已排除模板遗漏、外部本地依赖和共享工作树脏改动三类环境干扰；
  剩余阻塞是 clean HEAD 的 Editor test binary 编译失败。声明的 project/reference 行为门仍未开始，
  本 handoff 保持 `open`。

### 2026-09-24 当前源码与验收路径复核

- 将已不存在的 `tests/host/manager/bootstrap_and_startup.rs` 修正为当前 `bootstrap_and_startup/` 模块目录，补充可定位的 `core/project/tests/boundary.rs`；这仅修正文档索引，未接管或修改外部会话正在编辑的测试源码。
- 前置原始命令 `cargo test -p zircon_editor --lib --locked project -- --test-threads=1`、`cargo test -p zircon_editor --lib --locked asset_references -- --test-threads=1` 与历史全包复现记录不变；上方 `tests` 改列当前 Windows 受管聚焦、原始两组过滤和完整 lib 门。须从真实日志确认 boundary、bootstrap/startup、workbench roundtrip、asset references 原失败目标分别执行且通过，不能把过滤词命中其他测试或零执行当作验收。
- 当前 boundary 源码的扫描器已排除测试路径，属于已有源码修复待当前快照动态验证；旧 full-gate 的 `686` 个编译错误、`0 tests` 不构成任何行为通过证据。外部 Git 源 `E:/Git/zr_vm` 当前存在其他 owner 的未提交改动，且相关测试源码也在他人会话中变更；这些输入未稳定前不封存不可变 Cargo 票据、不提交 `failure return`，本项继续 open。
- 对快照 `3753` 的独立审查 C/I/M 为 0/0/1：受管聚焦命令没有保留原始单线程语义，`project` 过滤器也不足以独立覆盖全部 bootstrap/startup 用例。已核对验证脚本的 `-TestThreads 1` 真正传递 `--test-threads 1`，并补列 `bootstrap_and_startup` 模块过滤及单线程完整门；此修正仍需新快照复审和实际执行证据。
- 对精确快照 `3754` 的再次独立只读审查 C/I/M 为 0/0/0：五条受管命令可定位原始 boundary、bootstrap/startup、project、asset references 与完整 lib 门，脚本的单线程/`--locked` 参数传递、现行路径及原始失败证据均已复核。该审查只覆盖本文档归属修正，尚无本快照的 Cargo 动态执行结果，完整行为门和 `failure return` 继续待验。

## 2026-09-25 current-source rolling reconciliation

- Successor Session `failure-roll-01a084c8-editor10-project-reference-r1` owns only this
  failure/plan record and an exact read-only source manifest. The previous stale primary
  `failure-roll-01a084c8-editor10-ui-asset-reference-r2` was archived through the
  coordinator with no live leases before this successor was admitted; no source ownership
  was silently absorbed.
- The current-source contract probe passed
  `EDITOR10_PROJECT_ASSET_CURRENT_SOURCE_PASS=8/8`: the ProjectAuthority boundary guard
  excludes its own `tests` tree while retaining all six retired-path needles; `OpenedProject`
  is the prepared-project owner; runtime `open_prepared_project`, source-path and asset-URI
  generation queries are present; editor document load/save use the activated
  `ProjectManager`; and layout projection reads the manager-owned URI index. A scoped scan
  also returned `RETIRED_SYMBOL_SCAN=PASS` for `open_project_manager_for_paths`,
  `load_from_path`, and `save_to_path` under the current project/workbench sources.
- Exact-source formatting and whitespace checks are green:
  `rustfmt --edition 2021 --check` over the 25-file manifest exited 0 and scoped
  `git diff --check` exited 0 (only the repository's normal LF/CRLF warnings were emitted).
  The working-tree changes remain pre-existing owner overlays; this session made no Rust
  source edits. Dirty paths are limited to `authority/mod.rs`, the project template and
  AssetRef/bootstrap/workbench fixtures, `layout_persistence.rs`, `project_access.rs`, and
  the activated-document load/save modules; clean paths retain their current hashes in the
  immutable manifest.
- Current manifest SHA-256 values are sealed in authoritative coordinator snapshot `3849` for the following exact
  paths: `authority/mod.rs` (`259892287996a344f9223de25a71427df058412a053af52bd3c822440027a112`),
  `authority/project_authority.rs` (`ebd3a7db6e894e89e7bad79227d02ea4ce0e7933a48ff0168b6beef42d81449b`),
  `authority/open_project.rs` (`ac1ea922dc8a5dda1fa8bde11dc1a00e79c0dff60ab28eea549ad5db054620fe`),
  `opened_project.rs` (`3e7267a68ffe3f5de658916c184d0768468712adf6d430bd750a383e18a82751`),
  `project_probe.rs` (`75419e880f97b361b690692fe9eef635584a7fb05b308e79c55351466aa5774e`),
  `project_access.rs` (`00407aee2010c56bd7c7ac6996b6cb4a7c73ab5c69512ca9c7d889309eebf8db`),
  `layout_persistence.rs` (`6c7f0db2acfac532d901904137d5dc4a0cc365f74b07a74a00eed8f21b68be77`),
  `ui_asset_promotion.rs` (`ab369d084e27a2646976426bf17fed2c8301501f0731010830478486a852c2f6`),
  `editing.rs` (`86740fed25eaa13e85bc068cb6c77cdc8934b0c3dccf00d50036a037558ee4b8`),
  `editing/navigation.rs` (`ec66a871c9dad4b2775089d0570c2919c5b97b903ea2bcad3eeb69125dab68e0`),
  `editing/node_ops.rs` (`96154e7df7e3c6eae3a58208040c89963f32641388c1d149f3741cb9f4bdb0d8`),
  `editor_project_document_load.rs` (`44d6c6b9536c7ac073527605ef2637306c9d6fea4c7c8107a68d3f9cd8e205a5`),
  `editor_project_document_save.rs` (`8d249a43a36eb65bc656371ce7bbc45b8edfcb01ce7191c6f2bafeae4d51fbcb`),
  `layout_preset_assets.rs` (`348cc3db376e6fa5c9a983b83e41ff73742aab904363949d49abc77440a97849`),
  `editor_startup_session_document_welcome_pane_snapshot.rs` (`9a3cec358229ded5869328310aeafda7f65ad527aa5f2859d0e80ff843d2e0b1`),
  welcome `project_probe.rs` (`c77466168ac1bea41cb177356881a0d46491473e25b399835c2db972342fa93f`),
  runtime `asset_manager.rs` (`ac9b4a3f5528a46707cbe3a3234a898339989298bc729b01d4dd0ab79fb800d0`),
  runtime `asset_manager_contract.rs` (`9192f48ed1f9c9fba40b523654c637eaf39437cd6e700bfbe293be268d04da4f`),
  boundary.rs (`ed87d5ab83480fae214a0a9a24af3ba50dc283a24d1531e969fd09c89d3dc0f5`),
  template_creation.rs (`b4e6074c653861a3102e069806506fc9b396b9d07e987aa1462dc51c5117f45f`),
  asset_references.rs (`57c3e81d6d9579253651d952ca6d2bc81b5369784251aec0b469d122ed549add`),
  bootstrap_and_startup/mod.rs (`959581ec013be345fd49718b59a8781ddc3a2d06214af4cc290f00e1ce56518d`),
  project_generation_projection.rs (`08666487ece5624ded96fb8da5dd0f424751af577d3956fc90526e28a75f92cf`),
  document_roundtrip.rs (`e3e842cf071b22d8f2987860d3f3f0ce8218892b1f578dd49b3da56f7152dfae`),
  and renderable_template.rs (`d2c344225c75911f3fce0c00faf1050006df5413a60384a7bb62f8f15f084ea9`).
- This is a current-source/static reconciliation only. The five exact Windows `--locked`
  filters, full Editor lib gate, external `E:/Git/zr_vm` admission, upward product and
  performance gates, independent review, canonical `fixed-*` return, and closeout remain
  pending. The failure therefore remains `open`; no historical or static receipt is reused
  as dynamic acceptance.

## 2026-09-25 independent current-source review

- Reviewer `review-editor03-gizmo-private` re-read authoritative snapshot `3850` and the
  manifest boundary from snapshot `3849`. It confirmed the 8/8 current-source marker,
  retired-symbol scan, all 25 source hashes, dirty-owner provenance, formatter/diff-check
  evidence, and the explicit dynamic-gate pending list.
- Review result: `Critical=0`, `Important=0`, `Moderate=0`. This is a static approval only;
  it does not satisfy the five managed Windows filters, full Editor lib gate, external
  `E:/Git/zr_vm` admission, product/performance evidence, fixed return, or closeout.
- Final wording-only recheck at snapshot `3854` reconfirmed `C/I/M=0/0/0`; it changed no
  source bytes or manifest hashes and treats `3853` as the authoritative queued-ticket
  boundary.

## 2026-09-25 static validation ticket receipt

- Coordinator accepted static source-contract ticket `e071be59ba1446918b8dc5c5ba9dd51d`
  (request `939e46aa045044ad6858f0942d1fa78e0`) against the exact manifest sealed by
  snapshot `3851`. Its command is the 8/8 current-source probe recorded above; admission
  returned `status=queued`, not `passed`, and no Cargo process or test body has run.
- Post-ticket coordinator snapshot `3853` is the authoritative receipt boundary for this
  queued request; it preserves the same 27-file manifest and hashes as `3851` while sealing
  the ticket ID and dependency-blocker text below.
- Admission reported dependency blockers through the fixing-plan graph, including the open
  Runtime04 targeted-import failure and downstream Runtime11/Editor14/Plugins13 chains.
  The external dirty `E:/Git/zr_vm` gate and all five Editor10 Windows filters remain
  independently pending. This receipt is retained for wake-up/retry and is not reused as
  dynamic acceptance; the failure remains `open`.
