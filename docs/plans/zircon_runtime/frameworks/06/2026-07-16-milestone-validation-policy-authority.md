# Frameworks 06 Milestone Validation Policy Authority

> 本文件记录 `06-development-conventions-and-guardrails.md` 的 docs-only execution-policy maintenance；不替代任何业务里程碑的编译、测试或产品验收。

| 切片 | 完成项目 | 状态锚 | 日期 | 当前状态与证据 |
|---|---|---|---|---|
| M0 / C6 | Track milestone validation policy authority | `frameworks_06_milestone_validation_policy_authority_tracked` | 2026-07-16 | **accepted docs maintenance**。将先前已存在但未跟踪的 46 行 `docs/plans/milestone-validation-policy.md` 纳入版本控制，明确 implementation slice、milestone、execution wave 三种验证单位，保留全部回归断言，并把优化限定为调度与批处理，不允许用 focused gate 冒充 workspace/release pass。当前共享工作树已有 9 份计划文档的 11 个直接链接指向该权威文件；本批只提交政策单源及本记录，不吸收这些文档的 foreign dirty 内容。fresh `python tools/check_conventions.py --only docs --json` 保持全局预期 RED 218 missing / 33 affected docs，exact scope violation 为 0；plan-output audit、scoped `git diff --check` 与 policy invariant guard 通过。独立审查首轮发现并修正 2 Important（slice exception 表述冲突、accepted 记录仍含待补占位），修正后 final review 为 0 Critical / 0 Important / 0 Minor。本批不运行 Cargo，也不声明 Frameworks06 M1 或全局计划完成。 |

## 精确范围

- `docs/plans/milestone-validation-policy.md`
- `docs/plans/zircon_runtime/frameworks/06/2026-07-16-milestone-validation-policy-authority.md`

## 边界

- 政策文件只定义何时批量验证，不降低各编号计划原有验收范围。
- 业务里程碑仍必须使用 coordinator native `milestone prepare/validate/review/commit`，不得由 generic finalize 代替。
- 其余引用该政策的 modified plan/index 继续由各自 owner 提交，本 manifest 不代为吸收。
