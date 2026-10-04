---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-gateway-session
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor/01
related_code:
  - zircon_editor/src/tests/gateway/mod.rs
  - zircon_editor/src/tests/gateway/session
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib gateway --locked
---

# Editor01: gateway session test owner exceeds the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 交接原因：gateway session 覆盖属于 Editor01 kernel/runtime interaction；不得由 EditorUI10 通用 audit 或 Editor02 messaging 吸收。

## 失败现象与复现证据

准确结构审计在 0 个 test-budget exemption 下报告
`zircon_editor/src/tests/gateway/session.rs` 为 1137 行。gateway session 行为属于 Editor01 kernel/runtime
interaction，未拆分会使全局 zero-tolerance structure gate 继续 RED。

## 最低共享层根因

gateway session 生命周期、runtime interaction 和 fixture/assertion 场景集中追加到一个 flat owner，
没有按 session contract 建立 folder-backed 测试边界。

## 架构修复验收

- 按 gateway session 行为拆为 folder-backed tests，薄 `mod.rs` 挂载，所有测试文件不超过 800 行。
- 保留 kernel/runtime interaction、session lifecycle 和错误路径覆盖；共享 fixture 唯一归属。
- 不得保留 flat compatibility test、`#[path]` mount、duplicate tree 或 exemption。
- 重审计不再报告该路径；全局 owner 清零后受管 structure gate 才可 GREEN。

## 禁止临时方案

- 不得提高预算、删除 gateway 覆盖或把 session contract 转给 Editor02 messaging。

## 修复结果与回传

Open state: `source hard-cut complete / managed validation blocked`。旧
`tests/gateway/session.rs` 与 `core/gateway/session.rs` 均已物理删除；测试按构造、frame
demand、output ownership、plugin operations 与 world sync 拆为 folder-backed owner，
`SessionGateway` 生产实现也按 ABI output、protocol、world sync、frame、overlay、profile、
plugin event 和 operation 职责拆入同域目录，没有 compatibility mount、`#[path]` 或
test-budget exemption。

当前 gateway frame 合同不是本 test-budget failure 的修复面：provider-backed frame storage
由返回的 `EditorRuntimeFrame` 保持，并在显式 `release()` 或 drop 时恰好释放一次；
`output_ownership.rs` 直接覆盖该生命周期。2026-08-14 记录中的 host-owned `Vec<u8>` 描述
只反映当时快照，已被后续 gateway 合同演进超越，不得作为当前回传事实。早期
validation-copy compile-time template 缺件同样只保留为历史失败证据；当前仍待执行的是与
现行源码匹配的 gateway Cargo、全局 structure gate 及向上验收。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 gateway session test-budget handoff | `open` | 从准确 48/0 审计隔离 1137 行 gateway session owner。 | 取得源码 lease 后按 session contract folder-backed 拆分，受管 gateway 回归和结构审计复验。 |
| 2026-08-14 | Editor01 gateway session hard cut and frame ownership | `implemented / validation_blocked / historical snapshot` | 删除 flat test/source `session.rs`；`tests/gateway/session/mod.rs` 加六个行为 owner，当时保留 26 个测试，各叶子低于 800 行；`core/gateway/session/` 的 11 个具名 leaf 分离 ABI output、protocol、world sync、frame、overlay、profile、plugin event、operation 与 trait dispatch，最大 188 行。当时快照把 `EditorRuntimeFrame` 改为 host-owned `Vec<u8>`；该 frame 所有权描述后来被 provider-backed release/drop 合同超越，当前事实见下方 2026-09-21 reconciliation。精确 `rustfmt --check`、diff check、静态 hard-cut guard 与独立复审均无 findings；结构审计当时全局为 migration debt 32、oversized production 2、oversized test 28，未将其他 owner 债务计入本项。先前 source manifest `31655886ca6a5cc8fddbca07d666646caa319ddddbb65d38ddd1e28481c1ff95` 的受管 run `d219061daebe4a8e9b31d80c83a0dd15` 于测试前 `exit 101`：validation copy 未物化 `zircon_runtime_interface/templates/projects/renderable-empty/**`，`include_bytes!` 无法编译；该证据已被当前源码硬切超越，不能作验收。 | 以现行源码重建 manifest，运行 gateway focused gate、全局 structure audit 与 frame ownership tests；通过前不得关闭 handoff、提交里程碑或发送完成通知。 |

## 2026-09-21 current-source reconciliation

- Successor Session `failure-roll-01a084c8-editor01-gateway-session-budget-r1` corrected the stale
  `related_code` entry from the deleted flat file to the current parent module and folder-backed
  owner. Snapshot `3717` binds the failure record plus the eight current test/module files.
- The current tree contains six behavior owners plus the thin session `mod.rs`; the largest file is
  `fixture.rs` at 644 lines and the five behavior files now contain 27 tests. The old flat file is
  absent, the parent mounts `mod session;`, and no `#[path]` mount or test-budget exemption exists.
- The current structure audit exits successfully and no longer lists this gateway-session owner.
  It still reports unrelated global debt: 17 migration items, 3 oversized production files,
  12 oversized test files, and 2 duplicate test-tree pairs. Those results are not promoted to a
  global GREEN claim.
- Independent review found one documentation-only Important because the prior current-state text
  still described the superseded host-owned frame contract and 26 tests. The reconciliation above
  fixes that record drift without editing or absorbing the concurrently owned gateway production
  files. A zero-finding re-review is required before return.
- Initial managed static ticket `b4f9ba9e6e364e65b40e1d6612a24737` failed before any source
  assertion because its generated Python checker embedded a newline inside a quoted join string and
  raised `SyntaxError: unterminated string literal`. This is a checker-command failure, not a product
  finding.
- Corrected successor ticket `970d16651bd74d1d939cbea425c6ebfc` resealed snapshot `3719`
  (manifest `71bfccc253ec8387ac2aa909327906056e0305361fc1975006f333ac419dc4e3`).
  Coordinator job `026eecdf7cb84c0c8c20e1e1b6bb9da5` / run
  `970d16651bd74d1d939cbea425c6ebfc` exited 0 with
  `EDITOR01_GATEWAY_SESSION_TEST_BUDGET_CURRENT_SOURCE_PASS`,
  `MAX_TEST_OWNER_LINES=644`, and `GATEWAY_SESSION_TEST_COUNT=27`.
- Corrected independent review is `Critical=0 / Important=0 / Moderate=0`. It confirmed the
  folder-backed test boundary and current provider-owned frame lifetime while preserving the
  unrelated global structure debt. This receipt append changes only the failure record after the
  ticket snapshot; the eight checked Rust files retain their sealed hashes.
- Focused gateway Cargo, the full structure gate after other owners clear their debt, canonical
  fixed return, and closeout remain pending.
