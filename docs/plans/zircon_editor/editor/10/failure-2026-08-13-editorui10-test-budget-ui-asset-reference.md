---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-ui-asset-reference
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor/10
related_code:
  - zircon_editor/src/tests/editing/ui_asset/mod.rs
  - zircon_editor/src/tests/editing/ui_asset/reference_and_promotion/mod.rs
  - zircon_editor/src/tests/editing/ui_asset/reference_and_promotion/reference.rs
  - zircon_editor/src/tests/editing/ui_asset/reference_and_promotion/promotion.rs
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib editing --locked
---

# Editor10: UI asset reference and promotion test owner exceeds the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 交接原因：UI asset reference/promotion 是 Editor10 project and asset reference 管理责任，应与 Editor07 authoring/replay 分离。

## 失败现象与复现证据

准确结构审计在 0 个 test-budget exemption 下报告
`zircon_editor/src/tests/editing/ui_asset/reference_and_promotion.rs` 为 806 行。它覆盖 UI asset reference/promotion，
属于 Editor10 project/asset reference 管理，必须拆分以解除全局 structure gate RED。

## 最低共享层根因

reference resolution、promotion 和相关 fixture/assertion 被累积到 flat 测试 owner，没有按 reference/promotion
行为建立 folder-backed 边界。

## 架构修复验收

- 按 reference 与 promotion 行为拆为 folder-backed tests，薄 `mod.rs` 挂载，测试文件不超过 800 行。
- 保留 project/asset reference 解析、promotion 和错误路径语义；共享 fixture 唯一归属。
- 不得保留 old flat test、`#[path]` shim、duplicate tree 或 budget exemption。
- 重审计不再报告该路径；全局 owner 清零后受管 structure gate 才可 GREEN。

## 禁止临时方案

- 不得提高预算、删除 reference/promotion 覆盖，或将其回归混入 Editor07 的 authoring/replay 责任。

## 修复结果与回传

Open state: `source_split_static_confirmed_validation_pending`。原始 806 行 flat owner 已由现行
`ui_asset/mod.rs` → `reference_and_promotion/mod.rs` → `reference.rs` / `promotion.rs` 挂载替代；
原始失败路径保留在上方复现证据，当前 `related_code` 仅列存在的源码路径。两份行为文件分别为
384/425 行，现行源码保留 7/5 个 `#[test]`；本次只修正 handoff 验收元数据，未改业务测试。
原 owner 的结构验收票据 `835b7acba13b423b96a332f24015adf4` 仍为 queued，不转移票据归属；
尚无匹配当前快照的受管 editing 动态通过记录、全局 structure gate 及向上验收，故不执行 return/closeout。
2026-09-24 对当前源码执行原结构审计命令：不再列出本项原 flat 路径，且 test-budget exemption 为 0；
全局仍有 12 个其他 oversized test 文件、3 个 oversized production 文件及 2 处其他 duplicate test tree，
`m1_gate_status=migration-debt-present`，不能将本项静态消失误记为全局 structure gate GREEN。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 UI asset reference test-budget handoff | `open` | 从准确 48/0 审计隔离 806 行 reference/promotion owner。 | 取得源码 lease 后按 reference/promotion folder-backed 拆分，受管 editing 回归和结构审计复验。 |
| 2026-09-24 | Editor10 现行 owner 核对 | `open / validation_pending` | 原 flat 路径已移除；source owner 以精确内容哈希从 archived session 转移至 `failure-roll-01a084c8-editor10-ui-asset-reference-r2`，当前 4 个 related_code 文件存在且未编辑；原结构审计不再报告本项路径，全局门禁仍 RED。 | 等待原票据及匹配快照的受管 editing/upward 门禁后再回传。 |

### 2026-09-24 独立静态复审

- 对 snapshot `3740` 的 failure 文档及四份现行源码执行只读复审，结果 `Critical=0 / Important=0 / Moderate=0`；复审没有编辑源码或变更协调器。
- 复审独立重跑结构审计，确认本项不再列为 oversized/duplicate，原始复现记录仍保留，12 项行为测试和薄模块挂载仍在；全局结构门禁继续为 `migration-debt-present`。
- 以上仅证明当前静态归属及文档准确性，`cargo test -p zircon_editor --lib editing --locked`、向上受管验收、return 和 closeout 仍待完成。
