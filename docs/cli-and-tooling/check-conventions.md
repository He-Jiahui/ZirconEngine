---
related_code:
  - tools/check_conventions.py
  - tools/convention_exemptions.py
  - tools/check-conventions.ps1
  - tools/tests/test_check_conventions.py
  - tools/tests/check_conventions/document_paths.py
  - .github/workflows/ci.yml
implementation_files:
  - tools/check_conventions.py
  - tools/convention_exemptions.py
  - tools/check-conventions.ps1
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/zircon_runtime/frameworks/development-conventions.md
tests:
  - python -m unittest tools.tests.test_check_conventions -v
  - tools/tests/check_conventions/document_paths.py::DocumentPathAuditTests
  - pwsh -NoProfile -File tools/check-conventions.ps1 -DryRun -Json
doc_type: module-detail
---

# Convention Gate Runner

`tools/check-conventions.ps1` 是 Windows 本地入口，调用 `tools/check_conventions.py` 聚合 Frameworks 06 的基础规范门。它不修改源码或文档，只执行检查并以进程退出码表达结果。

## Owners and boundaries

- `check_conventions.py` 唯一拥有命令计划、Windows managed-Cargo 委派判定、文档 front matter 解析、仓库相对路径验证、规范总纲 MUST 守卫勾稽与机器报告组装。
- `convention_exemptions.py` 独占 Rust `allow` lexical scanner、workspace source inventory 与 exemption 趋势/违规统计；runner 只把同轮规则目录审计结果传入，不复制扫描逻辑。
- `check-conventions.ps1` 只把 PowerShell 参数翻译给 Python，不复制规则或重新解释结果。
- `.github/workflows/ci.yml` 只调用聚合入口，不复制或改写 fmt、clippy、docs 命令计划。
- 工具不会自动删除过期引用、创建缺失文件或维护历史 allowlist；路径债务必须回到对应文档 owner 做真实硬迁移。

## Gates

| Gate | Contract |
|---|---|
| `docs` | 扫描 `docs/**/*.md` YAML front matter 中的 `related_code`、`implementation_files` 与 `tests`。前两者逐条视为 owner 路径；`tests` 只审计整条 scalar 构成的具体仓库文件，可带 `::test_name` 或 `:line[:column]`，命令、glob、模板、远程 URI 及 `target`/`build` 产物不作为 owner。具体路径统一拒绝绝对路径、仓库逃逸和不存在路径；重复声明路径复用一次判定，普通 leaf 按父目录复用最终路径，带 `..` 的声明和 symlink/junction leaf 仍完整解析。每个文档声明仍单独进入违规清单，报告的 `resolution_metrics` 给出唯一声明、full resolution 及三类解析计数。 |
| `guards` | 扫描 `development-conventions.md` 的规则表；每个表块必须遵循唯一 header、唯一 separator、连续 data row 顺序，逐行拒绝重复表标记、malformed row、空规则正文、表外规则行、重复 rule ID、MUST 空守卫和任意级别的未知非空 guard，输出有序 rule/MUST ID、数量与按受控 guard 词表汇总的计数。SHOULD 可不指定 guard；一旦指定也必须来自受控词表。新增规则或 guard 类别必须同批更新 runner 契约，单行损坏不得静默缩小审计面。 |
| `exemptions` | 从根 `Cargo.toml` 读取并按 resolved member identity 盘点 workspace Rust 源码中的 `#[allow]` 与 crate-level `#![allow]`。lexical scanner 排除字符串、字符与嵌套注释中的伪属性/伪 marker，并覆盖同一行或跨行 token；首批严格成员 `zircon_app` 与 `zircon_runtime_interface` 的每个属性必须紧邻一行真实 `// EXEMPT(<已知 MUST 规则ID>): <非空理由>`。缺失、格式错误、未知/SHOULD 规则、空理由、规则目录损坏或严格成员缺失均阻断。真实 Git 仓通过 `git grep --untracked --exclude-standard` 同时覆盖 clean/dirty tracked 与非忽略 untracked 源，忽略项不进入库存；Git inventory 故障返回结构化 RED，只有无 `.git` 的 isolated fixture 才使用 Cargo-root fallback，fallback 同时盘点标准 Cargo roots、显式 build/lib/bin/example/test/bench 路径，并按最长 resolved member root 归属嵌套 member。报告按成员和规则统计全 workspace `allow`/有效豁免，并把尚未纳入严格成员的属性显式列为 `unscoped_allow_attribute_count`，不维护历史 allowlist，也不把趋势库存冒充合规。 |
| `layering` | 运行 Frameworks 05 production-only 域引用与 module-identity 契约测试，作为 G1 的常驻 Python 层向门；runner 只调用现有审计 owner，不复制引用扫描规则。 |
| `structure` | 固定执行 `cargo +1.94.1 test -p zircon_runtime --lib structure_convention --locked --jobs 1`。Linux CI 或已处于 managed target/scratch/cache 环境时直接执行；普通 Windows 本地入口通过 `rustup run 1.94.1` 委派给 `validate-matrix.ps1 -SkipBuild -SkipTest -RunConventionStructure`，由协调器分配同一兼容热池。 |
| `fmt` | 固定执行 `cargo +1.94.1 fmt --all --check`。 |
| `clippy` | 固定执行 `cargo +1.94.1 clippy -p zircon_runtime_interface -p zircon_app --all-targets --no-deps --locked --jobs 1 -- -D warnings`。Windows 本地通过 validator 的 `-RunConventionClippy` 精确门进入 managed check lane；依赖仍会正常编译，但尚未进入零警告名单的 `zircon_runtime` 警告不在本门中升级。 |

默认执行全部七门。Python 入口的 `--only <gate>` 可重复指定；PowerShell wrapper 的 `-Only` 接受一个 PowerShell 数组，不能重复写参数名。`pwsh -File` 调用适合单 gate，多 gate 必须在 PowerShell 表达式中用单个数组参数调用，例如 `pwsh -NoProfile -Command "& './tools/check-conventions.ps1' -Only structure,fmt -DryRun -Json"`。Rust 子门在唯一命令计划内固定 toolchain，编译型 structure/clippy 同时固定单 job，调用方不得通过外部默认 toolchain 或并行度改变验收口径。Windows runner 只有在 `CARGO_TARGET_DIR`、job scratch `TEMP`、shared `SCCACHE_DIR`、`CARGO_INCREMENTAL=0` 与 `SCCACHE_CLIENT_SIDE=1` 同时满足 managed-storage 合同时才允许直接 Cargo；否则自动委派，避免仓库 `target`、重复依赖编译和绕过 FIFO。`-DryRun` 仍真实执行进程内的 `docs`/`guards`/`exemptions` 审计，但只规划 `layering`/`structure`/`fmt`/`clippy` 子进程，不启动 Python 或 Cargo 子门。`-Json` 保证 stdout 只有一个完整 JSON 对象；子进程输出被捕获到对应 `commands[].stdout/stderr`，不会污染机器读取面。顶层 `passed` 为全部已选门的合取结果，命令的真实 `exit_code` 同步进入报告；若命令无法启动，则保留 `exit_code: null` 并在 `launch_error.kind/message` 记录确定性失败。任一门失败时 Python 与 PowerShell 入口都返回非零。docs 报告同时给出 `affected_document_count`、`reason_counts` 与按违规数量降序排列的 `path_root_counts`，用于把完整明细路由到实际 owner，不改变逐条违规和退出码语义。

## Current acceptance state

2026-07-13 全库 docs 审计曾收敛为 GREEN；后续并行 hard-cut 又产生新的 current-owner 路径债务，因此默认全门运行当前仍会被 `docs` 门正确阻断，不能把历史 GREEN 当作 current acceptance。精确、会持续变化的违规计数只记录在 Frameworks 06 编号产出归档中；本模块说明只固定“债务未归零即 RED”的契约。`.github/workflows/ci.yml` 的既有 `rust` job 在 workspace build/test 前先执行 runner 契约测试，再只调用 `python tools/check_conventions.py --json` 这一聚合入口。workflow 复用同一 Linux 依赖、Rust toolchain 与 Cargo cache，不复制 fmt/clippy 参数；命令计划仍由 `check_conventions.py` 唯一持有。

当前源码已把 G1 production 层向、G2 Rust structure filter 与 MUST 守卫勾稽纳入同一个 runner 命令计划；CI 继续只调用 runner，因此没有第二份参数或扫描规则。该实现状态不等同于 Frameworks 06 M1/M2 已完成：本地 fmt、scoped clippy 与历史结构门证据不能替代本切片的 fresh managed testing stage，真实分支 CI 也仍需实际执行。详细切片与测试结果只在里程碑通过后由 `docs/plans/zircon_runtime/frameworks/06/` 编号产出记录持有。

## Validation

```powershell
python -m unittest tools.tests.test_check_conventions -v
pwsh -NoProfile -File tools/check-conventions.ps1 -Only docs -Json
pwsh -NoProfile -File tools/check-conventions.ps1 -Only guards -Json
pwsh -NoProfile -File tools/check-conventions.ps1 -Only exemptions -Json
pwsh -NoProfile -Command "& './tools/check-conventions.ps1' -Only structure,fmt -DryRun -Json"
pwsh -NoProfile -File tools/check-conventions.ps1 -DryRun -Json
```

测试覆盖缺失路径、有效文件与目录、绝对/逃逸路径、命令/glob/模板/远程 URI/构建产物排除、同父目录解析规模、相对路径段以及 symlink/junction leaf 与 parent 逃逸；Windows reparse 属性回归锁定 CI Python 3.11 可用的 `lstat` 合同，不依赖 3.12 才提供的 `Path.is_junction()`。其余覆盖包括精确 63 条 rule/49 条 MUST ID 清单、MUST 空守卫、SHOULD 未知非空 guard、缺失或重复 separator、重复 header、空规则正文、malformed/表外规则行、重复 rule ID、Rust 豁免的有效/缺失/未知/空理由与未收紧成员趋势统计、clean/dirty tracked 和非忽略 untracked 清单、显式 Cargo roots 与嵌套 member owner、子门 stdout/stderr 捕获、启动失败与非零退出传播、PowerShell `$LASTEXITCODE` 转发、PowerShell 多 gate 单数组参数文档契约、稳定的七门计划，以及 CI 必须在 `rust` job 的 workspace build 前通过唯一聚合入口接线、该步骤不得带条件禁用或 `continue-on-error`、workflow 不得复制命令计划的契约。`guards` 与 `exemptions` 进程内门当前预期返回零；`docs` 门在所有 owner 完成真实路径硬切前保持 RED。任何新悬空路径、损坏规则行或未勾稽 MUST 都必须令本地入口与 CI 同时返回非零，不得增加 allowlist、兼容路径或将 RED 转换为成功退出码。`layering` 与 `structure` 的执行结果必须来自里程碑测试阶段的 managed validation，不得用 `-DryRun` 计划冒充通过。
