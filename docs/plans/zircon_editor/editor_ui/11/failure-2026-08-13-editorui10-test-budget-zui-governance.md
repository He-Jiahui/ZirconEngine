---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-zui-governance
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor_ui/11
related_code:
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/workbench_primitives
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib ui --locked
---

# EditorUI11: ZUI governance test owners exceed the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/11-zui-suffix-convergence-and-ui-toml-retirement.md`
- 交接原因：这两项覆盖 ZUI asset governance 和 workbench primitive retirement/convergence，属于 EditorUI11 的
  suffix/convergence 责任，不属于 MUI 或通用 audit owner。

## 失败现象与复现证据

准确结构审计报告：`tests/ui/boundary/zui_asset_governance.rs` 为 981 行，
`tests/ui/boundary/zui_asset_governance/workbench_primitives.rs` 为 1174 行；test-budget exemptions 为 0。

## 最低共享层根因

ZUI governance、asset policy 和 workbench primitive retirement 回归累积在两个 flat owner 文件，未按治理规则、
asset fixture 与 primitive cutover 行为拆分为 folder-backed 模块。

## 架构修复验收

- 将两项按独立 ZUI governance/cutover 行为拆为 folder-backed 模块，薄 `mod.rs` 仅挂载，每个 Rust 测试文件
  不超过 800 行。
- 保留 suffix convergence、UI TOML retirement、governance 与 workbench primitive 的覆盖语义，不复制 fixture。
- 不得保留 flat compatibility test、`#[path]` mount 或以 exemption 回避预算。
- 重审计不再报告这两项；全局计数清零后，受管 structure gate 才可 GREEN。

## 禁止临时方案

- 不得把 ZUI 责任回退到旧 suffix/compat 路径，不得提高预算或使用 blanket exemption。
- 不得将无关 MUI/component 或 retained-host 覆盖混入本切片。

## 修复结果与回传

Open state: `源码修复完成，受管 Rust/upward gate 待完成`。稳定 Session 为
`failure-roll-01a07160-editorui11`，保留上方历史路径、行数和既有迁移记录。

- 2026-09-06 复核发现两个根文件仍保留 87/720 行辅助逻辑或合同数据。本轮物理删除这两个
  flat owner，新 `mod.rs` 分别为 29/7 行声明；原有资产头、导入和重复项辅助函数进入现行
  `metadata.rs`，primitive/overlay/shell 合同分别进入唯一的 `*_contracts.rs`。
- 精确所有权转移 `9b88bbbadb2e4c0daa31112f6429a61b` 纳入 22 个路径。直接修改文件中的
  20 个 Rust 测试体经同版 rustfmt 归一后完全一致，9 组合同常量值、3 个合同结构体和 3 个迁移辅助函数
  保持一致。其他 26 个文件的哈希未变；整个治理目录共 91 个测试，测试清单保留不代表动态运行。
- 所有本轮 Rust 输入通过 Rust 2024 格式检查，根文件使用 `skip_children=true` 限定检查范围；
  本轮不重写其他会话的子文件。最大直接修改测试文件为 555 行，三个合同文件为 290/237/186 行。
  当前结构审计为 13 个超预算测试、3 个生产文件、2 个重复测试树、0 豁免，原始两条不在清单中。
- Python 基础设施合约实际执行 11 项并通过（0.034 秒）。共享守卫由原 UI06 Session 维护，
  改为递归扫描规范目录、要求目录非空，并且只排除唯一的根 `support.rs`。直接 Web 合同检查
  已指向资产行为模块和 primitive 合同/行为模块，`node --check` 通过。
- Web 合同脚本在迁移前后均因已删除的
  `zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs` 报 ENOENT；
  该既有 retained-painter 路径漂移未由本切片修复，整条 Web 合同检查未通过。
- 当前文档规范引用已同步，UI05/UI08 所有的文档仍分别归属原 Session；历史带日期证据不改写。
  按用户指令跳过 `zr_vm`，原始 `cargo test -p zircon_editor --lib ui --locked`、受管上行验收、
  独立审查、fixed return、closeout 和企微发送均尚未完成。
- 当前源码快照 `2830` 固定 22 个 UI11 路径（含两个 flat 删除）。受管格式请求
  `failure-roll-01a07160-editorui11-governance-format-20260906-r1` 已取得票据
  `42d6aa5a14c1423a971503fbb430573b`，13 个输入、准入无 blocker、回执 `queued`；
  不将该回执视作动态测试执行通过，后续收取终态时须逐项核对输入哈希。
- 任务边界收取该票据终态：`passed`，实际执行 rustfmt 递归格式与模块解析，退出码 0，
  job `03918eed6bdb489cbd93d9bf5c873c5f`；13 个输入哈希与票据 manifest 一致，
  manifest hash 为 `54d25fd8d42f14d6da4ea1164f8a45e6559cfb85cc86277b2939f4515d7ffe59`。
  票据只覆盖 11 个 Rust owner 的格式门禁，不替代 91 个治理测试的动态 Cargo 回归。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 ZUI governance test-budget handoff | `open` | 从准确 48/0 审计中隔离 2 个 ZUI governance owner，最大项为 1174 行。 | 取得源码 exact lease 后按 cutover 行为 folder-backed 拆分，重跑受管回归和结构审计。 |
| 2026-08-24 | ZUI governance owner split | `code_complete_validation_pending` | RED 审计为 981/1174 行；GREEN owner 行数为 87/190/555/167 与 740/164/120/162，目标两项从审计清单移除，oversized test 总数 33→31；23/23 测试名称保留，rustfmt 与 scoped diff-check 通过。 | 提交 exact snapshot 的受管结构审计与 `zircon_editor --lib ui` gate；仅在 Cargo/upward gate 终态后执行 failure return。 |

### 2026-09-18 structure-audit terminal result

The immutable EditorUI11 structure ticket `a74b6e9ea8f0485fab50f85c9b4f52a3`
reached terminal `passed` with job `6aefbcd492404a189801637cf69b621b`, run
`a74b6e9ea8f0485fab50f85c9b4f52a3`, and exit code `0`; both target owners were
under budget (`target oversized owners: 0/2`). The global 27 oversized owners
remain unrelated. This is structure evidence only; the required Editor Cargo/
