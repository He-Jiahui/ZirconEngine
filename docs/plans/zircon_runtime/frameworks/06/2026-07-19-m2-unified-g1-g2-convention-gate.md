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
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python -m unittest tools.tests.test_check_conventions -v
  - python tools/check_conventions.py --only guards --json
  - python -m unittest tools.tests.test_frameworks_05_layer_direction -v
  - cargo +1.94.1 test -p zircon_runtime --lib structure_convention --locked --jobs 1
  - cargo +1.94.1 fmt --all --check
  - cargo +1.94.1 clippy -p zircon_runtime_interface -p zircon_app --all-targets --no-deps --locked --jobs 1 -- -D warnings
---

# Frameworks06 M2 Unified G1/G2 Convention Gate

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M2
Status: implementation_complete_re_review_pending_external_plugins04_managed_compile
Date: 2026-07-19
Updated: 2026-07-29
Session: `frameworks06-m2-unified-g1-g2-convention-gate-r2-20260727`

## 完成项目

- 将 Frameworks05 production-only 层向审计、Runtime `structure_convention`、fmt 与首批 scoped clippy 收敛到 `tools/check_conventions.py` 唯一命令计划；PowerShell 和 CI 只转发，不复制参数或扫描规则。
- `guards` 对总纲规则表逐个表块、逐行验证，强制唯一 header 后立即出现唯一 separator，拒绝重复表标记、malformed/表外规则行、空规则正文、重复 rule ID、MUST 空守卫与任意级别的未知非空 guard；机器报告固定输出有序 `63` 条 rule / `49` 条 MUST ID 清单。
- 表外候选只识别受管的 `GEN` / `RT` / `ED` / `PL` / `IF` / `WF` rule ID 家族；普通 `CI` / `G1` 汇总表不再被误判为规则行，新 rule ID 家族仍须与 runner 契约同批显式登记。
- `--json` 捕获每个子门 stdout/stderr 到 `commands[]`，stdout 只保留一个 JSON 对象；子门真实退出码同时驱动 `commands[].exit_code`、顶层 `passed`、Python main 与 PowerShell 进程退出码，命令启动失败则以 `exit_code: null` 和结构化 `launch_error` 保持同一 JSON 失败面。
- Rust 子门不再继承调用方的默认 toolchain 或编译并行度：唯一命令计划固定 `cargo +1.94.1`，structure/clippy 固定 `--jobs 1`，fmt 使用同一 toolchain；PowerShell 与 CI 仍只转发聚合入口，不复制这些参数。
- 不增加 allowlist、兼容入口、旧命令计划或第二套 G1/G2 规则 owner。

## Fresh Static Evidence

- TDD RED：新增契约首先暴露缺失 `rule_ids/must_rule_ids`、未识别 malformed/unknown guard 与缺失 JSON capture API；2026-07-27 successor 又精确复现普通 `CI` / `G1` 表被报为表外规则行的误伤。独立复审随后暴露 SHOULD 未知 guard 被跳过、header 后缺 separator、空规则正文和命令启动失败逃逸四个缺口；下一轮又复现重复 header、前导双 separator 与 data-row 后 separator 被接受。新增 focused tests 均先 RED，再由显式表状态与结构化 launch error 修复。
- 2026-07-28 toolchain/job 契约先将稳定命令计划 focused test 精确打成 RED，再由 runner 唯一 owner 修复；snapshot1175 复审又发现 CI/PowerShell 去重守卫只识别未固定 toolchain 的 Cargo 命令，synthetic fixture 对三个 `cargo +1.94.1` 分支精确 RED，随后由共享的可选 toolchain 前缀识别修复。snapshot1176 继续精确暴露 PowerShell 参数名不可重复的文档漂移，新增契约先 RED，再将多 gate 调用硬切为单数组 `-Only`。全量 runner `19/19`、guards `63/49/0`、layering `28/28`、py_compile 与 exact-scope diff-check GREEN，Python 重复 `--only` 与 PowerShell 单数组调用均只投影授权 gate。snapshot1208 独立复审 Critical/Important/Moderate/Minor=`0/0/0/0`、Ready；managed Cargo 仍待执行。
- 2026-07-29 fresh 复验再次确认本切片自有 runner `19/19`、guards `63` rules / `49` MUST / `0` violations、py_compile 与 exact-scope diff-check GREEN。首次 Frameworks05 layer-direction 全集因守卫读取 Plugins04 已物理删除的 `animation/scene_hook/sequences.rs` 为 `27/28 + 1 FileNotFoundError`；守卫硬切到 `animation/sequence/apply.rs` 后曾得到 `28/28`，但 snapshot1220 独立复审发现其仍正向匹配无法解析的 `crate::sequence` production caller，因此拒绝该假阳性。契约加入当前 crate-root 正向断言和旧路径负向断言后先精确 RED；Plugins04 caller 最小 hard cut 后聚焦 `1/1` 与完整 `28/28` fresh GREEN。Frameworks05 open handoff 见 [`failure-2026-07-29-animation-scene-hook-guard-stale-path.md`](../05/failure-2026-07-29-animation-scene-hook-guard-stale-path.md)，其下级 Plugins04 受管编译仍待完成。
- `python tools/check_conventions.py --only guards --json`：`63` rules / `49` MUST / `0` violations，exit `0`。
- PowerShell `-Only guards -Json` 可被 `ConvertFrom-Json` 直接解析，exit `0`；默认 `-DryRun -Json` 因真实 docs 债务保持 RED，不吞错。
- `python -m unittest tools.tests.test_check_conventions -v`：`19/19` GREEN；其中新增契约固定 Python 重复 `--only` 与 PowerShell 单数组 `-Only` 的真实调用差异，拒绝重复 PowerShell 参数名的虚假文档契约。
- `python -m unittest tools.tests.test_frameworks_05_layer_direction -v`：2026-07-28 历史 current-source 为 `28/28` GREEN；2026-07-29 仅修复 guard 读取路径后的 `28/28`（122.235 秒）已被 snapshot1220 判定为旧 caller 字符串假阳性，不是 acceptance。caller hard cut 后的当前 fresh 证据为 `28/28` GREEN（120.095 秒）。
- 计划记录写入前的全库 docs 诊断为 `512` violations / `144` documents / `67,126` checked paths；写入后复验因并发文档变化为 `515` / `147`，而本记录、父计划与 runner 模块文档 focused 均为 `0` violations。全局数字只作为这两次运行的时序证据，不固化为持续 current fact；外部 G7 债务继续开放，也不计为本 runner 的通过证据。

## Immutable Review

- 初始 snapshot568 exact4，ordinal fingerprint `3abbe16f405ac8bd414e99a66ea35c835b3a104390e5fbb12dce82d1bd19fcd2`；独立复审为 Critical 0 / Important 3 / Minor 0，拒绝静默漏规则、JSON stdout 污染和失败传播缺口。
- 修复 snapshot589 后复审为 Critical 0 / Important 1 / Minor 0，仅剩表外规则行分支未被 synthetic fixture 实际触发。
- 最终 snapshot591 exact4，ordinal fingerprint `0eef1a64d25f1ff7e94b2f58cbaf392699fc263562009d80d9f0ad07587c2fd8`；双遍复审 Critical 0 / Important 0 / Minor 0，interpass drift `none`，Ready。
- 2026-07-27 successor 的表外候选误伤修复改变了 runner/test/记录内容，以上 snapshot591 只保留历史证据；当前 exact5 必须重新生成不可变快照并完成独立复审，未复用旧 accepted review。
- successor snapshot1161 的独立复审为 Critical 0 / Important 3 / Moderate 1 / Minor 0，拒绝上述 SHOULD guard、表结构、launch error 与状态记录矛盾；这些问题已完成 TDD 修复，但尚未生成修复后快照或复审结论。
- snapshot1163 的修复后复审为 Critical 0 / Important 1 / Moderate 0 / Minor 0，确认上轮四类问题均关闭，但拒绝重复 header/separator 仍可穿过状态机；新回归已先 RED 后修复，1163 因源码变化失效。
- snapshot1164 的复审继续为 Critical 0 / Important 1 / Moderate 0 / Minor 0，确认 pending/data 阶段的重复 separator 已拒绝，但 data-row 后的新 header 仍可被误作新表；第四种序列已加入同一参数化回归并先 RED 后修复，1164 因源码变化失效。
- snapshot1165 exact5 的最终双遍复审为 Critical 0 / Important 0 / Moderate 0 / Minor 0，interpass drift `none`，Ready；`17/17`、`63/49/0`、四种损坏表序列和两个合法分隔表块均由审阅方独立核对。该结论已落盘，1165 随本记录状态更新转为历史 accepted evidence，提交仍须使用 successor manifest。
- snapshot1166 exact5 在 toolchain/job 契约修复前完成独立复审 Critical 0 / Important 0 / Moderate 0 / Minor 0；本次 runner/test/docs/记录变更使其转为历史证据，managed Cargo 与提交必须使用新的 successor snapshot 和复审，不能复用 1166。
- snapshot1175 exact5 独立复审为 Critical 0 / Important 1 / Moderate 1 / Minor 0：拒绝 CI/PowerShell 重复命令守卫遗漏 `cargo +toolchain` 形式，并指出里程碑文字提前声明复审收敛。两个问题均已完成 TDD/记录修复；1175 因内容变化失效，尚不能用于 managed validation 或提交。
- snapshot1176 exact5 独立复审为 Critical 0 / Important 0 / Moderate 1 / Minor 0：确认 toolchain/job、规则状态机、CI 去重和 pending 状态均正确，但拒绝模块文档把 PowerShell `-Only` 误述为可重复参数。focused invocation 已复现参数绑定 RED，新增文档契约测试后硬切为 Python 重复 `--only`、PowerShell 单数组 `-Only`，1176 因内容变化失效；必须生成 successor snapshot 并重新复审。
- snapshot1208 exact5 独立复审为 Critical 0 / Important 0 / Moderate 0 / Minor 0、Ready；审阅方独立运行 Python 重复 `--only`、PowerShell 单数组 `-Only`、全量 runner `19/19` 和 exact diff-check，并完成双遍及结束 preview 无漂移。本记录状态回写会生成新的 successor snapshot；1208 作为已接受的实现/文档内容证据保留，不直接用于变化后的提交 manifest。
- snapshot1219 exact5 对 runner 与文档本身为 Critical 0 / Important 0 / Moderate 0 / Minor 0，但与 snapshot1220 Frameworks05 exact2 联合复审时发现 Plugins04 caller 仍引用不存在的 `crate::sequence` 根模块，整体结论为 Important 1 / Not Ready。当前 caller/guard/本记录均已变化，1219/1220 仅保留历史拒绝证据；受管 Plugins04 编译后必须生成 successor snapshot 并重新独立复审。
- snapshot1223 exact5 独立复审为 Critical 0 / Important 0 / Moderate 1 / Minor 0：状态和主证据段已正确拒绝旧假阳性，但命令证据行仍把 122.235 秒结果写成 GREEN，且未指明 caller hard cut 后 120.095 秒才是当前 fresh 证据。本记录已修正该矛盾，1223 因内容变化失效；successor 必须重新独立复审。

## 待完成测试阶段

- 在没有其他 owner 修改 Runtime 全编译输入的安静窗口，通过 coordinator 运行 fresh managed `structure_convention`、workspace fmt 与 scoped clippy；不得把 `-DryRun`、历史 Cargo 或 source-raced gate 当验收。
- managed 命令前后重新验证不可变 manifest；只有测试阶段 GREEN、计划记录 successor snapshot 复审通过并完成 coordinator milestone commit 后，本切片才能改为 `accepted`。

## 里程碑判定

统一 runner 实现和本切片非 Cargo 契约保持 GREEN；Frameworks05 layer guard 与 Plugins04 caller 的静态 hard cut 已收敛，但下级 Plugins04 受管编译、successor 独立复审及本切片 managed Cargo 测试阶段均未完成。因此 Frameworks06 M2 与总计划保持 `in_progress`，当前记录不声明完成或可提交。
