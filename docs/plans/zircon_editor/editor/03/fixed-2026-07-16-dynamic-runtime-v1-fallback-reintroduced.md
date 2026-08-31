---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: dynamic-runtime-v1-fallback-reintroduced
origin_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
origin_child_dir: docs/plans/zircon_editor/editor/03
fixing_child_dir: docs/plans/zircon_runtime/runtime/10
related_code:
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
tests:
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime_interface -SkipBuild -VerboseOutput
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild -VerboseOutput
resolved_at: 2026-07-16
---


# Runtime 10: dynamic runtime V1 fallback reintroduced

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 来源执行切片：M3 operation factory/runtime dynamic API testing stage
- 修复责任计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 交接原因：动态 runtime C ABI 表、导出符号及 app loader 的唯一 owner 属于 Runtime 10；Editor03 只消费该边界，不能在编辑器侧掩盖 ABI 回退。

## 失败现象与复现证据

2026-07-15 的 V2-only hard cutover 已删除 V1 table/export/loader fallback，但并行 Plugins12 工作重新引入了旧兼容面。当前只读搜索再次发现：

- `zircon_runtime_interface/src/runtime_api/api_table.rs` 定义 `ZrRuntimeApiV1`、`ZrRuntimeGetApiFnV1` 和 `ZR_RUNTIME_GET_API_SYMBOL_V1`；
- `zircon_runtime/src/dynamic_api/exports.rs` 导出 `zircon_runtime_get_api_v1` 并保留 `RUNTIME_API_V1`；
- `zircon_app/src/entry/runtime_library/loaded_runtime.rs` 在 V2 symbol 缺失时回退到 V1；
- interface/runtime/app 测试与结构审计重新把 V1 table 当成受支持产品合同；
- `docs/zircon_runtime/operation.md` 已明确要求 “There is no V1 table export, loader fallback, compatibility wrapper”。

这直接违反用户要求的“不再兼容旧的架构情况，所有旧的架构设计改造升级为新版”，并使 Editor03 operation submit/poll/harvest 在 V1 分支上退化为 capability unavailable。

## 最低共享层根因

Runtime 10 的版本策略被错误解释为“冻结 V1 并同时新增 V2”。本项目当前迁移规则要求的是产品级硬切换：DTO/host callback 中仍以 `V1` 命名的稳定记录可继续存在，但旧的 runtime function table、旧导出符号和 loader fallback 不能继续作为可运行架构。

## 架构修复验收

- runtime table 只保留 `ZrRuntimeApiV2`、`ZrRuntimeGetApiFnV2`、`ZR_RUNTIME_GET_API_SYMBOL_V2` 和 `zircon_runtime_get_api_v2`；旧 table/export/loader symbol 在生产源与当前结构守卫中为零命中。
- `zircon_app` 只装载完整 V2 table；缺少 V2 symbol、table 过短或 operation/mirror required group 不完整时返回显式错误，不回退到 V1。
- interface、runtime、app 相关测试与 `dynamic_runtime_api_boundary.py` 同步到 V2-only inventory，且静态审计 `risks = []`。
- 受管 Windows 验证依次通过 `zircon_runtime_interface`、`zircon_runtime` 和受影响的 `zircon_app` editor-host 路径，再允许 Editor03 operation gate 恢复。

## 禁止临时方案

- 不得保留 V1 table、V1 export、V1 loader fallback、兼容枚举、兼容 accessor 或 V1-only test fixture。
- 不得把 V1 fallback 描述为 ABI 稳定性、外部库支持或 capability negotiation。
- 不得弱化 operation/mirror required-group 校验，也不得在 Editor03 侧增加 capability unavailable 绕行。

## 修复结果与回传

- 根因：Parallel plugin mirror work reintroduced the retired runtime V1 function table, export symbol, and app loader fallback after the product hard cut to V2.
- 架构修复：Removed the V1 table/export/loader fallback and converged interface, runtime exports, app loader, 17 FFI wrappers, operation and plugin-mirror required groups, source inventory, split owners, and mirror documentation on one V2-only contract without compatibility shims.
- 验证：dynamic_runtime_api_boundary: risks=[], legacy_runtime_api_hits=[], 17 wrappers, no missing owners; zircon_runtime_interface managed job d52d3bd9891941bb9d72ca3f8746dffc exit 0; zircon_app managed job 376c7ad2e2f141b79b3c9ad39e979557 exit 0; Runtime10 managed dynamic_api green3 reached 98 passed/1 stale source-guard failure/10 ignored, then exact current-source compile job 1f2ee891ade941eb9c21229bb82f9b3e exit 0 plus direct source assertion; rustfmt/diff checks passed.
- 回传：Runtime10 V2-only function table, export, loader and complete operation/mirror groups are restored; Editor03 may resume its operation-factory runtime gate. Full zircon_runtime package acceptance remains independently blocked by foreign Frameworks05/Text integration work and is not claimed here.
