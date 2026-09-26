Plan: docs/plans/zircon_tooling/source_comment_audit/01-repository-source-comment-audit.md
Milestone: M1
Status: completed
Files: ["docs/plans/zircon_tooling/source_comment_audit/01/2026-09-26-m1-comment-standard-and-inventory.md"]

# M1 注释规范与初始覆盖清单

## Scope Delivered

`982f8a70b33c6cdbb8b2b62a4e73dca07c530223` 已按精确快照提交注释规范、初始清单、覆盖检查工具及其测试。初始清单记录 37,519 个路径，其中 26,771 个纳入审查，10,748 个按明确理由排除。清单记录的是待审范围，不代表这些源码已完成调用链审查；其中 36 个纳入路径在建立清单时已有工作区删除，31 个可归属其他 Session，另 5 个归属待确认，后续须协调并复核。

规范要求先追踪定义、调用方、动态入口、测试及设计前提，再决定是否注释；有充分证据的缺陷使用 `BUG:`，待证实的问题使用 `TODO:`，并与问题账本互相对应。覆盖工具按当前文件哈希核对每份审查记录，并在全仓验收时核对问题标签和账本。文件哈希本身不能证明调用方仍然一致，最终阶段还须人工复核受调用方变化影响的记录。

## Fresh Testing Evidence

协调器隔离验证票据 `e02801eba93b423492a67ddd5e63da5e` 对四个已提交文件的精确哈希运行 `python -m unittest tools.tests.test_source_comment_audit -v`，12 项测试通过。`py_compile`、目标文件空白检查及规范文档的目标 frontmatter 路径检查通过。全局文档惯例检查存在既有外来违规，未将其计作本里程碑通过证据。候选 `ad1b63ba3c4b42528296787e3d29a3e2` 已由协调器集成，提交仅含上述四个目标文件。

首部的 `completed` 指实现与聚焦测试已完成；协调器的正式 milestone validation 和独立 reviewer gate 尚待终态，不能据此认定 M1 已被接受。

| 里程碑 | 阶段 | 状态 | 证据 |
|---|---|---|---|
| M1 | M1-T testing | passed | 隔离验证票据 `e02801eba93b423492a67ddd5e63da5e`；12 项定向测试通过 |

## Review

独立复核发现并促成修正三项覆盖验收风险：排除项转入纳入范围时的假通过、UTF-8 前缀截断造成的错误排除，以及扫描后标签变更竞态。修正后的复核结论为 critical 0、important 0；剩余两个 minor 项是逐行词法标签扫描可能误认多行字符串，以及损坏清单在 `mark` 时的错误提示不够友好。后续每批仍需人工确认标签的语法位置和调用证据。
