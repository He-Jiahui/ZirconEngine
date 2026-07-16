---
related_code:
  - docs/plans/zircon_editor/editor/03/fixed-2026-07-15-external-source-cubemap-contract-api-drift.md
  - docs/plans/zircon_editor/editor/03/fixed-2026-07-15-manager-resolver-weak-core-test-consumer-drift.md
  - docs/plans/zircon_editor/editor/03/fixed-2026-07-15-plugin-mirror-v1-runtime-fallback.md
  - docs/plans/zircon_editor/editor/03/fixed-2026-07-15-volumetric-fog-component-id-export-drift.md
  - zircon_editor/src/core/editing/operation
  - zircon_editor/src/core/editing/engine/transaction
  - zircon_editor/src/core/gateway
  - zircon_runtime/src/operation
  - zircon_runtime_interface/src/runtime_api/operation.rs
plan_sources:
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py --repo-root .
  - git diff --cached --name-only
status: resolving_failure
---

# Editor03 M3.2 operation factory/runtime wiring exact-manifest 恢复记录

Plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
Milestone: M3.2
Status: completed
Files: [".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_abi_inventory.py", ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/dynamic_runtime_api_failure_inventory.py", "docs/plans/zircon_editor/editor/03/fixed-2026-07-15-external-source-cubemap-contract-api-drift.md", "docs/plans/zircon_editor/editor/03/fixed-2026-07-15-manager-resolver-weak-core-test-consumer-drift.md", "docs/plans/zircon_editor/editor/03/fixed-2026-07-15-plugin-mirror-v1-runtime-fallback.md", "docs/plans/zircon_editor/editor/03/fixed-2026-07-15-volumetric-fog-component-id-export-drift.md", "docs/zircon_editor/core/commands.md", "docs/zircon_runtime/operation.md", "docs/zircon_runtime_interface/runtime_api.md", "zircon_app/src/entry/entry_runner/editor.rs", "zircon_app/src/entry/runtime_library/runtime_session/operation.rs", "zircon_app/src/entry/runtime_library/tests.rs", "zircon_editor/src/core/commands/contribution.rs", "zircon_editor/src/core/commands/defaults.rs", "zircon_editor/src/core/commands/descriptor.rs", "zircon_editor/src/core/commands/menu.rs", "zircon_editor/src/core/commands/mod.rs", "zircon_editor/src/core/commands/registry.rs", "zircon_editor/src/core/editing/context.rs", "zircon_editor/src/core/editing/engine/command.rs", "zircon_editor/src/core/editing/engine/history.rs", "zircon_editor/src/core/editing/engine/mod.rs", "zircon_editor/src/core/editing/engine/transaction.rs", "zircon_editor/src/core/editing/engine/transaction/operation_group.rs", "zircon_editor/src/core/editing/mod.rs", "zircon_editor/src/core/editing/operation/command.rs", "zircon_editor/src/core/editing/operation/error.rs", "zircon_editor/src/core/editing/operation/factory.rs", "zircon_editor/src/core/editing/operation/mod.rs", "zircon_editor/src/core/editing/operation/registration.rs", "zircon_editor/src/core/editor_operation.rs", "zircon_editor/src/core/gateway/contract.rs", "zircon_editor/src/core/gateway/detached.rs", "zircon_editor/src/core/gateway/error.rs", "zircon_editor/src/core/gateway/handle.rs", "zircon_editor/src/core/gateway/mod.rs", "zircon_editor/src/tests/commands/operation_factory.rs", "zircon_editor/src/tests/editing/transaction_engine/operation_group.rs", "zircon_editor/src/tests/gateway/handle.rs", "zircon_editor/src/tests/gateway/mod.rs", "zircon_editor/src/ui/host/editor_operation_dispatch.rs", "zircon_plugins/navigation/editor/src/operation_command/command.rs", "zircon_plugins/navigation/editor/src/operation_command/error.rs", "zircon_plugins/navigation/editor/src/operation_command/mod.rs", "zircon_plugins/navigation/runtime/src/tests/operation.rs", "zircon_runtime/src/core/framework/navigation/operation.rs", "zircon_runtime/src/dynamic_api/mod.rs", "zircon_runtime/src/dynamic_api/tests/operation.rs", "zircon_runtime/src/navigation/mod.rs", "zircon_runtime/src/navigation/operation/handler.rs", "zircon_runtime/src/navigation/operation/mod.rs", "zircon_runtime/src/navigation/operation/registration.rs", "zircon_runtime/src/operation/context.rs", "zircon_runtime/src/operation/error.rs", "zircon_runtime/src/operation/handler.rs", "zircon_runtime/src/operation/mod.rs", "zircon_runtime/src/operation/service.rs", "zircon_runtime/src/operation/task.rs", "zircon_runtime/src/operation/tests.rs", "zircon_runtime_interface/src/runtime_api/operation.rs", "zircon_runtime_interface/src/tests/runtime_operation.rs", "zircon_runtime_interface/src/version.rs"]

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `INDEPENDENT REVIEW BLOCKED / FOREIGN DEPENDENCIES IDENTIFIED` | 2026-07-16 | HEAD `5ab51df733bf01c13a4af9fa85c2ef99489f9659` 上对当前 63 文件执行独立复核，初始结果 `P0=1 / P1=3 / P2=1`，不得提交。P0 已定位为清单外 `zircon_editor/src/tests/editing/transaction_engine/fixture.rs`：EditorLayout15 已加入 `runtime_gateway` 适配，current SHA-256 `bd5af0dca1fdfbdec085a2e33a1c61fefa18b6837386c2fbc43cbfa4cbceb397`，但仍未落 HEAD；Editor03 不吸收 foreign 文件，必须等待其先行提交。P1 的 `tests/gateway/mod.rs` 当前 12 字节内容为合法 `mod handle;`，fresh current-source 不复现；其余生产闭环依赖 Editor12 的 `ui/host/editor_extension_registration.rs` factory drain/attach 与 Plugins05 `navigation-bake-selection-operation-arguments` open lifecycle，均不得塞入本清单。P2 文档已修正：undo display metadata 属于 `OperationCommandFactoryRegistration`，不在 `EditorCommandDescriptor`。cubemap fixed 仍保留在 63 文件内；待依赖落 HEAD 后重新 attribution、managed validation 与独立复核。 |
| `CURRENT 63-PATH MANIFEST CLAIMED / COORDINATOR CLOSEOUT FAILURE OPEN` | 2026-07-16 | schema 36 authoritative instance `e3a97c6e45114976a5175fd4329fc11a` 上重新读取 HEAD `d6b073132ab57709d173962171d1b84d832c34ab`：旧 67 文件基线中 4 个依赖路径已落 HEAD，剩余 63 个 dirty 路径形成当前唯一 M3.2 candidate；63/63 无 foreign live lease，已由 `editor03-operation-factory-runtime-wiring-20260715` 逐路径 claim。强制 handoff `fixed-2026-07-15-external-source-cubemap-contract-api-drift.md` 位于清单内，SHA-256 仍为 `b1843058ebddc19ed691cdf3152f9489d8b7d9bd6ae6518dc2ccbd55239cc593`、长度 `5102`。Editor03 fixing-plan open Failure 为 `[]`，handoff validator 为 `155 artifacts / 0 errors`。当前不能宣称 closeout：Coordinator01 新登记 `native-slice-closeout-checker-staged-index-contract-drift`，其 checker 仍拒绝 `M<n>.<slice>` 且要求调用前污染共享 index；同计划既有 same-topology evidence drift 也已使 Editor02 version 3 证据失效。Editor03 可继续 current-hash attribution、prepare、managed validation 与 review，但在 checker 回传前不得原子 commit。 |
| `REVIEWED / PREPARE BLOCKED` | 2026-07-15 | 对 Editor03 `M3 / 3.2` 当前 attribution 进行独立只读审计，结果 `P0=0 / P1=5 / P2=1`。`fixed-2026-07-15-external-source-cubemap-contract-api-drift.md` 已确认属于 Editor03 child owner，且必须进入下一份 exact manifest；当前两个 Editor03 workflow run 都没有 topology、prepared manifest 或 commit intent，因此尚未产生可提交清单。Git index 为 `0 staged paths`。 |
| `HANDOFF MANIFEST REQUIRED` | 2026-07-15 | `fixed-2026-07-15-external-source-cubemap-contract-api-drift.md` SHA-256 仍为 `b1843058ebddc19ed691cdf3152f9489d8b7d9bd6ae6518dc2ccbd55239cc593`，内容和既有 3/3 evidence 未变；该文件与本记录必须进入下一份 exact manifest，不得回流 Shader06/Shader04 业务提交。全库 handoff validator fresh 结果为 `150 artifacts / 0 errors`。 |
| `PREPARE FAILED / PLAN METADATA OWNER IDENTIFIED` | 2026-07-15 | fresh `milestone prepare --milestone M3` 在 run `a6aec66d7900415e8acbc9528dff8518` 返回 `workflow_topology_missing`，尚未选择或暂存任何业务路径。根因是本计划仍使用 `### M1..M4` 人类标题且缺少 `zircon-workflow` 机器块；独立维护 Session `editor03-plan-topology-maintenance-20260715` 负责补充 `M1 -> M2 -> M3 -> M4` milestone 拓扑与 1.1 至 4.2 slice 节点、同步当前计划状态，不扩展旧格式兼容解析。后续只允许 prepare `M3.2`，不得把该切片记为整个 M3 accepted。 |
| `NATIVE SLICE BACKEND GREEN / ATOMIC LANDING BLOCKED` | 2026-07-15 | Coordinator01 当前源码已让 `M3.2` 可走 typed action、manifest、gate refresh、review 与 commit，并以 sibling/dependency 阻塞测试证明 slice 成功不会接受父 M3；相关 fresh 回归 `46/46`、干净 HEAD 最小闭包复放 `3/3`。但协调器最小 6 文件闭包包含 active artifact-governance owner 的既有改动，且显式 support/failure-return lifecycle 仍 open，尚无合法 Coordinator01 原子 commit；因此本计划 topology 维护提交与 M3.2 prepare 继续等待，不声明已提交。 |
| `PLAN TOPOLOGY COMMITTED / M3.2 BUSINESS PREPARE STILL BLOCKED` | 2026-07-15 | 受保护主计划已通过一文件 coordinator maintenance commit `4ca06c35c136fac0cf8dbe9cc8898e3ef47fb798` 落地，主题 `docs(editor): register Editor03 workflow topology`；受管事务内 `test_workflow_topology` 13/13 通过，plan-specific parser 得到 topology hash `5a5b199c5bcdfe675eb23fef5d07af38fd7e7c6674db92051d4dba6be002dfb7`，节点为 M1→M2→M3→M4 与 M1.1 至 M4.2 共 10 个 slices，包含唯一 `M3.2`。提交后 shared staged count 为 0，全库 handoff validator 为 `151 artifacts / 0 errors`；cubemap fixed SHA-256 仍为 `b1843058ebddc19ed691cdf3152f9489d8b7d9bd6ae6518dc2ccbd55239cc593`。本维护提交没有吸收 recovery record、fixed handoff 或任何业务代码；它们继续等待下一份 M3.2 exact manifest。 |

## 下一份 exact manifest 的强制项

下一次 Editor03 `M3.2` manifest 必须包含：

- `docs/plans/zircon_editor/editor/03/fixed-2026-07-15-external-source-cubemap-contract-api-drift.md`；
- 本产出记录；
- 其余已经归属于 Editor03 operation/gateway/transaction/runtime-operation 切片且哈希重新归因成功的业务文件。

fresh current-hash 审计共有 72 条 attribution rows：64 个当前 dirty 路径哈希匹配、6 个路径哈希漂移、2 条不可用记录（1 条把数十个路径拼成单个空格字符串，1 条为已删除的旧 `failure-2026-07-13-plugin-operation-factory-runtime-wiring.md`）。以下路径必须排除，不得为了凑齐 manifest 混入：

- `docs/plans/zircon_runtime/runtime/10/failure-2026-07-15-dynamic-runtime-v1-fallback-reintroduced.md`：`outside_registered_child`，仍属于 Runtime10 open lifecycle；
- `docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md` 已以受保护 maintenance commit `4ca06c35c136fac0cf8dbe9cc8898e3ef47fb798` 落 HEAD，不再是 dirty business candidate。
- `zircon_plugins/navigation/editor/src/operation_command/factory.rs` 与 `zircon_plugins/navigation/editor/src/tests/operation_command.rs`：当前漂移新增 selected-surface typed arguments，属于 Plugins05 `navigation-bake-selection-operation-arguments` open lifecycle；必须由 Plugins05 完整实现真实选择态并先行提交，Editor03 不重归因其 partial changes。

其余 3 个 Editor03 自有漂移路径是 runtime-interface loader 文档、commands 文档与 app runtime-library tests；逐项确认当前内容仍属于 V2 operation ABI/command factory 后重归因，64 + 3 得到当前预期 67 文件 exact manifest。该 67 已包含本 recovery record、cubemap fixed 与另外三份 Editor03 fixed handoff；数字只作为 prepare 前核对基线，必须以依赖落地后的 fresh attribution 和逐路径 hash 为准，不能复用旧 run。

## 2026-07-16 当前 exact manifest

本记录的 `Files:` 已从历史 66 个业务路径收敛为当前 62 个 dirty 业务路径；加上本记录自身，
M3.2 candidate 总数为 63。以下 4 个路径已由先行依赖落 HEAD，不再进入 dirty manifest：

- `docs/engine-architecture/runtime-interface-cdylib-loader.md`；
- `zircon_app/src/entry/runtime_library/loaded_runtime.rs`；
- `zircon_runtime/src/dynamic_api/exports.rs`；
- `zircon_runtime/src/dynamic_api/tests/api_table.rs`。

当前 63 文件继续排除 Plugins05 尚未完成的 selected-surface factory/test partial changes、
受保护主计划、Runtime10/Runtime04 owner 记录与任何 Shader04 文件。cubemap fixed handoff 必须保留在
63 文件 manifest 中；不得因其来源是 Shader06 而回流 Shader06 业务提交。

## 当前提交阻断

1. 受保护 topology 计划已在 `4ca06c35c136fac0cf8dbe9cc8898e3ef47fb798` 落 HEAD；旧 run `a6aec66d7900415e8acbc9528dff8518` 仍没有已激活 topology、prepared manifest 或 commit intent。必须先由正确 Coordinator01 owner 原子落地 node-scoped Failure、same-topology version 保留与 prepare identity，再对当前 Editor03 Session 执行 fresh topology activation，且只允许 prepare `M3.2`。
2. 三个 Editor03 自有业务路径的哈希必须在 prepare 前重新归因：runtime-interface loader 文档、commands 文档与 app runtime-library tests。Navigation operation factory/对应测试等待 Plugins05 提交后从 dirty scope 消失；受保护主计划已独立提交，Runtime10 open failure 继续排除。
3. Plugins12/Frameworks05/Text03 持有或漂移的 dynamic-session/V2 API 依赖尚未先行落 HEAD，`dynamic_api/session/state.rs` 仍未跟踪且无 owner。
4. Editor02 的 inspection 先行依赖已收到 Runtime15 fixed return，并扩为 output record、scene inspection 文档、两份 Rust 文件与 fixed handoff 共 5 文件 exact manifest；它仍被 Coordinator01 support-slice Failure 阻断，必须先原子落地，Editor03 不吸收这 5 个 foreign paths。
5. attribution 元数据仍有一条把几十个路径拼成单个空格字符串的畸形记录；逐路径记录虽然存在，但 prepare 前必须由 coordinator owner 清理该污染。

## 提交纪律

- 不手工 `git add`、`git commit` 或 unstage 其他会话文件。
- 不把 Runtime10 open failure、受保护主计划、Shader06/Shader04 文件或 foreign dependency 塞进 Editor03 业务提交。
- 只有先行依赖落 HEAD、哈希重新归因、fresh prepare 成功、independent review 与 managed validation 均通过后，才能执行 coordinator milestone commit。

当前状态保持 `resolving_failure`；M3.2 实现已完成，但 closeout 尚未完成，不声明已提交或父 M3 完成。

## Scope delivered

M3.2 完成 operation factory 注册、统一事务组映射、editor gateway、Runtime V2 operation ABI、动态 API/loader 必需函数指针缓存、app consumer 与 Navigation runtime operation 接线；旧 V1/capability fallback、旧 operation stack 和兼容路径不保留。当前 `Status: completed` 只描述该 slice 的实现与复核完成，不代表已提交、父 M3 完成或外部 Failure 已关闭。

## Fresh testing evidence

- Editor03 主计划 topology maintenance commit `4ca06c35c136fac0cf8dbe9cc8898e3ef47fb798` 的 `test_workflow_topology` 为 13/13，plan-specific parser hash 为 `5a5b199c5bcdfe675eb23fef5d07af38fd7e7c6674db92051d4dba6be002dfb7`。
- Navigation editor/runtime 与 `zircon_app` 既有 current-source 受管门已通过；完整 runtime 门已越过 Editor03/Runtime10/Shader04 编译范围，后续 Runtime04 与 Plugins05 功能缺口均已进入各自 open lifecycle。
- 全库 handoff validator fresh 为 `151 artifacts / 0 errors`，scoped diff-check 通过，shared staged count 为 0；cubemap fixed SHA-256 为 `b1843058ebddc19ed691cdf3152f9489d8b7d9bd6ae6518dc2ccbd55239cc593`。

## Review

M3.2 核心实现既有独立复核为 `P0=0`；V2-only、required-pointer 缓存、progress/result 双层 ABI、`Applied` 后 fault 与 retained factory→transaction 路径均通过。剩余 P1 均是提交顺序/外部 owner：Editor02 五文件先行切片、Coordinator01 node-scoped manifest lifecycle、Plugins05 selected-surface 参数投影与 Runtime10/Runtime04 open Failure；本记录不吸收这些 foreign paths，也不把 M3.2 冒充父 M3 accepted。
