---
related_code:
  - tools/check_conventions.py
  - tools/check-conventions.ps1
  - tools/tests/test_check_conventions.py
  - .github/workflows/ci.yml
implementation_files:
  - tools/check_conventions.py
  - tools/check-conventions.ps1
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/zircon_runtime/frameworks/development-conventions.md
tests:
  - python -m unittest tools.tests.test_check_conventions -v
  - pwsh -NoProfile -File tools/check-conventions.ps1 -DryRun -Json
doc_type: module-detail
---

# Convention Gate Runner

`tools/check-conventions.ps1` 是 Windows 本地入口，调用 `tools/check_conventions.py` 聚合 Frameworks 06 的基础规范门。它不修改源码或文档，只执行检查并以进程退出码表达结果。

## Owners and boundaries

- `check_conventions.py` 唯一拥有命令计划、文档 front matter 解析、仓库相对路径验证与机器报告组装。
- `check-conventions.ps1` 只把 PowerShell 参数翻译给 Python，不复制规则或重新解释结果。
- `.github/workflows/ci.yml` 只调用聚合入口，不复制或改写 fmt、clippy、docs 命令计划。
- 工具不会自动删除过期引用、创建缺失文件或维护历史 allowlist；路径债务必须回到对应文档 owner 做真实硬迁移。

## Gates

| Gate | Contract |
|---|---|
| `docs` | 扫描 `docs/**/*.md` YAML front matter 中的 `related_code` 与 `implementation_files`；拒绝绝对路径、仓库逃逸和不存在路径。重复声明路径复用一次文件系统判定，但每个文档声明仍单独进入违规清单。 |
| `fmt` | 固定执行 `cargo fmt --all --check`。 |
| `clippy` | 固定执行 `cargo clippy -p zircon_runtime_interface -p zircon_app --all-targets --no-deps --locked -- -D warnings`，只把 Frameworks 06 M1 声明的首批零警告包提升为错误；依赖仍会正常编译，但尚未进入零警告名单的 `zircon_runtime` 警告不在本门中升级。 |

默认执行全部三门；`-Only docs` 可选择单门，`-DryRun` 只阻止 Cargo 命令执行，文档审计仍会真实运行。`-Json` 输出完整报告，顶层 `passed` 为全部已选门的合取结果；任一门失败时进程返回非零。docs 报告同时给出 `affected_document_count`、`reason_counts` 与按违规数量降序排列的 `path_root_counts`，用于把完整明细路由到实际 owner，不改变逐条违规和退出码语义。

## Current acceptance state

2026-07-13 全库 docs 审计已收敛为 GREEN；期间捕获的活动 AI hard-cutover source/doc owner 漂移已由对应 owner 同步到真实新路径。精确快照只记录在 Frameworks 06 编号产出归档中，避免模块说明复制会持续变化的当前计数。`.github/workflows/ci.yml` 的既有 `rust` job 在 workspace build/test 前先执行 runner 契约测试，再只调用 `python tools/check_conventions.py --json` 这一聚合入口。workflow 复用同一 Linux 依赖、Rust toolchain 与 Cargo cache，不复制 fmt/clippy 参数；命令计划仍由 `check_conventions.py` 唯一持有。

该状态不等同于 Frameworks 06 M1 已完成：本地 fmt 与 scoped clippy 已有 testing-stage 通过证据，但真实分支 CI 仍需实际执行。详细切片与测试结果继续由 `docs/plans/zircon_runtime/frameworks/06/` 编号产出归档唯一持有。

## Validation

```powershell
python -m unittest tools.tests.test_check_conventions -v
pwsh -NoProfile -File tools/check-conventions.ps1 -Only docs -Json
pwsh -NoProfile -File tools/check-conventions.ps1 -DryRun -Json
```

测试覆盖缺失路径、有效文件与目录、绝对/逃逸路径、稳定的 fmt/clippy 命令计划，以及 CI 必须在 `rust` job 的 workspace build 前通过唯一聚合入口接线、该步骤不得带条件禁用或 `continue-on-error`、workflow 不得复制命令计划的契约。docs 门当前预期返回零；任何新悬空路径都必须令本地入口与 CI 同时返回非零，不得增加 allowlist、兼容路径或将 RED 转换为成功退出码。
