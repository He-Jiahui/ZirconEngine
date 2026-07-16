---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: plugin-mirror-v1-runtime-fallback
origin_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
fixing_plan: docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
origin_child_dir: docs/plans/zircon_editor/editor/03
fixing_child_dir: docs/plans/zircon_plugins/12
related_code:
  - zircon_runtime/src/dynamic_api/session/linked_plugins.rs
  - zircon_runtime/src/dynamic_api/tests/linked_plugins.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_editor/src/core/runtime_event_consumer/host.rs
tests:
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_runtime -SkipBuild -VerboseOutput
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_app -SkipBuild -VerboseOutput
resolved_at: 2026-07-15
---


# Plugins 12: plugin mirror must not restore the V1 runtime fallback

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 来源执行切片：M3 operation factory/runtime dynamic API testing stage
- 修复责任计划：`docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md`
- 交接原因：Plugins12 的 typed event mirror、linked runtime plugin composition 和 PIE consumer lifecycle 属于插件 DX owner；这些功能当前以重新引入 V1/V2 双表和 app fallback 为代价，插件计划必须移除该集成错误。

## 失败现象与复现证据

Plugins12 会话在 2026-07-15 08:00 +08:00 的协调器状态中明确记录 “V1/V2 ABI ... implemented”。当前源码同时存在 `ZrRuntimeApiV1`、`ZrRuntimeApiV2`、两个导出符号以及 app loader 的 V2-to-V1 fallback；结构审计因此错误报告 function tables `11/11` 且 `risks = []`。这与用户批准的 V2-only hard cutover 及 `docs/zircon_runtime/operation.md` 的无 V1 table/export/fallback 合同冲突。

关联的 Runtime10 根 failure 为 `../../zircon_runtime/runtime/10/failure-2026-07-15-dynamic-runtime-v1-fallback-reintroduced.md`。Runtime10 负责唯一 ABI table；Plugins12 负责证明 mirror/linked runtime/PIE 路径不依赖旧 table。

## 最低共享层根因

插件 mirror 修复把“旧 host/runtime 能力协商”误建模为 V1 table fallback。实际架构已经要求所有 app/runtime/plugin consumer 同步切到 V2 table；可选插件事件能力通过 V2 required group 和 typed manifest/lifecycle 管理，而不是通过旧 table 降级。

## 架构修复验收

- 保留 linked runtime plugin registration、World tick/report drain、subscription refcount、editor catalog injection 与 PIE reconcile 的新版实现。
- 删除 Plugins12 为这些功能引入或要求的 V1 runtime table/export/loader fallback；所有 mirror 与 operation consumer 只通过完整 V2 table。
- V2 table 的 plugin event subscribe/unsubscribe/drain 与 operation submit/poll/harvest group 保持显式校验，任何缺失均为加载错误，不回退。
- Runtime10 V2-only 静态审计通过后，Plugins12 focused runtime/app/editor gates 沿正常路径通过。

## 禁止临时方案

- 不得保留 `RuntimeApi::V1` 分支、V1 symbol lookup、V1 table fixture、capability-unavailable 降级或 dual-table guard。
- 不得删除 typed mirror、linked plugin composition 或 PIE consumer 来规避 V2 集成。
- 不得弱化 Runtime10 或 Editor03 的结构守卫与测试。

## 修复结果与回传

- 根因：The editor mirror integration depended on a legacy Runtime API V1 lookup/fallback path that conflicted with the shared V2-only dynamic ABI owner.
- 架构修复：Hard-cut dynamic runtime loading to the single 19-field V2 table and V2 export/lookup path, removed V1 fallback surfaces, and split the session implementation into bounded state, FFI, construction, hook, and operation owners.
- 验证：V2-only dynamic audit passes 48/48 with loader failure anchors 13/13, legacy V1 hits `[]`, missing `[]`, and risks `[]`; schedule and performance audits report risks=[]; focused Frameworks05 Python guard passes 1/1; independent review accepted with 0 Critical and 0 Important.
- 回传：Plugins12 provides the V2-only mirror runtime path and returns the fixed handoff to Editor03.
